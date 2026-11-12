//! Google Gemini (Generative Language API) adapter — streaming + function calls.

use super::provider::{ChatRequest, LlmProvider, ProviderCaps};
use super::sse::sse_json_stream;
use super::types::{Message, Role, StreamEvent, ToolCall};
use crate::error::{AppError, Result};
use async_trait::async_trait;
use futures::StreamExt;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct GeminiConfig {
    pub id: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            id: "gemini".into(),
            base_url: "https://generativelanguage.googleapis.com/v1beta".into(),
            api_key: String::new(),
            model: "gemini-1.5-pro".into(),
        }
    }
}

pub struct GeminiProvider {
    cfg: GeminiConfig,
    client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new(cfg: GeminiConfig) -> Self {
        Self { cfg, client: reqwest::Client::new() }
    }

    fn endpoint(&self, model: &str, stream: bool) -> String {
        let method = if stream { "streamGenerateContent" } else { "generateContent" };
        let base = self.cfg.base_url.trim_end_matches('/');
        format!("{base}/models/{model}:{method}?alt=sse&key={}", self.cfg.api_key)
    }

    fn build_body(&self, req: &ChatRequest) -> Value {
        let contents: Vec<Value> = req
            .messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(|m| self.encode_message(m))
            .collect();

        let mut body = json!({ "contents": contents });

        let system = req.system.clone().or_else(|| {
            req.messages.iter().find(|m| m.role == Role::System).map(|m| m.content.clone())
        });
        if let Some(s) = system {
            body["systemInstruction"] = json!({ "parts": [{ "text": s }] });
        }

        let mut gen_cfg = json!({});
        if let Some(t) = req.temperature {
            gen_cfg["temperature"] = json!(t);
        }
        if let Some(mt) = req.max_tokens {
            gen_cfg["maxOutputTokens"] = json!(mt);
        }
        // Extended thinking (Gemini 2.5). 0 disables it, -1 = dynamic budget.
        if let Some(level) = req.thinking {
            gen_cfg["thinkingConfig"] = if level == crate::ai::types::ThinkingLevel::Auto {
                json!({ "thinkingBudget": -1 })
            } else {
                json!({ "thinkingBudget": level.budget_tokens() })
            };
        }
        if gen_cfg.as_object().map(|o| !o.is_empty()).unwrap_or(false) {
            body["generationConfig"] = gen_cfg;
        }

        if !req.tools.is_empty() {
            let decls: Vec<Value> = req
                .tools
                .iter()
                .map(|t| json!({
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters,
                }))
                .collect();
            body["tools"] = json!([{ "functionDeclarations": decls }]);
        }
        body
    }

    fn encode_message(&self, m: &Message) -> Value {
        match m.role {
            Role::Tool => json!({
                "role": "user",
                "parts": [{
                    "functionResponse": {
                        "name": m.name.clone().unwrap_or_else(|| "tool".into()),
                        "response": { "result": m.content },
                    }
                }]
            }),
            Role::Assistant => {
                let mut parts: Vec<Value> = vec![];
                if !m.content.is_empty() {
                    parts.push(json!({ "text": m.content }));
                }
                for c in &m.tool_calls {
                    parts.push(json!({
                        "functionCall": { "name": c.name, "args": c.arguments }
                    }));
                }
                json!({ "role": "model", "parts": parts })
            }
            _ => json!({ "role": "user", "parts": [{ "text": m.content }] }),
        }
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    fn id(&self) -> &str {
        &self.cfg.id
    }

    fn capabilities(&self) -> ProviderCaps {
        ProviderCaps { streaming: true, tool_calling: true, vision: true, json_mode: true }
    }

    async fn chat_stream(&self, req: ChatRequest, tx: mpsc::Sender<StreamEvent>) -> Result<()> {
        if self.cfg.api_key.is_empty() {
            let _ = tx.send(StreamEvent::Error { message: "missing API key".into() }).await;
            return Err(AppError::Auth);
        }
        let model = if req.model.is_empty() { self.cfg.model.clone() } else { req.model.clone() };
        let resp = self
            .client
            .post(self.endpoint(&model, true))
            .json(&self.build_body(&req))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            let msg = format!("gemini returned {status}: {body}");
            let _ = tx.send(StreamEvent::Error { message: msg.clone() }).await;
            return Err(AppError::Provider(msg));
        }

        let mut stream = sse_json_stream(resp);
        let mut tools: BTreeMap<String, (String, Value)> = BTreeMap::new();

        while let Some(item) = stream.next().await {
            let payload = match item? {
                Some(p) => p,
                None => break,
            };
            let ev: Value = match serde_json::from_str(&payload) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if let Some(candidates) = ev["candidates"].as_array() {
                for cand in candidates {
                    if let Some(parts) = cand["content"]["parts"].as_array() {
                        for part in parts {
                            if let Some(text) = part["text"].as_str() {
                                if !text.is_empty() {
                                    let _ = tx.send(StreamEvent::Token { text: text.to_string() }).await;
                                }
                            }
                            if let Some(fc) = part.get("functionCall") {
                                let name = fc["name"].as_str().unwrap_or_default().to_string();
                                let args = fc["args"].clone();
                                tools.insert(format!("{name}_{}", tools.len()), (name, args));
                            }
                        }
                    }
                }
            }
            if let Some(usage) = ev.get("usageMetadata") {
                let p = usage["promptTokenCount"].as_u64().unwrap_or(0) as u32;
                let c = usage["candidatesTokenCount"].as_u64().unwrap_or(0) as u32;
                let _ = tx.send(StreamEvent::Usage { prompt: p, completion: c }).await;
            }
        }

        for (i, (name, args)) in tools.into_values().enumerate() {
            let _ = tx
                .send(StreamEvent::ToolCallComplete(ToolCall {
                    id: format!("call_{i}"),
                    name,
                    arguments: args,
                }))
                .await;
        }
        let _ = tx.send(StreamEvent::Done).await;
        Ok(())
    }

    async fn test_connection(&self) -> Result<bool> {
        let url = format!(
            "{}/models?key={}",
            self.cfg.base_url.trim_end_matches('/'),
            self.cfg.api_key
        );
        let resp = self.client.get(url).send().await?;
        Ok(resp.status().is_success())
    }
}
