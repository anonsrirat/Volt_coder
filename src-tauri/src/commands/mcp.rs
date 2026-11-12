//! MCP commands: manage Model Context Protocol server connections.

use crate::config::McpServerConfig;
use crate::error::Result;
use crate::mcp::McpTool;
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn list_mcp_servers(state: State<'_, Arc<AppState>>) -> Result<Vec<McpServerConfig>> {
    Ok(state.config.get().mcp_servers)
}

#[derive(serde::Deserialize)]
pub struct SaveMcpInput {
    #[serde(flatten)]
    pub config: McpServerConfig,
    #[serde(default)]
    pub connect_now: bool,
}

#[tauri::command]
pub fn save_mcp_server(input: SaveMcpInput, state: State<'_, Arc<AppState>>) -> Result<()> {
    let cfg = input.config.clone();
    state.config.update(|s| {
        if let Some(existing) = s.mcp_servers.iter_mut().find(|m| m.id == cfg.id) {
            *existing = cfg.clone();
        } else {
            s.mcp_servers.push(cfg.clone());
        }
    })?;

    if input.connect_now && cfg.enabled {
        state.mcp.disconnect(&cfg.id);
        let _ = state.mcp.connect(&cfg);
    }
    Ok(())
}

#[tauri::command]
pub fn delete_mcp_server(id: String, state: State<'_, Arc<AppState>>) -> Result<()> {
    state.mcp.disconnect(&id);
    state.config.update(|s| s.mcp_servers.retain(|m| m.id != id))?;
    Ok(())
}

#[tauri::command]
pub fn connect_mcp_server(id: String, state: State<'_, Arc<AppState>>) -> Result<Vec<McpTool>> {
    let cfg = state
        .config
        .get()
        .mcp_servers
        .into_iter()
        .find(|m| m.id == id)
        .ok_or_else(|| crate::error::AppError::NotFound(format!("mcp server {id}")))?;
    state.mcp.disconnect(&id);
    state.mcp.connect(&cfg)
}

/// Connect to every enabled server (called on startup).
#[tauri::command]
pub fn connect_all_mcp(state: State<'_, Arc<AppState>>) -> Result<usize> {
    let servers = state.config.get().mcp_servers;
    let mut connected = 0;
    for cfg in servers.into_iter().filter(|m| m.enabled) {
        state.mcp.disconnect(&cfg.id);
        match state.mcp.connect(&cfg) {
            Ok(_) => connected += 1,
            Err(e) => tracing::warn!("MCP '{}' failed: {e}", cfg.name),
        }
    }
    Ok(connected)
}

#[tauri::command]
pub fn disconnect_mcp_server(id: String, state: State<'_, Arc<AppState>>) -> Result<()> {
    state.mcp.disconnect(&id);
    Ok(())
}

#[tauri::command]
pub fn list_mcp_tools(state: State<'_, Arc<AppState>>) -> Result<Vec<McpTool>> {
    Ok(state.mcp.tools())
}
