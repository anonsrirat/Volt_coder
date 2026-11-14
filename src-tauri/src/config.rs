//! Configuration: provider registry + OS-keyring secret storage + settings file.

use crate::error::{AppError, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The kind of backend a provider talks to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    OpenAiCompat,
    Anthropic,
    Gemini,
    Custom,
}

/// A persisted provider definition. **Never** contains the API key itself —
/// the key lives in the OS keyring under `key_ref`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub kind: ProviderKind,
    pub label: String,
    pub base_url: String,
    pub model: String,
    /// Reference into the OS keyring, e.g. `keyring:volt/<id>`.
    #[serde(default)]
    pub key_ref: Option<String>,
    /// Extra headers (custom provider).
    #[serde(default)]
    pub headers: std::collections::BTreeMap<String, String>,
    /// Whether to send `Authorization: Bearer <key>`.
    #[serde(default = "default_true")]
    pub use_bearer: bool,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    /// Context window size (tokens) — used by the compaction engine.
    #[serde(default = "default_context_window")]
    pub context_window: u32,
}

fn default_true() -> bool {
    true
}
fn default_context_window() -> u32 {
    128_000
}

/// Whether actions still require explicit user approval.
///
/// `full_access` is a master switch: when it is on, nothing is gated and the
/// agent runs tools end-to-end without a single click. The shell blocklist in
/// `tools::shell` still refuses destructive commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalPolicy {
    /// Run every tool without asking. Defaults to **on** so a fresh install
    /// behaves like a full-access coding agent.
    #[serde(default = "default_true")]
    pub full_access: bool,
    #[serde(default = "default_true")]
    pub write_file: bool,
    #[serde(default = "default_true")]
    pub run_command: bool,
    #[serde(default)]
    pub read_file: bool,
}

impl Default for ApprovalPolicy {
    fn default() -> Self {
        Self { full_access: true, write_file: true, run_command: true, read_file: false }
    }
}

impl ApprovalPolicy {
    /// Apply this policy to a concrete tool name.
    ///
    /// Full access short-circuits everything. Otherwise known built-in tools
    /// are gated by their matching flag; MCP and any unrecognised tool fall
    /// back to the tool's own declaration (MCP tools always return `true`).
    pub fn needs_approval(&self, tool_name: &str, tool: Option<&dyn crate::tools::Tool>) -> bool {
        if self.full_access {
            return false;
        }
        match tool_name {
            "write_file" | "apply_patch" => self.write_file,
            "run_command" => self.run_command,
            "read_file" => self.read_file,
            _ => tool.map(|t| t.requires_approval()).unwrap_or(true),
        }
    }
}

#[cfg(test)]
mod approval_tests {
    use super::*;

    #[test]
    fn policy_gates_known_tools() {
        let p = ApprovalPolicy {
            full_access: false,
            write_file: false,
            run_command: true,
            read_file: true,
        };
        assert!(!p.needs_approval("write_file", None));
        assert!(!p.needs_approval("apply_patch", None));
        assert!(p.needs_approval("run_command", None));
        assert!(p.needs_approval("read_file", None));
    }

    #[test]
    fn unknown_tools_default_to_safe() {
        let p = ApprovalPolicy {
            full_access: false,
            ..ApprovalPolicy::default()
        };
        assert!(p.needs_approval("some_mcp__tool", None));
    }

    #[test]
    fn full_access_skips_every_gate() {
        let p = ApprovalPolicy {
            full_access: true,
            write_file: true,
            run_command: true,
            read_file: true,
        };
        for tool in ["write_file", "apply_patch", "run_command", "read_file", "mcp__x"] {
            assert!(!p.needs_approval(tool, None), "{tool} should be ungated");
        }
    }

    #[test]
    fn default_policy_is_full_access() {
        assert!(ApprovalPolicy::default().full_access);
    }
}

/// Context-compaction tuning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    /// Master switch.
    pub enabled: bool,
    /// Compact once estimated usage passes this fraction of the window (0–1).
    pub threshold: f32,
    /// How many recent turns to always keep verbatim.
    pub keep_recent_turns: u32,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self { enabled: true, threshold: 0.75, keep_recent_turns: 6 }
    }
}

/// A configured MCP (Model Context Protocol) server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    /// "stdio" | "sse" | "http"
    #[serde(default = "default_mcp_transport")]
    pub transport: String,
    /// Command for stdio transports, e.g. `npx -y @modelcontextprotocol/server-filesystem`.
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    /// URL for sse/http transports.
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_mcp_transport() -> String {
    "stdio".into()
}

/// An installed plugin / skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Slash command trigger, e.g. "/docker".
    #[serde(default)]
    pub command: Option<String>,
    /// The prompt/instructions injected when the plugin is active.
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Root settings document persisted to `settings.json` in the app config dir.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
    #[serde(default)]
    pub active_provider: Option<String>,
    /// `dark` | `light` | `system`
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Accent hue (HSL hue 0–360) for the brand colour.
    #[serde(default = "default_accent")]
    pub accent: u32,
    /// UI language: `en` | `th`
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub workspace_root: Option<String>,
    #[serde(default)]
    pub approval: ApprovalPolicy,
    #[serde(default)]
    pub compaction: CompactionConfig,
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
    /// When `true`, the transcript lists every read/write/tool action.
    /// Defaults to `false`: actions stay out of the chat and the agent shows a
    /// single summary once the turn finishes.
    #[serde(default)]
    pub show_tool_activity: bool,
    /// Auto-check GitHub for updates on launch.
    #[serde(default = "default_true")]
    pub auto_check_updates: bool,
    /// Last update check timestamp (ms).
    #[serde(default)]
    pub last_update_check: i64,
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
    #[serde(default)]
    pub plugins: Vec<PluginConfig>,
}

