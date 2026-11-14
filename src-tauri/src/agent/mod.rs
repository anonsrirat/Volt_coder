//! Agent layer: the ReAct loop, session persistence and prompt construction.

pub mod prompt;
pub mod session;

use crate::ai::provider::{ChatRequest, LlmProvider};
use crate::ai::types::{Message, PartialToolCall, Role, StreamEvent, ToolCall};
use crate::config::{ApprovalPolicy, ProviderKind};
use crate::error::{AppError, Result};
use crate::state::AppState;
use crate::tools::{ToolRegistry, ToolResult};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tokio_util_lite::CancellationToken;

/// Minimal cancellation primitive (avoids pulling a whole extra crate in).
pub mod tokio_util_lite {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[derive(Clone, Default)]
    pub struct CancellationToken(Arc<AtomicBool>);
    impl CancellationToken {
        pub fn new() -> Self {
            Self(Arc::new(AtomicBool::new(false)))
        }
        pub fn cancel(&self) {
            self.0.store(true, Ordering::SeqCst);
        }
        pub fn is_cancelled(&self) -> bool {
            self.0.load(Ordering::SeqCst)
        }
    }
}

/// A request from the UI to start an agent run.
#[derive(Debug, Clone, Deserialize)]
pub struct RunAgentInput {
    pub session_id: String,
    pub text: String,
    pub provider_id: String,
    /// How much reasoning budget to request (`"off" | "auto" | "low" | "medium" | "high"`).
    #[serde(default)]
    pub thinking: Option<String>,
}

/// Build the concrete provider from a stored config + its keyring secret.
pub fn build_provider(state: &AppState, provider_id: &str) -> Result<Arc<dyn LlmProvider>> {
    let settings = state.config.get();
    let cfg = settings
        .providers
        .iter()
        .find(|p| p.id == provider_id)
        .ok_or_else(|| AppError::NotFound(format!("provider {provider_id}")))?;

    let api_key = crate::config::load_secret(provider_id)?.unwrap_or_default();

    let provider: Arc<dyn LlmProvider> = match cfg.kind {
        ProviderKind::OpenAiCompat => Arc::new(crate::ai::openai::OpenAiProvider::new(
            crate::ai::openai::OpenAiConfig {
                id: cfg.id.clone(),
                base_url: cfg.base_url.clone(),
                api_key,
                model: cfg.model.clone(),
                extra_headers: cfg.headers.clone(),
                use_bearer: cfg.use_bearer,
            },
        )),
        ProviderKind::Anthropic => Arc::new(crate::ai::anthropic::AnthropicProvider::new(
            crate::ai::anthropic::AnthropicConfig {
                id: cfg.id.clone(),
                base_url: cfg.base_url.clone(),
                api_key,
                model: cfg.model.clone(),
            },
        )),
        ProviderKind::Gemini => Arc::new(crate::ai::gemini::GeminiProvider::new(
            crate::ai::gemini::GeminiConfig {
                id: cfg.id.clone(),
                base_url: cfg.base_url.clone(),
                api_key,
                model: cfg.model.clone(),
            },
        )),
        ProviderKind::Custom => Arc::new(crate::ai::custom::CustomProvider::new(
            crate::ai::openai::OpenAiConfig {
                id: cfg.id.clone(),
                base_url: cfg.base_url.clone(),
                api_key,
                model: cfg.model.clone(),
                extra_headers: cfg.headers.clone(),
                use_bearer: cfg.use_bearer,
            },
            cfg.headers.clone(),
            cfg.use_bearer,
        )),
    };
    Ok(provider)
}

/// The core ReAct agent loop.
pub struct AgentLoop {
    pub app: AppHandle,
    pub state: Arc<AppState>,
    pub provider: Arc<dyn LlmProvider>,
    pub tools: Arc<ToolRegistry>,
    /// Which actions still need an explicit user approval this run.
    pub approval: ApprovalPolicy,
    /// Reasoning budget requested for this run.
    pub thinking: crate::ai::types::ThinkingLevel,
    pub max_iterations: u32,
    pub cancel: CancellationToken,
}

