//! Filesystem tools: read_file, write_file, list_dir.

use super::{Tool, ToolResult};
use crate::ai::types::ToolSpec;
use crate::error::{AppError, Result};
use crate::workspace::ProjectContext;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct ReadFile;

#[async_trait]
impl Tool for ReadFile {
    fn name(&self) -> &str {
        "read_file"
    }
    fn requires_approval(&self) -> bool {
        false
    }
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "read_file".into(),
            description: "Read the full text content of a file inside the open project.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path relative to the project root." }
                },
                "required": ["path"]
            }),
        }
    }
    async fn run(&self, args: Value, ctx: &ProjectContext) -> Result<ToolResult> {
        let rel = args["path"].as_str().ok_or_else(|| AppError::Other("missing 'path'".into()))?;
        let path = ctx.resolve(rel)?;
        if !path.exists() {
            return Ok(ToolResult::err(format!("file not found: {rel}")));
        }
        let content = tokio::fs::read_to_string(&path).await?;
        Ok(ToolResult::ok(content))
    }
}

pub struct WriteFile;

#[async_trait]
impl Tool for WriteFile {
    fn name(&self) -> &str {
        "write_file"
    }
    fn requires_approval(&self) -> bool {
        true
    }
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "write_file".into(),
            description: "Create or overwrite a file with the given content. Requires user approval.".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path relative to the project root." },
                    "content": { "type": "string", "description": "Full file content to write." }
                },
                "required": ["path", "content"]
            }),
        }
    }
    async fn run(&self, args: Value, ctx: &ProjectContext) -> Result<ToolResult> {
        let rel = args["path"].as_str().ok_or_else(|| AppError::Other("missing 'path'".into()))?;
        let content = args["content"].as_str().ok_or_else(|| AppError::Other("missing 'content'".into()))?;
        let path = ctx.resolve(rel)?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&path, content).await?;
        Ok(ToolResult::ok(format!("wrote {} bytes to {rel}", content.len())))
    }
}

pub struct ListDir;

#[async_trait]
impl Tool for ListDir {
    fn name(&self) -> &str {
        "list_dir"
    }
    fn requires_approval(&self) -> bool {
        false
    }
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "list_dir".into(),
            description: "List entries in a directory (defaults to the project root).".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Directory path relative to the root (default '.')." }
                }
            }),
        }
    }
    async fn run(&self, args: Value, ctx: &ProjectContext) -> Result<ToolResult> {
        let rel = args["path"].as_str().unwrap_or(".");
        let path = ctx.resolve(rel)?;
        let mut entries = vec![];
        let mut rd = tokio::fs::read_dir(&path).await?;
        while let Some(entry) = rd.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            let suffix = if entry.file_type().await?.is_dir() { "/" } else { "" };
            entries.push(format!("{name}{suffix}"));
        }
        entries.sort();
        Ok(ToolResult::ok(entries.join("\n")))
    }
}
