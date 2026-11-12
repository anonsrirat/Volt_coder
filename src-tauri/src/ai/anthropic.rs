//! Anthropic (Claude) Messages API adapter — streaming + tool use.

use super::provider::{ChatRequest, LlmProvider, ProviderCaps};
use super::sse::sse_json_stream;
use super::types::{Message, Role, StreamEvent, ToolCall};
use crate::error::{AppError, Result};
use async_trait::async_trait;
use futures::StreamExt;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    pub id: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            id: "anthropic".into(),
            base_url: "https://api.anthropic.com/v1".into(),
            api_key: String::new(),
            model: "claude-3-5-sonnet-latest".into(),
        }
    }
}

pub struct AnthropicProvider {
    cfg: AnthropicConfig,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(cfg: AnthropicConfig) -> Self {
        Self { cfg, client: reqwest::Client::new() }
    }

    fn endpoint(&self) -> String {
        format!("{}/messages", self.cfg.base_url.trim_end_matches('/'))
    }

    fn build_body(&self, req: &ChatRequest) -> Value {
        let messages: Vec<Value> = req
            .messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(|m| self.encode_message(m))
            .collect();

        let mut body = json!({
            "model": if req.model.is_empty() { self.cfg.model.clone() } else { req.model.clone() },
            "messages": messages,
            "max_tokens": req.max_tokens.unwrap_or(4096),
            "stream": true,
        });

        // Extended thinking → Anthropic's `thinking` block. The output budget must
        // stay larger than the thinking budget, so we bump max_tokens if needed.
        if let Some(level) = req.thinking {
            if level != crate::ai::types::ThinkingLevel::Auto && level.budget_tokens() > 0 {
                let budget = level.budget_tokens();
                let max = req.max_tokens.unwrap_or(4096).max(budget + 1024);
                body["max_tokens"] = json!(max);
                body["thinking"] = json!({ "type": "enabled", "budget_tokens": budget });
            }
        }

        let system = req.system.clone().or_else(|| {
            req.messages.iter().find(|m| m.role == Role::System).map(|m| m.content.clone())
        });
        if let Some(s) = system {
            body["system"] = json!(s);
        }
        if let Some(t) = req.temperature {
            body["temperature"] = json!(t);
        }
        if !req.tools.is_empty() {
            let tools: Vec<Value> = req
                .tools
                .iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "description": t.description,
                        "input_schema": t.parameters,
                    })
                })
                .collect();
            body["tools"] = json!(tools);
        }
        body
    }

    fn encode_message(&self, m: &Message) -> Value {
        match m.role {
            Role::Tool => json!({
                "role": "user",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": m.tool_call_id.clone().unwrap_or_default(),
                    "content": m.content,
                }]
            }),
            Role::Assistant if !m.tool_calls.is_empty() => {
                let mut blocks: Vec<Value> = vec![];
                if !m.content.is_empty() {
                    blocks.push(json!({ "type": "text", "text": m.content }));
                }
                for c in &m.tool_calls {
                    blocks.push(json!({
                        "type": "tool_use",
                        "id": c.id,
                        "name": c.name,
                        "input": c.arguments,
                    }));
                }
                json!({ "role": "assistant", "content": blocks })
            }
            _ => {
                let role = if m.role == Role::Assistant { "assistant" } else { "user" };
                json!({ "role": role, "content": m.content })
            }
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn id(&self) -> &str {
        &self.cfg.id
    }

    fn capabilities(&self) -> ProviderCaps {
        ProviderCaps { streaming: true, tool_calling: true, vision: true, json_mode: false }
    }

    async fn chat_stream(&self, req: ChatRequest, tx: mpsc::Sender<StreamEvent>) -> Result<()> {
        if self.cfg.api_key.is_empty() {
            let _ = tx.send(StreamEvent::Error { message: "missing API key".into() }).await;
            return Err(AppError::Auth);
        }

        let resp = self
            .client
            .post(self.endpoint())
            .header("x-api-key", &self.cfg.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&self.build_body(&req))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            let msg = format!("anthropic returned {status}: {body}");
            let _ = tx.send(StreamEvent::Error { message: msg.clone() }).await;
            return Err(AppError::Provider(msg));
        }

        let mut stream = sse_json_stream(resp);
        // index -> assembled tool call
        let mut tools: BTreeMap<usize, (String, String, String)> = BTreeMap::new();
        let mut prompt_tokens = 0u32;
        let mut completion_tokens = 0u32;

        while let Some(item) = stream.next().await {
            let payload = match item? {
                Some(p) => p,
                None => break,
            };
            let ev: Value = match serde_json::from_str(&payload) {
                Ok(v) => v,
                Err(_) => continue,
            };
            match ev["type"].as_str().unwrap_or_default() {
                "message_start" => {
                    prompt_tokens = ev["message"]["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32;
                }
                "content_block_start" => {
                    let idx = ev["index"].as_u64().unwrap_or(0) as usize;
                    if ev["content_block"]["type"] == "tool_use" {
                        let id = ev["content_block"]["id"].as_str().unwrap_or_default().to_string();
                        let name = ev["content_block"]["name"].as_str().unwrap_or_default().to_string();
                        tools.insert(idx, (id, name, String::new()));
                    }
                }
                "content_block_delta" => {
                    let idx = ev["index"].as_u64().unwrap_or(0) as usize;
                    let delta = &ev["delta"];
                    match delta["type"].as_str().unwrap_or_default() {
                        "text_delta" => {
                            if let Some(t) = delta["text"].as_str() {
                                let _ = tx.send(StreamEvent::Token { text: t.to_string() }).await;
                            }
                        }
                        "input_json_delta" => {
                            if let Some(part) = delta["partial_json"].as_str() {
                                if let Some(entry) = tools.get_mut(&idx) {
                                    entry.2.push_str(part);
                                }
                            }
                        }
                        "thinking_delta" => {
                            if let Some(t) = delta["thinking"].as_str() {
                                let _ = tx.send(StreamEvent::Reasoning { text: t.to_string() }).await;
                            }
                        }
                        _ => {}
                    }
                }
                "message_delta" => {
                    completion_tokens = ev["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32;
                }
                "message_stop" => break,
                "error" => {
                    let msg = ev["error"]["message"].as_str().unwrap_or("unknown").to_string();
                    let _ = tx.send(StreamEvent::Error { message: msg.clone() }).await;
                    return Err(AppError::Provider(msg));
                }
                _ => {}
            }
        }

        for (idx, (id, name, raw)) in tools {
            let args = if raw.trim().is_empty() {
                json!({})
            } else {
                serde_json::from_str(&raw).unwrap_or(json!({ "__raw": raw }))
            };
            let _ = tx
                .send(StreamEvent::ToolCallComplete(ToolCall { id, name, arguments: args }))
                .await;
            let _ = idx;
        }

        if prompt_tokens > 0 || completion_tokens > 0 {
            let _ = tx.send(StreamEvent::Usage { prompt: prompt_tokens, completion: completion_tokens }).await;
        }
        let _ = tx.send(StreamEvent::Done).await;
        Ok(())
    }

    async fn test_connection(&self) -> Result<bool> {
        // Cheap request: a 1-token completion.
        let body = json!({
            "model": self.cfg.model,
            "max_tokens": 1,
            "messages": [{ "role": "user", "content": "hi" }],
        });
        let resp = self
            .client
            .post(self.endpoint())
            .header("x-api-key", &self.cfg.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;
        Ok(resp.status().is_success())
    }
}

// Silence unused import warnings for Deserialize in some feature combos.
#[allow(dead_code)]
#[derive(Deserialize)]
struct _Keep {}