/// Absolute safety ceiling for a single turn.
///
/// `max_iterations` is a *work window* that auto-continues; this constant caps
/// the total number of windows so a pathological agent loop cannot run forever.
/// Real tasks finish long before it — it only exists as a runaway guard.
const MAX_WINDOWS: u32 = 8;

impl AgentLoop {
    /// Run one user turn to completion, emitting events to the UI.
    pub async fn run(&self, session_id: &str, user_text: &str, _provider_id: &str) -> Result<()> {
        // 1. Load + persist the user's message.
        let mut messages = self.state.sessions.load_messages(session_id)?;
        messages.push(Message::user(user_text));
        self.state.sessions.append_message(session_id, &messages[messages.len() - 1])?;

        let settings = self.state.config.get();
        let system = crate::agent::prompt::system_prompt(
            self.state.context.read().root_cstr(),
            &settings.plugins,
            self.thinking,
        );
        // Built-in tools + any MCP tools currently connected.
        let mut tools = self.tools.specs();
        tools.extend(self.state.mcp.specs());

        // Context compaction — fold older turns before we hit the window limit.
        if let Some(provider_cfg) = settings.providers.iter().find(|p| p.id == _provider_id) {
            if crate::compaction::should_compact(&messages, &settings.compaction, provider_cfg.context_window) {
                let outcome = crate::compaction::compact(&messages, &settings.compaction);
                if outcome.summarised > 0 {
                    tracing::info!("compacted {} messages into a summary", outcome.summarised);
                    self.emit("ai://compacted", serde_json::json!({
                        "session_id": session_id,
                        "summarised": outcome.summarised,
                        "tokens_before": crate::compaction::conversation_tokens(&messages),
                    }));
                    messages = outcome.messages;
                }
            }
        }

        let mut iteration = 0;
        // Steps used in the current work window, and how many windows we have
        // already auto-continued through.
        let mut window_steps = 0u32;
        let mut windows = 0u32;
        // Guard against a model that keeps replaying the exact same tool call.
        let mut last_signature: Option<String> = None;
        let mut repeats = 0u32;
        // Count identical calls across the whole turn: a model that alternates
        // between the same few failing actions is still a loop.
        let mut call_counts: HashMap<String, u32> = HashMap::new();

        loop {
            if self.cancel.is_cancelled() {
                self.emit_phase(session_id, "run", "done", "", "error");
                self.emit(
                    "ai://notice",
                    serde_json::json!({
                        "session_id": session_id,
                        "kind": "cancelled",
                        "message": "stopped",
                    }),
                );
                self.emit("ai://done", serde_json::json!({ "session_id": session_id }));
                return Ok(());
            }

            // Auto-continue: hitting the window budget is a checkpoint, not the
            // end of the turn. The user wants ONE command to finish the job, so
            // we roll into a fresh window instead of stopping and asking for
            // "continue". The turn only ends on completion / loop / cancel.
            if window_steps >= self.max_iterations {
                if windows + 1 >= MAX_WINDOWS {
                    // Absolute safety net for a runaway agent.
                    self.emit_phase(session_id, "run", "done", "", "done");
                    self.emit(
                        "ai://notice",
                        serde_json::json!({
                            "session_id": session_id,
                            "kind": "safety_limit",
                            "message": "reached the hard step limit",
                        }),
                    );
                    self.emit("ai://done", serde_json::json!({ "session_id": session_id }));
                    return Ok(());
                }
                windows += 1;
                window_steps = 0;
                // Leave a short "continuing" row in the Canvas timeline so it is
                // obvious the run is still going (never a stuck spinner).
                let id = format!("auto-{windows}");
                self.emit_phase(session_id, &id, "working", "continuing", "running");
                self.emit_phase(session_id, &id, "working", "continuing", "done");
            }
            window_steps += 1;
            iteration += 1;

            let think_id = format!("think-{iteration}");
            self.emit_phase(session_id, &think_id, "thinking", "", "running");

            // 2. Ask the provider to stream a completion.
            let (tx, mut rx) = mpsc::channel::<StreamEvent>(64);
            let req = ChatRequest {
                messages: messages.clone(),
                tools: tools.clone(),
                model: String::new(),
                temperature: None,
                max_tokens: None,
                system: Some(system.clone()),
                thinking: Some(self.thinking),
            };

            let provider = self.provider.clone();
            let app = self.app.clone();
            let producer = tokio::spawn(async move { provider.chat_stream(req, tx).await });

            let mut assistant_text = String::new();
            let mut completed_calls: Vec<ToolCall> = vec![];
            let mut partials: BTreeMap<usize, PartialToolCall> = BTreeMap::new();

            while let Some(ev) = rx.recv().await {
                match ev {
                    StreamEvent::Token { text } => {
                        assistant_text.push_str(&text);
                        app.emit("ai://token", serde_json::json!({ "session_id": session_id, "text": text })).ok();
                    }
                    StreamEvent::Reasoning { text } => {
                        app.emit("ai://reasoning", serde_json::json!({ "text": text })).ok();
                    }
                    StreamEvent::ToolCallDelta(delta) => {
                        partials.insert(delta.index, delta);
                    }
                    StreamEvent::ToolCallComplete(call) => completed_calls.push(call),
                    StreamEvent::Usage { prompt, completion } => {
                        app.emit("ai://usage", serde_json::json!({ "prompt": prompt, "completion": completion })).ok();
                    }
                    StreamEvent::Error { message } => {
                        app.emit("ai://error", serde_json::json!({ "message": message })).ok();
                    }
                    StreamEvent::Done => {}
                }
            }
            let _ = producer.await;
            self.emit_phase(session_id, &think_id, "thinking", "", "done");

            // 3. Persist the assistant message.
            let mut assistant_msg = Message::assistant(assistant_text.clone());
            assistant_msg.tool_calls = completed_calls.clone();
            messages.push(assistant_msg.clone());
            self.state.sessions.append_message(session_id, &assistant_msg)?;

            if completed_calls.is_empty() {
                // No more tools — we're done.
                self.emit_phase(session_id, "run", "done", "", "done");
                self.emit("ai://done", serde_json::json!({ "session_id": session_id }));
                break;
            }

            // Duplicate-call guard: if the model asks for the exact same batch
            // three times in a row, stop instead of burning iterations.
            let signature = completed_calls
                .iter()
                .map(|c| format!("{}:{}", c.name, c.arguments))
                .collect::<Vec<_>>()
                .join("|");
            if last_signature.as_deref() == Some(signature.as_str()) {
                repeats += 1;
            } else {
                repeats = 0;
                last_signature = Some(signature);
            }
            if repeats >= 2 {
                self.emit_phase(session_id, "run", "done", "", "done");
                self.emit(
                    "ai://notice",
                    serde_json::json!({
                        "session_id": session_id,
                        "kind": "loop_guard",
                        "message": "the same action repeated",
                    }),
                );
                self.emit("ai://done", serde_json::json!({ "session_id": session_id }));
                break;
            }

            // 4. Execute each requested tool (with approval if required).
            let mut loop_detected = false;
            for call in &completed_calls {
                let key = format!("{}:{}", call.name, call.arguments);
                let count = call_counts.entry(key).or_insert(0);
                *count += 1;
                if *count > 4 {
                    loop_detected = true;
                    break;
                }

                let (phase, detail) = describe_tool(&call.name, &call.arguments);
                let phase_id = format!("{iteration}-{}", call.id);
                self.emit_phase(session_id, &phase_id, phase, &detail, "running");

                let result = self.execute_tool(session_id, call).await?;

                self.emit_phase(
                    session_id,
                    &phase_id,
                    phase,
                    &detail,
                    if result.is_error { "error" } else { "done" },
                );

                let tool_msg = Message {
                    role: Role::Tool,
                    content: result.content,
                    tool_calls: vec![],
                    tool_call_id: Some(call.id.clone()),
                    name: Some(call.name.clone()),
                };
                messages.push(tool_msg.clone());
                self.state.sessions.append_message(session_id, &tool_msg)?;
            }

            if loop_detected {
                self.emit_phase(session_id, "run", "done", "", "done");
                self.emit(
                    "ai://notice",
                    serde_json::json!({
                        "session_id": session_id,
                        "kind": "loop_guard",
                        "message": "the same action repeated",
                    }),
                );
                self.emit("ai://done", serde_json::json!({ "session_id": session_id }));
                break;
            }
            // loop back → send tool results to the model.
        }
        Ok(())
    }

