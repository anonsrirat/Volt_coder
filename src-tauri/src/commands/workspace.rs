//! Workspace commands: open projects, browse and read files.

use crate::error::{AppError, Result};
use crate::state::AppState;
use crate::workspace::{FileNode, ProjectInfo};
use serde::Serialize;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn open_project(path: String, state: State<'_, Arc<AppState>>) -> Result<ProjectInfo> {
    let mut ctx = state.context.write();
    let info = ctx.open(path)?;
    // Persist the last-opened project.
    state.config.update(|s| s.workspace_root = Some(info.root.clone()))?;
    Ok(info)
}

#[tauri::command]
pub fn current_project(state: State<'_, Arc<AppState>>) -> Result<Option<ProjectInfo>> {
    let ctx = state.context.read();
    let Some(root) = ctx.root_path() else {
        return Ok(None);
    };
    let name = root
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    // Re-scan so the explorer is populated after a restart (the in-memory
    // context is empty until something opens the project again).
    let tree = ctx.tree().unwrap_or_default();
    Ok(Some(ProjectInfo {
        root: root.to_string_lossy().to_string(),
        name,
        tree,
    }))
}

#[tauri::command]
pub fn list_dir(path: String, state: State<'_, Arc<AppState>>) -> Result<Vec<FileNode>> {
    let ctx = state.context.read();
    let dir = ctx.resolve(&path)?;
    let mut nodes = vec![];
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type()?.is_dir();
        nodes.push(FileNode {
            name,
            path: entry.path().to_string_lossy().replace('\\', "/"),
            is_dir,
            children: None,
        });
    }
    nodes.sort_by_key(|n| (!n.is_dir, n.name.clone()));
    Ok(nodes)
}

#[derive(Serialize)]
pub struct FileContent {
    pub path: String,
    pub content: String,
    pub language: String,
}

#[tauri::command]
pub fn read_file(path: String, state: State<'_, Arc<AppState>>) -> Result<FileContent> {
    let ctx = state.context.read();
    let resolved = ctx.resolve(&path)?;
    let content = std::fs::read_to_string(&resolved)
        .map_err(|_| AppError::NotFound(path.clone()))?;
    let language = guess_language(&path);
    Ok(FileContent { path, content, language })
}

fn guess_language(path: &str) -> String {
    match path.rsplit('.').next().unwrap_or("") {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" => "python",
        "go" => "go",
        "json" => "json",
        "toml" => "toml",
        "md" => "markdown",
        "html" => "html",
        "css" => "css",
        "sql" => "sql",
        "sh" => "bash",
        other => other,
    }
    .to_string()
}
