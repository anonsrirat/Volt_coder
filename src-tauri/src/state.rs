//! Shared application state managed by Tauri (wrapped in `Arc` by the builder).

use crate::agent::session::SessionStore;
use crate::agent::tokio_util_lite::CancellationToken;
use crate::config::ConfigStore;
use crate::mcp::McpRegistry;
use crate::tools::ToolRegistry;
use crate::workspace::ProjectContext;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// A responder used to resolve a pending tool-approval request.
pub type ApprovalSender = tokio::sync::oneshot::Sender<bool>;

pub struct AppState {
    pub config: ConfigStore,
    pub sessions: SessionStore,
    pub context: RwLock<ProjectContext>,
    pub tools: Arc<ToolRegistry>,
    pub mcp: Arc<McpRegistry>,
    /// In-flight generation cancellation tokens, keyed by session id.
    pub cancellations: RwLock<HashMap<String, CancellationToken>>,
    /// Pending approval responses, keyed by tool-call id.
    pub pending_approvals: RwLock<HashMap<String, ApprovalSender>>,
}

impl AppState {
    pub fn new() -> Self {
        let dir = crate::config::config_dir();
        let _ = std::fs::create_dir_all(&dir);
        let config = ConfigStore::load(dir.join("settings.json"));
        let sessions = SessionStore::open(&dir.join("sessions.db"))
            .expect("failed to open sessions database");

        Self {
            config,
            sessions,
            context: RwLock::new(ProjectContext::new()),
            tools: Arc::new(ToolRegistry::default_set()),
            mcp: Arc::new(McpRegistry::new()),
            cancellations: RwLock::new(HashMap::new()),
            pending_approvals: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