    async fn execute_tool(&self, session_id: &str, call: &ToolCall) -> Result<ToolResult> {
        // MCP tools are namespaced (`server_id__tool`) and always require approval.
        let is_mcp = self.state.mcp.has_tool(&call.name);
        let tool = self.tools.get(&call.name);

        if tool.is_none() && !is_mcp {
            return Ok(ToolResult::err(format!("unknown tool: {}", call.name)));
        }

        self.emit(
            "ai://tool-call",
            serde_json::json!({
                "session_id": session_id,
                "id": call.id,
                "name": call.name,
                "arguments": call.arguments,
            }),
        );

        // Approval gate — driven by the user's policy in Settings, falling back
        // to the tool's own default so unknown/MCP tools stay safe.
        let needs_approval = if self.approval.full_access {
            false
        } else {
            is_mcp || self.approval.needs_approval(&call.name, tool)
        };
        if needs_approval {
            let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
            self.state
                .pending_approvals
                .write()
                .insert(call.id.clone(), resp_tx);

            let preview = build_preview(&call.name, &call.arguments);
            self.emit(
                "ai://approval-request",
                serde_json::json!({
                    "session_id": session_id,
                    "tool_call_id": call.id,
                    "tool_name": call.name,
                    "arguments": call.arguments,
                    "preview": preview,
                }),
            );

            let approved = match tokio::time::timeout(std::time::Duration::from_secs(300), resp_rx).await {
                Ok(Ok(v)) => v,
                _ => false,
            };
            if !approved {
                let res = ToolResult::err("user denied this action");
                self.emit(
                    "ai://tool-result",
                    serde_json::json!({ "id": call.id, "name": call.name, "result": res.content, "is_error": true }),
                );
                return Ok(res);
            }
        }

        // Take an owned snapshot so we never hold the lock across `.await`.
        let result = if is_mcp {
            match self.state.mcp.call(&call.name, call.arguments.clone()) {
                Ok(text) => ToolResult::ok(text),
                Err(e) => ToolResult::err(e.to_string()),
            }
        } else {
            let ctx = self.state.context.read().clone();
            let tool = tool.expect("checked above");
            match tool.run(call.arguments.clone(), &ctx).await {
                Ok(r) => r,
                Err(e) => ToolResult::err(e.to_string()),
            }
        };

        self.emit(
            "ai://tool-result",
            serde_json::json!({
                "id": call.id,
                "name": call.name,
                "result": result.content,
                "is_error": result.is_error,
            }),
        );
        Ok(result)
    }

