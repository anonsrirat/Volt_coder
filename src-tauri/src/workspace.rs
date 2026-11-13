//! Workspace: the currently open project directory + safe path resolution.

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

/// How deep the initial explorer scan goes.
///
/// Startup used to scan four levels eagerly, which is slow on big projects
/// (node_modules-heavy monorepos, Rust `target` trees, ...). One level is
/// enough to paint the explorer; deeper folders are loaded on demand through
/// `list_dir` when the user expands them.
pub const EAGER_DEPTH: u32 = 1;

/// A node in the project file tree (sent to the UI).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileNode>>,
}

/// Info returned when a project is opened.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub root: String,
    pub name: String,
    pub tree: Vec<FileNode>,
}

/// Holds the open project's root directory.
///
/// `Clone` is cheap and lets tools run against an owned snapshot without holding
/// the shared lock across an `.await` point.
#[derive(Debug, Default, Clone)]
pub struct ProjectContext {
    root: Option<PathBuf>,
}

impl ProjectContext {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn open(&mut self, path: impl Into<PathBuf>) -> Result<ProjectInfo> {
        let info = scan_project(path)?;
        self.root = Some(PathBuf::from(&info.root));
        Ok(info)
    }

    /// Attach a root that has already been canonicalised/scanned.
    pub fn set_root(&mut self, root: PathBuf) {
        self.root = Some(root);
    }

    pub fn root(&self) -> Result<&PathBuf> {
        self.root.as_ref().ok_or(AppError::NoProject)
    }

    /// Non-failing accessor for the root path (used by prompt building).
    pub fn root_path(&self) -> Option<&PathBuf> {
        self.root.as_ref()
    }

    /// Re-scan the project root and return a fresh file tree.
    ///
    /// Used to restore the explorer on launch without re-running `open` (which
    /// would emit a whole new `ProjectInfo` round-trip).
    pub fn tree(&self) -> Result<Vec<FileNode>> {
        let root = self.root()?;
        scan_dir(root, 0, EAGER_DEPTH)
    }

    /// Resolve a path and guarantee it stays inside the workspace root.
    pub fn resolve(&self, relative: &str) -> Result<PathBuf> {
        let root = self.root()?;
        let candidate = if Path::new(relative).is_absolute() {
            PathBuf::from(relative)
        } else {
            root.join(relative)
        };
        let normalized = normalize(&candidate);
        if !normalized.starts_with(root) {
            return Err(AppError::PathEscape(relative.to_string()));
        }
        Ok(normalized)
    }
}

/// Canonicalise a directory and scan it with the default eager depth.
pub fn scan_project(path: impl Into<PathBuf>) -> Result<ProjectInfo> {
    let root = path.into().canonicalize()?;
    if !root.is_dir() {
        return Err(AppError::NotFound(format!("{} is not a directory", root.display())));
    }
    let name = root
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| root.display().to_string());
    let tree = scan_dir(&root, 0, EAGER_DEPTH)?;
    Ok(ProjectInfo { root: root.to_string_lossy().to_string(), name, tree })
}

/// Scan an already-resolved directory to `max_depth`.
pub fn scan_tree(dir: &Path, max_depth: u32) -> Result<Vec<FileNode>> {
    scan_dir(dir, 0, max_depth)
}

/// Collapse `.` and `..` without touching the filesystem (works pre-creation).
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Recursively scan a directory up to `max_depth`, skipping noise folders.
fn scan_dir(dir: &Path, depth: u32, max_depth: u32) -> Result<Vec<FileNode>> {
    let mut nodes = vec![];
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| {
        let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
        (!is_dir, e.file_name())
    });

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if should_skip(&name) {
            continue;
        }
        let path = entry.path();
        let is_dir = entry.file_type()?.is_dir();
        let rel = path.to_string_lossy().replace('\\', "/");
        let children = if is_dir && depth < max_depth {
            Some(scan_dir(&path, depth + 1, max_depth).unwrap_or_default())
        } else {
            None
        };
        nodes.push(FileNode { name, path: rel, is_dir, children });
    }
    Ok(nodes)
}

fn should_skip(name: &str) -> bool {
    matches!(
        name,
        ".git" | "node_modules" | "target" | "dist" | ".next" | ".idea" | ".vscode"
            | "__pycache__" | ".venv" | "venv" | "build" | ".cache"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_collapses_parent() {
        let p = normalize(Path::new("/a/b/../c"));
        assert_eq!(p, PathBuf::from("/a/c"));
    }

    #[test]
    fn resolve_rejects_escape() {
        let mut ctx = ProjectContext::new();
        let tmp = std::env::temp_dir();
        ctx.root = Some(tmp.clone());
        let res = ctx.resolve("../../etc/passwd");
        assert!(res.is_err());
    }
}
