//! Plugin / skill commands: CRUD for prompt-injected skills.

use crate::config::PluginConfig;
use crate::error::Result;
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn list_plugins(state: State<'_, Arc<AppState>>) -> Result<Vec<PluginConfig>> {
    Ok(state.config.get().plugins)
}

#[tauri::command]
pub fn save_plugin(plugin: PluginConfig, state: State<'_, Arc<AppState>>) -> Result<()> {
    let id = plugin.id.clone();
    state.config.update(|s| {
        if let Some(existing) = s.plugins.iter_mut().find(|p| p.id == id) {
            *existing = plugin.clone();
        } else {
            s.plugins.push(plugin.clone());
        }
    })?;
    Ok(())
}

#[tauri::command]
pub fn toggle_plugin(id: String, enabled: bool, state: State<'_, Arc<AppState>>) -> Result<()> {
    state.config.update(|s| {
        if let Some(p) = s.plugins.iter_mut().find(|p| p.id == id) {
            p.enabled = enabled;
        }
    })?;
    Ok(())
}

#[tauri::command]
pub fn delete_plugin(id: String, state: State<'_, Arc<AppState>>) -> Result<()> {
    state.config.update(|s| s.plugins.retain(|p| p.id != id))?;
    Ok(())
}
