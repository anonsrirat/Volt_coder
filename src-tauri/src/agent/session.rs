//! Session persistence backed by SQLite.
//!
//! Stores chat sessions and their messages so conversations survive restarts.

use crate::ai::types::{Message, Role};
use crate::error::Result;
use parking_lot::Mutex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A chat session row (metadata only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub provider_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Thread-safe handle to the sessions database.
pub struct SessionStore {
    conn: Mutex<Connection>,
}

impl SessionStore {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS sessions (
                id          TEXT PRIMARY KEY,
                title       TEXT NOT NULL,
                provider_id TEXT,
                created_at  INTEGER NOT NULL,
                updated_at  INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS messages (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                role       TEXT NOT NULL,
                content    TEXT NOT NULL,
                tool_calls TEXT,
                tool_call_id TEXT,
                name       TEXT,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id);
            "#,
        )?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn create(&self, title: &str, provider_id: Option<&str>) -> Result<Session> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = now_ms();
        self.conn.lock().execute(
            "INSERT INTO sessions (id, title, provider_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
            rusqlite::params![id, title, provider_id, now],
        )?;
        Ok(Session { id, title: title.into(), provider_id: provider_id.map(String::from), created_at: now, updated_at: now })
    }

    pub fn list(&self) -> Result<Vec<Session>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, title, provider_id, created_at, updated_at FROM sessions ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Session {
                id: r.get(0)?,
                title: r.get(1)?,
                provider_id: r.get(2)?,
                created_at: r.get(3)?,
                updated_at: r.get(4)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn delete(&self, session_id: &str) -> Result<()> {
        self.conn.lock().execute("DELETE FROM sessions WHERE id = ?1", [session_id])?;
        Ok(())
    }

    pub fn rename(&self, session_id: &str, title: &str) -> Result<()> {
        self.conn.lock().execute(
            "UPDATE sessions SET title = ?2, updated_at = ?3 WHERE id = ?1",
            rusqlite::params![session_id, title, now_ms()],
        )?;
        Ok(())
    }

    pub fn append_message(&self, session_id: &str, msg: &Message) -> Result<()> {
        let role = match msg.role {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
        };
        let tool_calls = if msg.tool_calls.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&msg.tool_calls)?)
        };
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO messages (session_id, role, content, tool_calls, tool_call_id, name, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                session_id,
                role,
                msg.content,
                tool_calls,
                msg.tool_call_id,
                msg.name,
                now_ms()
            ],
        )?;
        conn.execute("UPDATE sessions SET updated_at = ?2 WHERE id = ?1", rusqlite::params![session_id, now_ms()])?;
        Ok(())
    }

    pub fn load_messages(&self, session_id: &str) -> Result<Vec<Message>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT role, content, tool_calls, tool_call_id, name FROM messages
             WHERE session_id = ?1 ORDER BY id ASC",
        )?;
        let rows = stmt.query_map([session_id], |r| {
            let role: String = r.get(0)?;
            let content: String = r.get(1)?;
            let tool_calls: Option<String> = r.get(2)?;
            let tool_call_id: Option<String> = r.get(3)?;
            let name: Option<String> = r.get(4)?;
            Ok((role, content, tool_calls, tool_call_id, name))
        })?;

        let mut out = vec![];
        for row in rows.flatten() {
            let (role, content, tool_calls, tool_call_id, name) = row;
            let role = match role.as_str() {
                "system" => Role::System,
                "user" => Role::User,
                "assistant" => Role::Assistant,
                _ => Role::Tool,
            };
            let calls = tool_calls
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            out.push(Message { role, content, tool_calls: calls, tool_call_id, name });
        }
        Ok(out)
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
