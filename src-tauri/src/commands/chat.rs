//! Chat commands: drive the agent loop and handle approvals/cancellation.

use crate::agent::AgentLoop;
use crate::agent::tokio_util_lite::CancellationToken;
use crate::error::Result;
use crate::state::AppState;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    session_id: String,
    text: String,
    provider_id: String,
    thinking: Option<String>,
    state: State<'_, Arc<AppState>>,
) -> Result<()> {
    let state: Arc<AppState> = state.inner().clone();
    let settings = state.config.get();
    let provider = crate::agent::build_provider(&state, &provider_id)?;

    let cancel = CancellationToken::new();
    state
        .cancellations
        .write()
        .insert(session_id.clone(), cancel.clone());

    let thinking = thinking
        .as_deref()
        .map(crate::ai::types::ThinkingLevel::parse)
        .unwrap_or_default();

    let agent = AgentLoop {
        app: app.clone(),
        state: state.clone(),
        provider,
        tools: state.tools.clone(),
        approval: settings.approval.clone(),
        thinking,
        max_iterations: settings.max_iterations,
        cancel,
    };

    let result = agent.run(&session_id, &text, &provider_id).await;
    state.cancellations.write().remove(&session_id);
    result
}

#[tauri::command]
pub fn stop_generation(session_id: String, state: State<'_, Arc<AppState>>) -> Result<()> {
    if let Some(token) = state.cancellations.read().get(&session_id) {
        token.cancel();
    }
    Ok(())
}

#[tauri::command]
pub fn respond_approval(
    tool_call_id: String,
    approved: bool,
    state: State<'_, Arc<AppState>>,
) -> Result<()> {
    let sender = state.pending_approvals.write().remove(&tool_call_id);
    if let Some(tx) = sender {
        let _ = tx.send(approved);
        Ok(())
    } else {
        Err(crate::error::AppError::NotFound(format!("approval {tool_call_id}")))
    }
}
