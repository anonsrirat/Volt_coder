//! Context compaction.
//!
//! When a conversation approaches the model's context window we summarise the
//! older turns into a single system note, keeping recent turns verbatim. This
//! mirrors the "compact" behaviour of modern coding agents.

use crate::ai::types::{Message, Role};
use crate::config::CompactionConfig;

/// Rough token estimate: ~4 characters per token (good enough for budgeting).
pub fn estimate_tokens(text: &str) -> u32 {
    (text.chars().count() as u32 / 4).max(1)
}

/// Estimate the token cost of a whole conversation.
pub fn conversation_tokens(messages: &[Message]) -> u32 {
    messages
        .iter()
        .map(|m| {
            let mut t = estimate_tokens(&m.content);
            for c in &m.tool_calls {
                t += estimate_tokens(&c.name) + estimate_tokens(&c.arguments.to_string());
            }
            t + 4 // per-message overhead
        })
        .sum()
}

/// Whether the conversation should be compacted given the policy + window.
pub fn should_compact(messages: &[Message], cfg: &CompactionConfig, context_window: u32) -> bool {
    if !cfg.enabled || context_window == 0 {
        return false;
    }
    let used = conversation_tokens(messages) as f32;
    let limit = context_window as f32 * cfg.threshold;
    used >= limit
}

/// Result of a compaction pass.
pub struct CompactionOutcome {
    pub messages: Vec<Message>,
    pub summarised: usize,
    pub summary: String,
}

/// Perform a heuristic compaction: fold everything except the last
/// `keep_recent_turns` user/assistant pairs into a summary system message.
///
/// A model-backed summary can be layered on top later; this keeps the agent
/// functional without an extra LLM round-trip.
pub fn compact(messages: &[Message], cfg: &CompactionConfig) -> CompactionOutcome {
    // Find the index of the Nth-from-last user message = keep boundary.
    let user_indices: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter(|(_, m)| m.role == Role::User)
        .map(|(i, _)| i)
        .collect();

    let keep = cfg.keep_recent_turns.max(1) as usize;
    let boundary = if user_indices.len() > keep {
        user_indices[user_indices.len() - keep]
    } else {
        return CompactionOutcome {
            messages: messages.to_vec(),
            summarised: 0,
            summary: String::new(),
        };
    };

    let (old, recent) = messages.split_at(boundary);
    let summary = build_summary(old);

    let mut out = Vec::with_capacity(recent.len() + 1);
    out.push(Message::system(format!(
        "[Context compacted — summary of {n} earlier messages]\n{summary}",
        n = old.len()
    )));
    out.extend_from_slice(recent);

    CompactionOutcome {
        messages: out,
        summarised: old.len(),
        summary,
    }
}

/// Build a compact digest of a slice of messages.
fn build_summary(messages: &[Message]) -> String {
    let mut turns: Vec<String> = vec![];
    for m in messages {
        let snippet = first_line(&m.content, 160);
        match m.role {
            Role::User => turns.push(format!("• User asked: {snippet}")),
            Role::Assistant => {
                if !m.tool_calls.is_empty() {
                    let names: Vec<&str> =
                        m.tool_calls.iter().map(|c| c.name.as_str()).collect();
                    turns.push(format!("• Assistant used tools [{}]", names.join(", ")));
                } else if !snippet.is_empty() {
                    turns.push(format!("• Assistant: {snippet}"));
                }
            }
            Role::Tool => turns.push(format!("• Tool result: {snippet}")),
            Role::System => {}
        }
    }
    // Cap the digest so it stays small.
    turns.truncate(40);
    turns.join("\n")
}

fn first_line(text: &str, max: usize) -> String {
    let line = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
    if line.chars().count() > max {
        let truncated: String = line.chars().take(max).collect();
        format!("{truncated}…")
    } else {
        line.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_estimate_scales() {
        assert!(estimate_tokens("hello world") >= 1);
        assert!(estimate_tokens(&"x".repeat(4000)) >= 900);
    }

    #[test]
    fn compaction_keeps_recent_turns() {
        let mut msgs = vec![Message::system("sys")];
        for i in 0..12 {
            msgs.push(Message::user(format!("question {i}")));
            msgs.push(Message::assistant(format!("answer {i}")));
        }
        let cfg = CompactionConfig { enabled: true, threshold: 0.75, keep_recent_turns: 3 };
        let out = compact(&msgs, &cfg);
        assert!(out.summarised > 0);
        assert_eq!(out.messages[0].role, Role::System);
        // 3 kept turns × 2 messages = 6, plus the summary = 7.
        assert_eq!(out.messages.len(), 7);
    }

    #[test]
    fn no_compaction_when_short() {
        let msgs = vec![Message::user("hi"), Message::assistant("hello")];
        let cfg = CompactionConfig::default();
        assert!(!should_compact(&msgs, &cfg, 128_000));
    }
}
