//! Custom / fully configurable adapter.
//!
//! Any service that speaks the OpenAI `/chat/completions` shape but needs
//! arbitrary headers (or a non-standard auth scheme) can be expressed here.
//! It simply wraps [`OpenAiProvider`] with pre-populated extra headers.

use super::openai::{OpenAiConfig, OpenAiProvider};
use super::provider::{ChatRequest, LlmProvider, ProviderCaps};
use super::types::StreamEvent;
use crate::error::Result;
use async_trait::async_trait;
use std::collections::BTreeMap;
use tokio::sync::mpsc;

pub struct CustomProvider {
    inner: OpenAiProvider,
}

impl CustomProvider {
    pub fn new(mut cfg: OpenAiConfig, headers: BTreeMap<String, String>, use_bearer: bool) -> Self {
        cfg.extra_headers = headers;
        cfg.use_bearer = use_bearer;
        Self { inner: OpenAiProvider::new(cfg) }
    }
}

#[async_trait]
impl LlmProvider for CustomProvider {
    fn id(&self) -> &str {
        self.inner.id()
    }

    fn capabilities(&self) -> ProviderCaps {
        self.inner.capabilities()
    }

    async fn chat_stream(&self, req: ChatRequest, tx: mpsc::Sender<StreamEvent>) -> Result<()> {
        self.inner.chat_stream(req, tx).await
    }

    async fn test_connection(&self) -> Result<bool> {
        self.inner.test_connection().await
    }
}
