//! Session commands: list / create / load / delete chat sessions.

use crate::agent::session::Session;
use crate::ai::types::Message;
use crate::error::Result;
use crate::state::AppState;
use serde::Serialize;
use std::sync::Arc;
use tauri::State;

#[derive(Serialize)]
pub struct SessionDetail {
    pub session: Session,
    pub messages: Vec<Message>,
}

#[tauri::command]
pub fn list_sessions(state: State<'_, Arc<AppState>>) -> Result<Vec<Session>> {
    state.sessions.list()
}

#[tauri::command]
pub fn create_session(
    title: Option<String>,
    provider_id: Option<String>,
    state: State<'_, Arc<AppState>>,
) -> Result<Session> {
    let title = title.unwrap_or_else(|| "New chat".to_string());
    state.sessions.create(&title, provider_id.as_deref())
}

#[tauri::command]
pub fn load_session(id: String, state: State<'_, Arc<AppState>>) -> Result<SessionDetail> {
    let session = state
        .sessions
        .list()?
        .into_iter()
        .find(|s| s.id == id)
        .ok_or_else(|| crate::error::AppError::NotFound(format!("session {id}")))?;
    let messages = state.sessions.load_messages(&id)?;
    Ok(SessionDetail { session, messages })
}

#[tauri::command]
pub fn delete_session(id: String, state: State<'_, Arc<AppState>>) -> Result<()> {
    state.sessions.delete(&id)
}