    fn emit(&self, event: &str, payload: serde_json::Value) {
        let _ = self.app.emit(event, payload);
    }

    /// Publish one slice of live progress for the Canvas timeline.
    ///
    /// `id` is stable per action so the UI can upsert (running → done/error)
    /// instead of appending duplicates.
    fn emit_phase(&self, session_id: &str, id: &str, phase: &str, detail: &str, status: &str) {
        self.emit(
            "ai://phase",
            serde_json::json!({
                "session_id": session_id,
                "id": id,
                "phase": phase,
                "detail": detail,
                "status": status,
            }),
        );
    }
}

/// Map a tool call onto a coarse phase + a short human-readable detail.
fn describe_tool(name: &str, args: &serde_json::Value) -> (&'static str, String) {
    let field = |key: &str| args[key].as_str().unwrap_or_default().to_string();
    let clamp = |s: String| {
        if s.chars().count() > 64 {
            format!("{}...", s.chars().take(64).collect::<String>())
        } else {
            s
        }
    };
    match name {
        "read_file" | "list_dir" => ("reading", clamp(field("path"))),
        "write_file" | "apply_patch" => ("editing", clamp(field("path"))),
        "run_command" => ("running", clamp(field("command"))),
        "grep" => ("searching", clamp(field("pattern"))),
        _ => ("working", clamp(name.to_string())),
    }
}

