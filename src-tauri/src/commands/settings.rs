//! Settings commands: theme, language, accent, approval & compaction policies.

use crate::config::Settings;
use crate::error::Result;
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> Result<Settings> {
    Ok(state.config.get())
}

#[derive(serde::Deserialize)]
pub struct SettingsPatch {
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub accent: Option<u32>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub max_iterations: Option<u32>,
    #[serde(default)]
    pub show_tool_activity: Option<bool>,
    #[serde(default)]
    pub auto_check_updates: Option<bool>,
    #[serde(default)]
    pub approval: Option<crate::config::ApprovalPolicy>,
    #[serde(default)]
    pub compaction: Option<crate::config::CompactionConfig>,
}

#[tauri::command]
pub fn update_settings(patch: SettingsPatch, state: State<'_, Arc<AppState>>) -> Result<Settings> {
    state.config.update(|s| {
        if let Some(v) = patch.theme {
            s.theme = v;
        }
        if let Some(v) = patch.accent {
            s.accent = v;
        }
        if let Some(v) = patch.language {
            s.language = v;
        }
        if let Some(v) = patch.max_iterations {
            s.max_iterations = v.clamp(10, 300);
        }
        if let Some(v) = patch.show_tool_activity {
            s.show_tool_activity = v;
        }
        if let Some(v) = patch.auto_check_updates {
            s.auto_check_updates = v;
        }
        if let Some(v) = patch.approval {
            s.approval = v;
        }
        if let Some(v) = patch.compaction {
            s.compaction = v;
        }
    })?;
    Ok(state.config.get())
}

#[derive(serde::Serialize)]
pub struct AppInfo {
    pub version: String,
    pub github_repo: String,
    pub os: String,
}

#[derive(serde::Serialize)]
pub struct ContextInfo {
    /// `0` when the active provider has no known context window.
    pub context_window: u32,
    pub used_tokens: u32,
    /// Fraction (0–1) of the window at which compaction kicks in.
    pub threshold: f32,
}

/// Live context-usage estimate for the composer's compaction indicator.
#[tauri::command]
pub fn context_tokens(
    messages: Vec<crate::ai::types::Message>,
    provider_id: Option<String>,
    state: State<'_, Arc<AppState>>,
) -> Result<ContextInfo> {
    let settings = state.config.get();
    let window = provider_id
        .as_deref()
        .and_then(|id| settings.providers.iter().find(|p| p.id == id))
        .map(|p| p.context_window)
        .unwrap_or(0);
    Ok(ContextInfo {
        context_window: window,
        used_tokens: crate::compaction::conversation_tokens(&messages),
        threshold: settings.compaction.threshold,
    })
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        version: crate::updater::current_version(),
        github_repo: crate::updater::repo(),
        os: std::env::consts::OS.to_string(),
    }
}
