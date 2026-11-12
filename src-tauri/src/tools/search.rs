//! grep-like search across the project (gitignore-aware).

use super::{Tool, ToolResult};
use crate::ai::types::ToolSpec;
use crate::error::{AppError, Result};
use crate::workspace::ProjectContext;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::io::BufRead;

pub struct Grep;

#[async_trait]
impl Tool for Grep {
    fn name(&self) -> &str {
        "grep"
    }
    fn requires_approval(&self) -> bool {
        false
    }
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "grep".into(),
            description: "Search for a plain-text pattern across project files. Respects .gitignore."
                .into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "Text to search for." },
                    "max_results": { "type": "integer", "description": "Cap on matches (default 100)." }
                },
                "required": ["pattern"]
            }),
        }
    }
    async fn run(&self, args: Value, ctx: &ProjectContext) -> Result<ToolResult> {
        let pattern = args["pattern"]
            .as_str()
            .ok_or_else(|| AppError::Other("missing 'pattern'".into()))?
            .to_string();
        let max = args["max_results"].as_u64().unwrap_or(100) as usize;
        let root = ctx.root()?.clone();

        let result = tokio::task::spawn_blocking(move || search_sync(&root, &pattern, max))
            .await
            .map_err(|e| AppError::Other(e.to_string()))??;
        Ok(ToolResult::ok(result))
    }
}

fn search_sync(root: &std::path::Path, pattern: &str, max: usize) -> Result<String> {
    let mut hits = vec![];
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .build();

    'outer: for entry in walker.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Ok(file) = std::fs::File::open(path) else { continue };
        let reader = std::io::BufReader::new(file);
        for (lineno, line) in reader.lines().enumerate() {
            let Ok(line) = line else { continue };
            if line.contains(pattern) {
                let rel = path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/");
                hits.push(format!("{rel}:{}: {}", lineno + 1, line.trim()));
                if hits.len() >= max {
                    break 'outer;
                }
            }
        }
    }

    if hits.is_empty() {
        Ok("no matches found".into())
    } else {
        Ok(hits.join("\n"))
    }
}
