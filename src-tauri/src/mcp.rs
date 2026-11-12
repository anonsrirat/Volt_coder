//! Model Context Protocol (MCP) client.
//!
//! Speaks JSON-RPC 2.0 over stdio to external MCP servers, exposing their
//! tools to the agent. Each configured server is spawned lazily and its tool
//! list merged into the tool registry.

use crate::ai::types::ToolSpec;
use crate::config::McpServerConfig;
use crate::error::{AppError, Result};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// A tool advertised by an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, rename = "inputSchema")]
    pub input_schema: Value,
    /// Which server this tool came from.
    #[serde(skip)]
    pub server_id: String,
}

impl McpTool {
    pub fn to_spec(&self) -> ToolSpec {
        ToolSpec {
            // Namespace tools so multiple servers can't collide.
            name: format!("{}__{}", self.server_id, self.name),
            description: self.description.clone(),
            parameters: if self.input_schema.is_null() {
                json!({ "type": "object", "properties": {} })
            } else {
                self.input_schema.clone()
            },
        }
    }
}

/// A live stdio connection to one MCP server.
struct Connection {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl Connection {
    fn spawn(cfg: &McpServerConfig) -> Result<Self> {
        let command = cfg
            .command
            .clone()
            .ok_or_else(|| AppError::Config(format!("mcp server {} has no command", cfg.id)))?;

        let (program, args) = split_command(&command, &cfg.args);

        #[cfg(windows)]
        let mut cmd = {
            // Windows needs the shell to resolve npx/npm shims.
            let mut c = Command::new("cmd");
            c.arg("/C").arg(&program).args(&args);
            c
        };
        #[cfg(not(windows))]
        let mut cmd = {
            let mut c = Command::new(&program);
            c.args(&args);
            c
        };

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| AppError::Other(format!("failed to spawn MCP server {}: {e}", cfg.name)))?;

        let stdin = child.stdin.take().ok_or_else(|| AppError::Other("no stdin".into()))?;
        let stdout = child.stdout.take().ok_or_else(|| AppError::Other("no stdout".into()))?;

        Ok(Connection {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        })
    }

    fn send(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        let line = serde_json::to_string(&request)?;
        writeln!(self.stdin, "{line}")?;
        self.stdin.flush()?;

        // Read until we get a response matching our id (skip notifications).
        loop {
            let mut buf = String::new();
            let n = self.stdout.read_line(&mut buf)?;
            if n == 0 {
                return Err(AppError::Provider("MCP server closed the stream".into()));
            }
            let trimmed = buf.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Ok(msg) = serde_json::from_str::<Value>(trimmed) else {
                continue;
            };
            if msg.get("id").and_then(|v| v.as_u64()) != Some(id) {
                continue;
            }
            if let Some(err) = msg.get("error") {
                return Err(AppError::Provider(format!("MCP error: {err}")));
            }
            return Ok(msg.get("result").cloned().unwrap_or(Value::Null));
        }
    }

