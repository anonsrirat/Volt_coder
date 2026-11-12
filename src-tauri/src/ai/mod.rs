//! AI layer: message types, provider abstraction and concrete adapters.

pub mod anthropic;
pub mod custom;
pub mod gemini;
pub mod openai;
pub mod provider;
pub mod sse;
pub mod types;

pub use provider::{ChatRequest, LlmProvider, ProviderCaps};
pub use types::*;
