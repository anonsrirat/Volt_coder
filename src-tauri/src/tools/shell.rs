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
                          Captures stdout and stderr. Times out after 120s unless \
                          `timeout_secs` is set."
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
        let timeout = Duration::from_secs(args["timeout_secs"].as_u64().unwrap_or(120));

        let lowered = command.to_lowercase();
        if BLOCKLIST.iter().any(|b| lowered.contains(b)) {
            return Ok(ToolResult::err(format!("command blocked by safety policy: {command}")));
        }

        let (shell, flag) = if cfg!(windows) { ("cmd", "/C") } else { ("sh", "-c") };
        let mut cmd = Command::new(shell);
        cmd.arg(flag)
            .arg(command)
            .current_dir(&cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // If the future is dropped on timeout, make sure the child dies
            // with it instead of lingering in the background.
            .kill_on_drop(true);

        let child = cmd
            .spawn()
            .map_err(|e| AppError::Other(format!("spawn failed: {e}")))?;

        // `wait_with_output` reads stdout/stderr concurrently with waiting.
        // The previous implementation waited first and only then drained the
        // pipes, which deadlocked (and timed out) whenever a command emitted
        // more than the OS pipe buffer — e.g. `npm test` or `cargo build`.
        let output = match tokio::time::timeout(timeout, child.wait_with_output()).await {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => return Ok(ToolResult::err(format!("process error: {e}"))),
            Err(_) => {
                return Ok(ToolResult::err(format!(
                    "command timed out after {}s (terminated)",
                    timeout.as_secs()
                )));
            }
        };

        let mut combined = String::from_utf8_lossy(&output.stdout).to_string();
        let err_text = String::from_utf8_lossy(&output.stderr);
        if !err_text.trim().is_empty() {
            combined.push_str("\n[stderr]\n");
            combined.push_str(&err_text);
        }
        let is_error = !output.status.success();
        let exit = output.status.code().unwrap_or(-1);
        let text = format!("(exit {exit})\n{combined}");
        Ok(if is_error { ToolResult::err(text) } else { ToolResult::ok(text) })
    }
}
