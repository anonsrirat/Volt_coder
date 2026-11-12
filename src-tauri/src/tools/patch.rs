//! apply_patch — apply a unified-ish diff to a file, with approval.

use super::{Tool, ToolResult};
use crate::ai::types::ToolSpec;
use crate::error::{AppError, Result};
use crate::workspace::ProjectContext;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct ApplyPatch;

#[async_trait]
impl Tool for ApplyPatch {
    fn name(&self) -> &str {
        "apply_patch"
    }
    fn requires_approval(&self) -> bool {
        true
    }
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "apply_patch".into(),
            description:
                "Replace an exact block of text in a file (surgical edit). Provide the exact \
                 `search` string currently in the file and the `replace` string. Requires approval."
                    .into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "search": { "type": "string", "description": "Exact text to find." },
                    "replace": { "type": "string", "description": "Replacement text." }
                },
                "required": ["path", "search", "replace"]
            }),
        }
    }
    async fn run(&self, args: Value, ctx: &ProjectContext) -> Result<ToolResult> {
        let rel = args["path"].as_str().ok_or_else(|| AppError::Other("missing 'path'".into()))?;
        let search = args["search"].as_str().ok_or_else(|| AppError::Other("missing 'search'".into()))?;
        let replace = args["replace"].as_str().ok_or_else(|| AppError::Other("missing 'replace'".into()))?;
        let path = ctx.resolve(rel)?;
        let original = tokio::fs::read_to_string(&path).await?;

        if !original.contains(search) {
            return Ok(ToolResult::err(format!("search block not found in {rel}")));
        }
        let updated = original.replacen(search, replace, 1);
        tokio::fs::write(&path, &updated).await?;

        let diff = similar::TextDiff::from_lines(&original, &updated);
        let summary = diff
            .unified_diff()
            .context_radius(3)
            .header(&format!("a/{rel}"), &format!("b/{rel}"))
            .to_string();
        Ok(ToolResult::ok(format!("patched {rel}\n{summary}")))
    }
}
