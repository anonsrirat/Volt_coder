//! OpenAI-compatible adapter.
//!
//! This single adapter covers the vast majority of providers in the wild:
//! OpenAI, Azure OpenAI, OpenRouter, Groq, Together, DeepSeek, Fireworks,
//! Ollama, LM Studio, vLLM and any other service exposing the
//! `/chat/completions` contract.

use super::provider::{ChatRequest, LlmProvider, ProviderCaps};
use super::sse::sse_json_stream;
use super::types::{Message, PartialToolCall, Role, StreamEvent, ToolCall};
use crate::error::{AppError, Result};
use async_trait::async_trait;
use futures::StreamExt;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::Instant;
use tokio::sync::mpsc;

/// Configuration for an OpenAI-compatible endpoint.
#[derive(Debug, Clone)]
pub struct OpenAiConfig {
    pub id: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    /// Extra headers merged into every request (used by `custom`).
    pub extra_headers: BTreeMap<String, String>,
    /// Some providers (e.g. Azure) use a different auth header.
    pub use_bearer: bool,
}

impl OpenAiConfig {
    pub fn endpoint(&self) -> String {
        let base = self.base_url.trim_end_matches('/');
        if base.ends_with("/chat/completions") {
            base.to_string()
        } else {
            format!("{base}/chat/completions")
        }
    }
}

pub struct OpenAiProvider {
    cfg: OpenAiConfig,
    client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new(cfg: OpenAiConfig) -> Self {
        Self { cfg, client: reqwest::Client::new() }
    }

    fn build_body(&self, req: &ChatRequest) -> Value {
        let messages: Vec<Value> = req
            .messages
            .iter()
            .map(|m| self.encode_message(m))
            .collect();

        let mut body = json!({
            "model": if req.model.is_empty() { self.cfg.model.clone() } else { req.model.clone() },
            "messages": messages,
            "stream": true,
            "stream_options": { "include_usage": true },
        });

        if let Some(t) = req.temperature {
            body["temperature"] = json!(t);
        }
        if let Some(mt) = req.max_tokens {
            body["max_tokens"] = json!(mt);
        }
        // Extended thinking → `reasoning_effort` (OpenAI o-series, Groq, ...).
        // `auto` leaves the field out so the model's own defaults apply.
        // Endpoints that don't know the field simply ignore it.
        if let Some(level) = req.thinking {
            if level != crate::ai::types::ThinkingLevel::Auto {
                if let Some(effort) = level.effort() {
                    body["reasoning_effort"] = json!(effort);
                }
            }
        }
        if !req.tools.is_empty() {
            let tools: Vec<Value> = req
                .tools
                .iter()
                .map(|t| {
                    json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters,
                        }
                    })
                })
                .collect();
            body["tools"] = json!(tools);
            body["tool_choice"] = json!("auto");
        }
        body
    }

    fn encode_message(&self, m: &Message) -> Value {
        let role = match m.role {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
        };
        let mut v = json!({ "role": role, "content": m.content });
        if !m.tool_calls.is_empty() {
            let calls: Vec<Value> = m
                .tool_calls
                .iter()
                .map(|c| {
                    json!({
                        "id": c.id,
                        "type": "function",
                        "function": {
                            "name": c.name,
                            "arguments": c.arguments.to_string(),
                        }
                    })
                })
                .collect();
            v["tool_calls"] = json!(calls);
        }
        if let Some(id) = &m.tool_call_id {
            v["tool_call_id"] = json!(id);
        }
        v
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn id(&self) -> &str {
        &self.cfg.id
    }

    fn capabilities(&self) -> ProviderCaps {
        ProviderCaps { streaming: true, tool_calling: true, vision: false, json_mode: true }
    }

    async fn chat_stream(
        &self,
        req: ChatRequest,
        tx: mpsc::Sender<StreamEvent>,
    ) -> Result<()> {
        if self.cfg.api_key.is_empty() && !self.cfg.extra_headers.contains_key("Authorization") {
            let _ = tx.send(StreamEvent::Error { message: "missing API key".into() }).await;
            return Err(AppError::Auth);
        }

        let mut rb = self.client.post(self.cfg.endpoint()).json(&self.build_body(&req));
        if self.cfg.use_bearer && !self.cfg.api_key.is_empty() {
            rb = rb.bearer_auth(&self.cfg.api_key);
        }
        for (k, v) in &self.cfg.extra_headers {
            rb = rb.header(k, v);
        }

        let resp = rb.send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            let msg = format!("provider returned {status}: {body}");
            let _ = tx.send(StreamEvent::Error { message: msg.clone() }).await;
            return Err(AppError::Provider(msg));
        }

        let mut stream = sse_json_stream(resp);
        // Accumulate streamed tool calls keyed by their index.
        let mut pending: BTreeMap<usize, PartialToolCall> = BTreeMap::new();

        while let Some(item) = stream.next().await {
            let payload = match item? {
                Some(p) => p,
                None => break, // [DONE]
            };
            let chunk: Chunk = match serde_json::from_str(&payload) {
                Ok(c) => c,
                Err(_) => continue, // ignore non-JSON keep-alives
            };

            if let Some(usage) = chunk.usage {
                let _ = tx
                    .send(StreamEvent::Usage {
                        prompt: usage.prompt_tokens,
                        completion: usage.completion_tokens,
                    })
                    .await;
            }

            let Some(choice) = chunk.choices.into_iter().next() else { continue };
            if let Some(delta) = choice.delta {
                if let Some(content) = delta.content {
                    if !content.is_empty() {
                        let _ = tx.send(StreamEvent::Token { text: content }).await;
                    }
                }
                for tc in delta.tool_calls {
                    let entry = pending.entry(tc.index).or_default();
                    entry.index = tc.index;
                    if let Some(id) = tc.id {
                        entry.id = Some(id);
                    }
                    if let Some(func) = tc.function {
                        if let Some(name) = func.name {
                            entry.name = Some(name);
                        }
                        if let Some(args) = func.arguments {
                            entry.arguments_delta.push_str(&args);
                        }
                    }
                    let _ = tx
                        .send(StreamEvent::ToolCallDelta(entry.clone()))
                        .await;
                }
            }
        }

        // Emit fully assembled tool calls.
        for (idx, p) in pending {
            let args = if p.arguments_delta.trim().is_empty() {
                json!({})
            } else {
                serde_json::from_str(&p.arguments_delta).unwrap_or(json!({ "__raw": p.arguments_delta }))
            };
            let call = ToolCall {
                id: p.id.unwrap_or_else(|| format!("call_{idx}")),
                name: p.name.unwrap_or_default(),
                arguments: args,
            };
            let _ = tx.send(StreamEvent::ToolCallComplete(call)).await;
        }

        let _ = tx.send(StreamEvent::Done).await;
        Ok(())
    }

    async fn test_connection(&self) -> Result<bool> {
        let start = Instant::now();
        let url = {
            let base = self.cfg.base_url.trim_end_matches('/');
            format!("{base}/models")
        };
        let mut rb = self.client.get(url);
        if self.cfg.use_bearer {
            rb = rb.bearer_auth(&self.cfg.api_key);
        }
        for (k, v) in &self.cfg.extra_headers {
            rb = rb.header(k, v);
        }
        let resp = rb.send().await?;
        tracing::debug!("test_connection {} -> {} in {:?}", self.cfg.id, resp.status(), start.elapsed());
        Ok(resp.status().is_success())
    }
}

