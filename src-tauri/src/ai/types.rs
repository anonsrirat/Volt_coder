//! Shared value types exchanged between the agent, providers and the UI.

use serde::{Deserialize, Serialize};

/// Conversation role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// A single chat message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    /// Tool invocations requested by the assistant.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    /// Set on `Role::Tool` messages to tie the result back to its request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Optional name (e.g. tool name) for providers that require it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: Role::System, content: content.into(), tool_calls: vec![], tool_call_id: None, name: None }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: Role::User, content: content.into(), tool_calls: vec![], tool_call_id: None, name: None }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: Role::Assistant, content: content.into(), tool_calls: vec![], tool_call_id: None, name: None }
    }
    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            tool_calls: vec![],
            tool_call_id: Some(tool_call_id.into()),
            name: None,
        }
    }
}

/// How much "thinking" budget the model should be given.
///
/// Each provider maps this onto its own knob:
/// - OpenAI-compatible → `reasoning_effort`
/// - Anthropic         → `thinking.budget_tokens`
/// - Gemini            → `thinkingConfig.thinkingBudget`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingLevel {
    /// No extended reasoning (fastest).
    #[default]
    Off,
    /// Let the model decide how much to reason.
    Auto,
    Low,
    Medium,
    High,
}

impl ThinkingLevel {
    /// Parse a loose user-facing string (`"off"`, `"medium"`, ...).
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "auto" | "adaptive" | "default" => Self::Auto,
            "low" | "minimal" => Self::Low,
            "medium" | "med" | "normal" => Self::Medium,
            "high" | "max" | "ultra" => Self::High,
            _ => Self::Off,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Auto => "auto",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    pub fn is_off(self) -> bool {
        self == Self::Off
    }

    /// Explicit token budget for providers that take one (0 = disabled).
    pub fn budget_tokens(self) -> u32 {
        match self {
            Self::Off => 0,
            // `auto` lets the provider pick; -1 is Gemini's sentinel for "dynamic".
            Self::Auto => u32::MAX,
            Self::Low => 1_024,
            Self::Medium => 4_096,
            Self::High => 16_384,
        }
    }

    /// Value for OpenAI's `reasoning_effort` parameter.
    pub fn effort(self) -> Option<&'static str> {
        match self {
            Self::Off => None,
            // `auto` = omit the field entirely so the model defaults apply.
            Self::Auto => Some("auto"),
            Self::Low => Some("low"),
            Self::Medium => Some("medium"),
            Self::High => Some("high"),
        }
    }
}

/// Description of a tool exposed to the model (JSON-schema based).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    /// JSON Schema object describing the parameters.
    pub parameters: serde_json::Value,
}

/// A concrete tool invocation produced by the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Incremental tool-call chunk (models stream arguments piecewise).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartialToolCall {
    pub index: usize,
    pub id: Option<String>,
    pub name: Option<String>,
    pub arguments_delta: String,
}

/// Events emitted while a provider streams a completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// A chunk of assistant text.
    Token { text: String },
    /// Reasoning/thinking text (for models that expose it).
    Reasoning { text: String },
    /// A streaming tool-call delta.
    ToolCallDelta(PartialToolCall),
    /// A fully assembled tool call.
    ToolCallComplete(ToolCall),
    /// Token accounting for the request.
    Usage { prompt: u32, completion: u32 },
    /// The stream finished normally.
    Done,
    /// A fatal error mid-stream.
    Error { message: String },
}