    fn initialize(&mut self) -> Result<()> {
        self.send(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "Volt", "version": env!("CARGO_PKG_VERSION") }
            }),
        )?;
        // Notification (no response expected).
        let note = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
        writeln!(self.stdin, "{}", serde_json::to_string(&note)?)?;
        self.stdin.flush()?;
        Ok(())
    }

    fn list_tools(&mut self) -> Result<Vec<McpTool>> {
        let result = self.send("tools/list", json!({}))?;
        let tools = result
            .get("tools")
            .and_then(|t| t.as_array())
            .cloned()
            .unwrap_or_default();
        let mut out = vec![];
        for t in tools {
            if let Ok(mut tool) = serde_json::from_value::<McpTool>(t) {
                tool.server_id = String::new(); // filled by caller
                out.push(tool);
            }
        }
        Ok(out)
    }

    fn call_tool(&mut self, name: &str, args: Value) -> Result<String> {
        let result = self.send(
            "tools/call",
            json!({ "name": name, "arguments": args }),
        )?;
        // MCP returns { content: [ {type:"text", text:"..."}, ... ] }
        if let Some(items) = result.get("content").and_then(|c| c.as_array()) {
            let mut text = String::new();
            for item in items {
                if let Some(t) = item.get("text").and_then(|v| v.as_str()) {
                    text.push_str(t);
                    text.push('\n');
                }
            }
            if !text.is_empty() {
                return Ok(text.trim_end().to_string());
            }
        }
        Ok(result.to_string())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

/// Registry of connected MCP servers.
#[derive(Default)]
pub struct McpRegistry {
    connections: Mutex<Vec<(String, Connection)>>,
    tools: Mutex<Vec<McpTool>>,
}

impl McpRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Connect to one server and merge its tools.
    pub fn connect(&self, cfg: &McpServerConfig) -> Result<Vec<McpTool>> {
        let mut conn = Connection::spawn(cfg)?;
        conn.initialize()?;
        let mut tools = conn.list_tools()?;
        for t in &mut tools {
            t.server_id = cfg.id.clone();
        }
        self.tools.lock().extend(tools.clone());
        self.connections.lock().push((cfg.id.clone(), conn));
        tracing::info!("MCP '{}' connected with {} tools", cfg.name, tools.len());
        Ok(tools)
    }

    /// Discover tools from a server without keeping the connection.
    pub fn probe(cfg: &McpServerConfig) -> Result<Vec<McpTool>> {
        let mut conn = Connection::spawn(cfg)?;
        conn.initialize()?;
        let mut tools = conn.list_tools()?;
        for t in &mut tools {
            t.server_id = cfg.id.clone();
        }
        Ok(tools)
    }

    pub fn tools(&self) -> Vec<McpTool> {
        self.tools.lock().clone()
    }

    pub fn specs(&self) -> Vec<ToolSpec> {
        self.tools.lock().iter().map(|t| t.to_spec()).collect()
    }

    pub fn disconnect(&self, server_id: &str) {
        self.connections.lock().retain(|(id, _)| id != server_id);
        self.tools.lock().retain(|t| t.server_id != server_id);
    }

    pub fn clear(&self) {
        self.connections.lock().clear();
        self.tools.lock().clear();
    }

    /// Invoke a namespaced tool (`server_id__tool_name`).
    pub fn call(&self, namespaced: &str, args: Value) -> Result<String> {
        let (server_id, tool_name) = namespaced
            .split_once("__")
            .ok_or_else(|| AppError::NotFound(format!("invalid MCP tool name: {namespaced}")))?;
        let mut conns = self.connections.lock();
        let (_, conn) = conns
            .iter_mut()
            .find(|(id, _)| id == server_id)
            .ok_or_else(|| AppError::NotFound(format!("MCP server {server_id} not connected")))?;
        conn.call_tool(tool_name, args)
    }

    pub fn has_tool(&self, namespaced: &str) -> bool {
        self.tools.lock().iter().any(|t| t.to_spec().name == namespaced)
    }
}

/// Split a command line into (program, extra args) using simple shell rules.
fn split_command(command: &str, extra: &[String]) -> (String, Vec<String>) {
    let parts: Vec<String> = command
        .split_whitespace()
        .map(|s| s.trim_matches('"').to_string())
        .collect();
    let program = parts.first().cloned().unwrap_or_default();
    let mut args: Vec<String> = parts.into_iter().skip(1).collect();
    args.extend(extra.iter().cloned());
    (program, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_spec_is_namespaced() {
        let tool = McpTool {
            name: "read".into(),
            description: "read a file".into(),
            input_schema: json!({ "type": "object" }),
            server_id: "fs".into(),
        };
        assert_eq!(tool.to_spec().name, "fs__read");
    }

    #[test]
    fn command_splitting() {
        let (prog, args) = split_command("npx -y @mcp/fs /tmp", &["--extra".into()]);
        assert_eq!(prog, "npx");
        assert_eq!(args, vec!["-y", "@mcp/fs", "/tmp", "--extra"]);
    }
}