// ---------- Wire format ----------

#[derive(Debug, Deserialize)]
struct Chunk {
    #[serde(default)]
    choices: Vec<Choice>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    #[serde(default)]
    delta: Option<Delta>,
}

#[derive(Debug, Deserialize)]
struct Delta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<ToolCallDelta>,
}

#[derive(Debug, Deserialize)]
struct ToolCallDelta {
    index: usize,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    function: Option<FunctionDelta>,
}

#[derive(Debug, Deserialize)]
struct FunctionDelta {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::types::ToolSpec;

    fn cfg() -> OpenAiConfig {
        OpenAiConfig {
            id: "test".into(),
            base_url: "https://api.openai.com/v1".into(),
            api_key: "sk-test".into(),
            model: "gpt-4o".into(),
            extra_headers: BTreeMap::new(),
            use_bearer: true,
        }
    }

    #[test]
    fn endpoint_builds_correctly() {
        let p = OpenAiProvider::new(cfg());
        assert_eq!(p.cfg.endpoint(), "https://api.openai.com/v1/chat/completions");
    }

    #[test]
    fn endpoint_respects_full_path() {
        let mut c = cfg();
        c.base_url = "https://x/v1/chat/completions".into();
        assert_eq!(OpenAiProvider::new(c).cfg.endpoint(), "https://x/v1/chat/completions");
    }

    #[test]
    fn body_includes_tools_when_present() {
        let p = OpenAiProvider::new(cfg());
        let req = ChatRequest {
            messages: vec![Message::user("hi")],
            tools: vec![ToolSpec {
                name: "read_file".into(),
                description: "read".into(),
                parameters: json!({"type": "object"}),
            }],
            model: "gpt-4o".into(),
            temperature: Some(0.2),
            max_tokens: Some(100),
            system: None,
            thinking: None,
        };
        let body = p.build_body(&req);
        assert_eq!(body["tools"][0]["function"]["name"], "read_file");
        assert_eq!(body["stream"], true);
        assert_eq!(body["max_tokens"], 100);
        let temp = body["temperature"].as_f64().unwrap();
        assert!((temp - 0.2).abs() < 1e-5, "temperature was {temp}");
    }
}
