//! Update checking against GitHub Releases.
//!
//! Queries the public GitHub Releases API, compares the latest tag with the
//! running version, and (on request) downloads + launches the new installer.
//!
//! Configure the repository via the `VOLT_GITHUB_REPO` env var at build time,
//! or fall back to the compiled-in default below.

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// `owner/repo` used to look up releases.
///
/// Overridable at build time with `VOLT_GITHUB_REPO`; otherwise falls back to
/// the project's canonical repository.
pub fn repo() -> String {
    option_env!("VOLT_GITHUB_REPO")
        .unwrap_or("anonsrirat/Volt_coder")
        .to_string()
}

/// The currently running version (from Cargo.toml at compile time).
pub fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub current: String,
    pub latest: String,
    pub notes: String,
    pub published_at: String,
    /// Direct browser/download URL for the installer asset.
    pub download_url: Option<String>,
    pub asset_name: Option<String>,
    pub asset_size: Option<u64>,
    pub html_url: String,
}

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    published_at: String,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    assets: Vec<GhAsset>,
}

#[derive(Debug, Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

/// Compare two dotted versions, returning true when `candidate > current`.
pub fn is_newer(candidate: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u64> {
        v.trim_start_matches('v')
            .split(['.', '-', '+'])
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect()
    };
    let a = parse(candidate);
    let b = parse(current);
    for i in 0..a.len().max(b.len()) {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        if x != y {
            return x > y;
        }
    }
    false
}

/// Check GitHub for a newer release.
pub async fn check() -> Result<UpdateInfo> {
    let repo = repo();
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let client = reqwest::Client::builder()
        .user_agent("Volt-Updater")
        .build()?;

    let resp = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(AppError::NotFound(format!(
            "no releases found for {repo} (publish one first)"
        )));
    }
    if !resp.status().is_success() {
        return Err(AppError::Provider(format!(
            "github returned {}",
            resp.status()
        )));
    }

    let release: GhRelease = resp.json().await?;
    let current = current_version();
    let latest = release.tag_name.trim_start_matches('v').to_string();
    let available = is_newer(&latest, &current);

    // Prefer an NSIS setup exe, then any .exe / .msi asset.
    let asset = release
        .assets
        .iter()
        .find(|a| a.name.contains("setup") && a.name.ends_with(".exe"))
        .or_else(|| release.assets.iter().find(|a| a.name.ends_with(".msi")))
        .or_else(|| release.assets.iter().find(|a| a.name.ends_with(".exe")));

    Ok(UpdateInfo {
        available,
        current,
        latest,
        notes: release.body,
        published_at: release.published_at,
        download_url: asset.map(|a| a.browser_download_url.clone()),
        asset_name: asset.map(|a| a.name.clone()),
        asset_size: asset.map(|a| a.size),
        html_url: release.html_url,
    })
}

/// Download the update asset into the temp dir and return its path.
pub async fn download(url: &str, filename: &str) -> Result<PathBuf> {
    let client = reqwest::Client::builder()
        .user_agent("Volt-Updater")
        .build()?;
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(AppError::Provider(format!(
            "download failed: {}",
            resp.status()
        )));
    }
    let bytes = resp.bytes().await?;
    let dir = std::env::temp_dir().join("volt-updates");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(filename);
    std::fs::write(&path, &bytes)?;
    tracing::info!("downloaded update to {}", path.display());
    Ok(path)
}

/// Launch a downloaded installer in the background.
pub fn launch_installer(path: &PathBuf) -> Result<()> {
    #[cfg(windows)]
    {
        std::process::Command::new(path)
            .spawn()
            .map_err(|e| AppError::Other(format!("failed to launch installer: {e}")))?;
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        return Err(AppError::Other("auto-install is only supported on Windows".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_comparison() {
        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("v0.1.1", "0.1.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.0.9", "0.1.0"));
    }
}