fn default_theme() -> String {
    "dark".into()
}
fn default_accent() -> u32 {
    262
}
fn default_language() -> String {
    "en".into()
}
/// The default size of a single "work window": the number of steps Volt runs
/// before it **automatically continues** into a fresh window. Reaching the
/// window is a checkpoint, not the end of the turn — so one command can finish
/// a long task without the user having to type "continue".
///
/// The turn only truly stops when the model reports the task is done, a loop
/// guard fires, the user cancels, or the absolute safety ceiling is hit.
fn default_max_iterations() -> u32 {
    100
}

/// Apply one-time migrations to a loaded settings document.
///
/// Returns `true` when something changed and should be persisted.
fn migrate(settings: &mut Settings) -> bool {
    let mut changed = false;
    // v0.2.3: the agent now auto-continues, so the old small step defaults
    // (25, then 50) no longer make sense as a limit. Bump untouched legacy
    // values once.
    if matches!(settings.max_iterations, 25 | 50) {
        settings.max_iterations = default_max_iterations();
        changed = true;
    }
    // Never allow an absurdly small window (older files could be 5).
    if settings.max_iterations < 10 {
        settings.max_iterations = 10;
        changed = true;
    }
    changed
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            providers: vec![],
            active_provider: None,
            theme: default_theme(),
            accent: default_accent(),
            language: default_language(),
            workspace_root: None,
            approval: ApprovalPolicy::default(),
            compaction: CompactionConfig::default(),
            max_iterations: default_max_iterations(),
            show_tool_activity: false,
            auto_check_updates: true,
            last_update_check: 0,
            mcp_servers: vec![],
            plugins: vec![],
        }
    }
}

/// Thread-safe settings store backed by a JSON file.
pub struct ConfigStore {
    path: PathBuf,
    inner: RwLock<Settings>,
}

impl ConfigStore {
    pub fn load(path: PathBuf) -> Self {
        let mut settings = match std::fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str::<Settings>(&raw).unwrap_or_default(),
            Err(_) => Settings::default(),
        };
        let migrated = migrate(&mut settings);
        let store = Self { path, inner: RwLock::new(settings) };
        if migrated {
            let snapshot = store.get();
            let _ = store.persist(&snapshot);
        }
        store
    }

    pub fn get(&self) -> Settings {
        self.inner.read().clone()
    }

    pub fn update<R>(&self, f: impl FnOnce(&mut Settings) -> R) -> Result<R> {
        let mut guard = self.inner.write();
        let out = f(&mut guard);
        self.persist(&guard)?;
        Ok(out)
    }

    fn persist(&self, settings: &Settings) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let raw = serde_json::to_string_pretty(settings)?;
        std::fs::write(&self.path, raw)?;
        Ok(())
    }
}

// ---------- Secret storage (OS keyring) ----------

const KEYRING_SERVICE: &str = "volt";

/// Store an API key in the OS credential manager.
pub fn store_secret(provider_id: &str, secret: &str) -> Result<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, provider_id)
        .map_err(|e| AppError::Config(format!("keyring init failed: {e}")))?;
    entry
        .set_password(secret)
        .map_err(|e| AppError::Config(format!("keyring write failed: {e}")))?;
    Ok(())
}

/// Fetch an API key from the OS credential manager.
pub fn load_secret(provider_id: &str) -> Result<Option<String>> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, provider_id)
        .map_err(|e| AppError::Config(format!("keyring init failed: {e}")))?;
    match entry.get_password() {
        Ok(p) => Ok(Some(p)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Config(format!("keyring read failed: {e}"))),
    }
}

/// Remove a stored API key.
pub fn delete_secret(provider_id: &str) -> Result<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, provider_id)
        .map_err(|e| AppError::Config(format!("keyring init failed: {e}")))?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Config(format!("keyring delete failed: {e}"))),
    }
}

/// Resolve where settings + database live for this platform.
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Volt")
}

/// Bootstrap the config directory (called from `setup`).
pub fn bootstrap(_app: &tauri::AppHandle) -> Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    tracing::info!("config dir: {}", dir.display());
    Ok(())
}

#[cfg(test)]
mod settings_tests {
    use super::*;

    #[test]
    fn default_window_is_100() {
        assert_eq!(default_max_iterations(), 100);
    }

    #[test]
    fn legacy_defaults_are_migrated() {
        for legacy in [25, 50] {
            let mut s = Settings { max_iterations: legacy, ..Settings::default() };
            assert!(migrate(&mut s), "legacy {legacy} should migrate");
            assert_eq!(s.max_iterations, 100);
        }
    }

    #[test]
    fn explicit_values_are_kept() {
        let mut s = Settings { max_iterations: 120, ..Settings::default() };
        assert!(!migrate(&mut s));
        assert_eq!(s.max_iterations, 120);
    }

    #[test]
    fn tiny_values_get_a_floor() {
        let mut s = Settings { max_iterations: 3, ..Settings::default() };
        assert!(migrate(&mut s));
        assert_eq!(s.max_iterations, 10);
    }
}
