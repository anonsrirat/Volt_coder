//! run_command — execute a shell command scoped to the project root.

use super::{Tool, ToolResult};
use crate::ai::types::ToolSpec;
use crate::error::{AppError, Result};
use crate::workspace::ProjectContext;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

/// Commands that are never allowed, even with approval, as a safety net.
const BLOCKLIST: &[&str] = &["rm -rf /", "format ", "del /f /s /q c:\\", "shutdown", "mkfs"];

pub struct RunCommand;

#[async_trait]
impl Tool for RunCommand {
    fn name(&self) -> &str {
        "run_command"
    }
    fn requires_approval(&self) -> bool {
        true
    }
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "run_command".into(),
            description: "Run a shell command with the project root as the working directory. \
                          Requires approval. Times out after 60s."
                .into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "The command line to execute." },
                    "timeout_secs": { "type": "integer", "description": "Optional timeout override." }
                },
                "required": ["command"]
            }),
        }
    }
    async fn run(&self, args: Value, ctx: &ProjectContext) -> Result<ToolResult> {
        let command = args["command"]
            .as_str()
            .ok_or_else(|| AppError::Other("missing 'command'".into()))?;
        let cwd = ctx.root()?.clone();
        let timeout = Duration::from_secs(args["timeout_secs"].as_u64().unwrap_or(60));

        let lowered = command.to_lowercase();
        if BLOCKLIST.iter().any(|b| lowered.contains(b)) {
            return Ok(ToolResult::err(format!("command blocked by safety policy: {command}")));
        }

        let (shell, flag) = if cfg!(windows) { ("cmd", "/C") } else { ("sh", "-c") };
        let mut child = Command::new(shell)
            .arg(flag)
            .arg(command)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::Other(format!("spawn failed: {e}")))?;

        // Take the pipes so we can read them after the process exits, while
        // keeping `child` available for the timeout/kill path.
        let mut stdout = child.stdout.take();
        let mut stderr = child.stderr.take();

        let status = match tokio::time::timeout(timeout, child.wait()).await {
            Ok(Ok(status)) => status,
            Ok(Err(e)) => return Ok(ToolResult::err(format!("process error: {e}"))),
            Err(_) => {
                let _ = child.kill().await;
                return Ok(ToolResult::err(format!("command timed out after {}s", timeout.as_secs())));
            }
        };

        use tokio::io::AsyncReadExt;
        let mut out_buf = Vec::new();
        let mut err_buf = Vec::new();
        if let Some(so) = stdout.as_mut() {
            so.read_to_end(&mut out_buf).await.ok();
        }
        if let Some(se) = stderr.as_mut() {
            se.read_to_end(&mut err_buf).await.ok();
        }

        let mut combined = String::new();
        combined.push_str(&String::from_utf8_lossy(&out_buf));
        let err_text = String::from_utf8_lossy(&err_buf);
        if !err_text.trim().is_empty() {
            combined.push_str("\n[stderr]\n");
            combined.push_str(&err_text);
        }
        let is_error = !status.success();
        let exit = status.code().unwrap_or(-1);
        let text = format!("(exit {exit})\n{combined}");
        Ok(if is_error { ToolResult::err(text) } else { ToolResult::ok(text) })
    }
}