/// Build a human-readable preview shown in the approval prompt.
///
/// - `write_file`  → a numbered preview of the first lines of the new file
/// - `apply_patch` → a real unified diff of the change
/// - `run_command` → the command line
fn build_preview(tool_name: &str, args: &serde_json::Value) -> String {
    match tool_name {
        "write_file" => {
            let rel = args["path"].as_str().unwrap_or("(unknown)");
            let content = args["content"].as_str().unwrap_or("");
            let total = content.lines().count();
            let mut out = format!("Create/overwrite {rel} ({total} lines)\n");
            for (i, line) in content.lines().take(40).enumerate() {
                out.push_str(&format!("{:>4} │ {line}\n", i + 1));
            }
            if total > 40 {
                out.push_str(&format!("     … {remaining} more lines\n", remaining = total - 40));
            }
            out
        }
        "apply_patch" => {
            let rel = args["path"].as_str().unwrap_or("(unknown)");
            let search = args["search"].as_str().unwrap_or("");
            let replace = args["replace"].as_str().unwrap_or("");
            // Render the change as a unified diff so the user sees exactly
            // what will be removed (-) and added (+).
            let diff = similar::TextDiff::from_lines(search, replace);
            let body = diff
                .unified_diff()
                .context_radius(3)
                .header(&format!("a/{rel}"), &format!("b/{rel}"))
                .to_string();
            body
        }
        "run_command" => format!("$ {}", args["command"].as_str().unwrap_or("")),
        _ => {
            // Generic JSON preview for MCP / unknown tools.
            serde_json::to_string_pretty(args).unwrap_or_default()
        }
    }
}

// Helper so prompt.rs can read the root from the RwLock<ProjectContext>.
impl crate::workspace::ProjectContext {
    pub fn root_cstr(&self) -> Option<String> {
        self.root_path().map(|p| p.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn describe_tool_maps_phases() {
        let (phase, detail) = describe_tool("read_file", &json!({ "path": "src/main.rs" }));
        assert_eq!(phase, "reading");
        assert_eq!(detail, "src/main.rs");

        let (phase, _) = describe_tool("apply_patch", &json!({ "path": "a.rs" }));
        assert_eq!(phase, "editing");

        let (phase, detail) = describe_tool("run_command", &json!({ "command": "cargo test" }));
        assert_eq!(phase, "running");
        assert_eq!(detail, "cargo test");

        let (phase, detail) = describe_tool("grep", &json!({ "pattern": "TODO" }));
        assert_eq!(phase, "searching");
        assert_eq!(detail, "TODO");

        let (phase, detail) = describe_tool("server__tool", &json!({}));
        assert_eq!(phase, "working");
        assert_eq!(detail, "server__tool");
    }

    #[test]
    fn describe_tool_clamps_long_details() {
        let long = "x".repeat(200);
        let (_, detail) = describe_tool("run_command", &json!({ "command": long }));
        assert!(detail.ends_with("..."));
        assert!(detail.chars().count() <= 67);
    }
}
