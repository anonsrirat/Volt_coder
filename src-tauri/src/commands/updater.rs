//! Updater commands: check GitHub releases, download and install updates.

use crate::error::Result;
use crate::state::AppState;
use crate::updater::{self, UpdateInfo};
use std::sync::Arc;
use tauri::State;

/// Check GitHub for a newer release.
#[tauri::command]
pub async fn check_update(state: State<'_, Arc<AppState>>) -> Result<UpdateInfo> {
    let info = updater::check().await?;
    state.config.update(|s| {
        s.last_update_check = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
    })?;
    Ok(info)
}

/// Download the update installer to a temp path.
#[tauri::command]
pub async fn download_update(url: String, filename: String) -> Result<String> {
    let path = updater::download(&url, &filename).await?;
    Ok(path.to_string_lossy().to_string())
}

/// Launch a previously-downloaded installer.
#[tauri::command]
pub fn install_update(path: String) -> Result<()> {
    updater::launch_installer(&std::path::PathBuf::from(path))
}
