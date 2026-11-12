//! Plan / canvas commands.
//!
//! Exposes the agent's execution plan (phases + steps) so the UI can render a
//! live canvas. The plan lives in memory for the active session and is emitted
//! as `plan://updated` whenever it changes.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    /// `pending` | `in_progress` | `completed`
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub title: String,
    #[serde(default)]
    pub summary: String,
    pub steps: Vec<PlanStep>,
}

/// Simple in-memory plan store (one plan per session).
#[derive(Default)]
pub struct PlanStore {
    plans: RwLock<std::collections::HashMap<String, Plan>>,
}

impl PlanStore {
    pub fn set(&self, session_id: &str, plan: Plan) {
        self.plans.write().unwrap().insert(session_id.to_string(), plan);
    }
    pub fn get(&self, session_id: &str) -> Option<Plan> {
        self.plans.read().unwrap().get(session_id).cloned()
    }
}

#[tauri::command]
pub fn get_plan(session_id: String, store: State<'_, PlanStore>) -> Result<Option<Plan>> {
    Ok(store.get(&session_id))
}

#[tauri::command]
pub fn set_plan(
    app: AppHandle,
    session_id: String,
    plan: Plan,
    store: State<'_, PlanStore>,
) -> Result<()> {
    store.set(&session_id, plan.clone());
    let _ = app.emit("plan://updated", serde_json::json!({ "session_id": session_id, "plan": plan }));
    Ok(())
}

/// Update a single step's status (used while streaming progress).
#[tauri::command]
pub fn update_plan_step(
    app: AppHandle,
    session_id: String,
    step_id: String,
    status: String,
    store: State<'_, PlanStore>,
) -> Result<()> {
    if let Some(mut plan) = store.get(&session_id) {
        if let Some(step) = plan.steps.iter_mut().find(|s| s.id == step_id) {
            step.status = status;
        }
        store.set(&session_id, plan.clone());
        let _ = app.emit("plan://updated", serde_json::json!({ "session_id": session_id, "plan": plan }));
    }
    Ok(())
}
