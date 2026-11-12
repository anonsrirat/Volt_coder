//! Provider commands: CRUD + connectivity testing. Keys go to the OS keyring.

use crate::agent::build_provider;
use crate::config::ProviderConfig;
use crate::error::Result;
use crate::state::AppState;
use serde::Serialize;
use std::sync::Arc;
use tauri::State;

/// Provider view for the UI — includes whether a key is stored, never the key.
#[derive(Serialize)]
pub struct ProviderView {
    #[serde(flatten)]
    pub config: ProviderConfig,
    pub has_key: bool,
    pub is_active: bool,
}

#[tauri::command]
pub fn list_providers(state: State<'_, Arc<AppState>>) -> Result<Vec<ProviderView>> {
    let settings = state.config.get();
    Ok(settings
        .providers
        .iter()
        .map(|p| ProviderView {
            config: p.clone(),
            has_key: crate::config::load_secret(&p.id).ok().flatten().is_some(),
            is_active: settings.active_provider.as_deref() == Some(&p.id),
        })
        .collect())
}

#[derive(serde::Deserialize)]
pub struct SaveProviderInput {
    #[serde(flatten)]
    pub config: ProviderConfig,
    /// Optional plaintext key; if present it is written to the keyring.
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub make_active: bool,
}

#[tauri::command]
pub fn save_provider(input: SaveProviderInput, state: State<'_, Arc<AppState>>) -> Result<()> {
    let id = input.config.id.clone();

    if let Some(key) = &input.api_key {
        if !key.is_empty() {
            crate::config::store_secret(&id, key)?;
        }
    }

    let mut cfg = input.config;
    cfg.key_ref = Some(format!("keyring:volt/{id}"));

    state.config.update(|s| {
        if let Some(existing) = s.providers.iter_mut().find(|p| p.id == id) {
            *existing = cfg;
        } else {
            s.providers.push(cfg);
        }
        if input.make_active {
            s.active_provider = Some(id);
        }
    })?;
    Ok(())
}

#[tauri::command]
pub fn delete_provider(id: String, state: State<'_, Arc<AppState>>) -> Result<()> {
    let _ = crate::config::delete_secret(&id);
    state.config.update(|s| {
        s.providers.retain(|p| p.id != id);
        if s.active_provider.as_deref() == Some(&id) {
            s.active_provider = None;
        }
    })?;
    Ok(())
}

#[tauri::command]
pub fn set_active_provider(id: String, state: State<'_, Arc<AppState>>) -> Result<()> {
    state.config.update(|s| s.active_provider = Some(id))?;
    Ok(())
}

#[derive(Serialize)]
pub struct TestResult {
    pub ok: bool,
    pub latency_ms: u128,
    pub message: String,
}

#[tauri::command]
pub async fn test_connection(id: String, state: State<'_, Arc<AppState>>) -> Result<TestResult> {
    let state: Arc<AppState> = state.inner().clone();
    let start = std::time::Instant::now();
    let provider = build_provider(&state, &id)?;
    match provider.test_connection().await {
        Ok(true) => Ok(TestResult {
            ok: true,
            latency_ms: start.elapsed().as_millis(),
            message: "connection successful".into(),
        }),
        Ok(false) => Ok(TestResult {
            ok: false,
            latency_ms: start.elapsed().as_millis(),
            message: "server responded but rejected the request (check model/key)".into(),
        }),
        Err(e) => Ok(TestResult {
            ok: false,
            latency_ms: start.elapsed().as_millis(),
            message: e.to_string(),
        }),
    }
}
