//! Provider abstraction: the single seam every LLM backend plugs into.

use crate::ai::types::{Message, StreamEvent, ThinkingLevel, ToolSpec};
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Optional knobs a caller can pass per request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    #[serde(default)]
    pub tools: Vec<ToolSpec>,
    pub model: String,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub system: Option<String>,
    /// Extended-thinking budget requested for this turn.
    #[serde(default)]
    pub thinking: Option<ThinkingLevel>,
}

/// What a provider is able to do — used to degrade gracefully in the UI/agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCaps {
    pub streaming: bool,
    pub tool_calling: bool,
    pub vision: bool,
    pub json_mode: bool,
}

impl Default for ProviderCaps {
    fn default() -> Self {
        Self { streaming: true, tool_calling: true, vision: false, json_mode: false }
    }
}

/// The core provider contract.
///
/// Implementors push [`StreamEvent`]s into `tx` as they arrive and return once
/// the stream is finished (or an error occurred).
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Stable identifier (matches `ProviderConfig::id`).
    fn id(&self) -> &str;

    /// Declared capabilities.
    fn capabilities(&self) -> ProviderCaps;

    /// Stream a completion for `req`, emitting events on `tx`.
    async fn chat_stream(
        &self,
        req: ChatRequest,
        tx: mpsc::Sender<StreamEvent>,
    ) -> Result<()>;

    /// Lightweight connectivity/credential check.
    async fn test_connection(&self) -> Result<bool>;
}
