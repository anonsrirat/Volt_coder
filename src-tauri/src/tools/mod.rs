//! Tool registry and built-in tools the agent can invoke.

pub mod fs;
pub mod patch;
pub mod search;
pub mod shell;

use crate::ai::types::ToolSpec;
use crate::error::Result;
use crate::workspace::ProjectContext;
use async_trait::async_trait;
use serde_json::Value;

/// Result of running a tool.
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub content: String,
    pub is_error: bool,
}

impl ToolResult {
    pub fn ok(content: impl Into<String>) -> Self {
        Self { content: content.into(), is_error: false }
    }
    pub fn err(content: impl Into<String>) -> Self {
        Self { content: content.into(), is_error: true }
    }
}

/// The tool contract.
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn spec(&self) -> ToolSpec;
    /// Whether the user must approve before this tool runs.
    fn requires_approval(&self) -> bool;
    async fn run(&self, args: Value, ctx: &ProjectContext) -> Result<ToolResult>;
}

/// Holds every registered tool.
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Build the default registry with all built-in tools.
    pub fn default_set() -> Self {
        let tools: Vec<Box<dyn Tool>> = vec![
            Box::new(fs::ReadFile),
            Box::new(fs::WriteFile),
            Box::new(fs::ListDir),
            Box::new(patch::ApplyPatch),
            Box::new(search::Grep),
            Box::new(shell::RunCommand),
        ];
        Self { tools }
    }

    /// Tool specs formatted for the provider.
    pub fn specs(&self) -> Vec<ToolSpec> {
        self.tools.iter().map(|t| t.spec()).collect()
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.iter().find(|t| t.name() == name).map(|t| t.as_ref())
    }

    pub fn names(&self) -> Vec<&str> {
        self.tools.iter().map(|t| t.name()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::default_set()
    }
}
