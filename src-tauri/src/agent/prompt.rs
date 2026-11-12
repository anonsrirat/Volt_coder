//! System-prompt construction for the coding agent.

use crate::ai::types::ThinkingLevel;
use crate::config::PluginConfig;

/// Build the system prompt, optionally embedding the project root and any
/// enabled skills/plugins.
pub fn system_prompt(
    project_root: Option<String>,
    plugins: &[PluginConfig],
    thinking: ThinkingLevel,
) -> String {
    let root_hint = match project_root {
        Some(root) => format!("The project you are working on is located at:\n  {root}\nAll file paths you use MUST be relative to this root.\n"),
        None => "No project is currently open. Ask the user to open a folder before using file tools.\n".to_string(),
    };

    let skills = build_skills_section(plugins);
    let thinking_hint = build_thinking_section(thinking);

    format!(
        r#"You are Volt, a fast and precise AI coding agent embedded in a desktop IDE.

{root_hint}{skills}{thinking_hint}
Guidelines:
- Think step by step, then act. Prefer reading files before editing them.
- Use the provided tools to inspect and modify the project. Never invent file contents.
- Tools named like `server__tool` come from connected MCP servers.
- Make minimal, surgical edits. Preserve existing code style and formatting.
- When editing, briefly explain the change in one or two sentences.
- Format code inside fenced ``` blocks with a language tag.
- If a task is ambiguous, ask a concise clarifying question instead of guessing.
- Be concise. Avoid restating the user's request back to them.
"#
    )
}

fn build_skills_section(plugins: &[PluginConfig]) -> String {
    let active: Vec<&PluginConfig> = plugins.iter().filter(|p| p.enabled).collect();
    if active.is_empty() {
        return String::new();
    }
    let mut s = String::from("\nActive skills / plugins (apply these behaviours):\n");
    for p in active {
        s.push_str(&format!("### {}\n{}\n", p.name, p.prompt.trim()));
    }
    s.push('\n');
    s
}

/// Describe the requested reasoning depth so the model can pace itself
/// even when its backend has no explicit thinking knob.
fn build_thinking_section(level: ThinkingLevel) -> String {
    let (label, guidance) = match level {
        ThinkingLevel::Off => return String::new(),
        ThinkingLevel::Auto => (
            "Auto",
            "Reason as much as the task needs — briefly for trivial asks, more thoroughly for hard ones.",
        ),
        ThinkingLevel::Low => (
            "Low",
            "Think briefly and act quickly. Prefer the obvious solution.",
        ),
        ThinkingLevel::Medium => (
            "Medium",
            "Think the change through, but keep reasoning tight and focused.",
        ),
        ThinkingLevel::High => (
            "High",
            "Think carefully and exhaustively. Weigh edge cases and verify assumptions before editing.",
        ),
    };

    format!(
        "\nReasoning depth for this task: {label}.\n{guidance} Keep any reasoning concise and do not restate it in your final answer.\n"
    )
}
