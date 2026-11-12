# ⚡ Volt v0.2.1 — Release Notes

**Release date:** 2026-11-13
**Focus:** reasoning controls, a clearer approval gate, a calmer chat, and a
smoother composer.

---

## ✨ New

### 🧠 Thinking level (per message)
Pick how much the model should reason, right from the composer:
**Auto · Off · Low · Medium · High**. The choice is sent to each provider on its
own native knob:

| Provider | Knob used |
|---|---|
| OpenAI-compatible | `reasoning_effort` |
| Google Gemini | `generationConfig.thinkingConfig.thinkingBudget` |
| Anthropic (Claude) | `thinking.budget_tokens` (with an auto-lifted `max_tokens`) |

`Auto` leaves the field out so the model's own defaults apply.

### 🗜️ Context meter & compaction notice
A live token-usage bar now sits under the composer (`used / window · %`) and
turns amber as it approaches the compaction threshold. When auto-compaction
fires, a *"Context compacted · N msgs"* notice appears and stays for the turn.

### 🔎 Real diff previews in the approval gate
When Volt asks to change something you now see exactly what will happen:
- `write_file` → a numbered preview of the file being created
- `apply_patch` → a genuine **unified diff**
- `run_command` → the command that will run

---

## 🐛 Fixes

- **Explorer survives a restart** — `current_project` used to return an empty
  tree, so the file explorer was blank until you re-opened the folder. It now
  re-scans the last workspace and repopulates on launch.
- **The approval policy in Settings is now honoured** — the *write file /
  run command / read file* toggles previously had no effect (approval was
  hard-wired per tool). The gate is now driven by your policy, while MCP and
  unknown tools stay approval-gated by default.
- **`prompt.rs` build break** — `system_prompt` referenced a missing
  `build_thinking_section`, so the crate did not compile. Fixed and wired up.
- Removed dead code (`ApprovalRequest`, an unused `AgentLoop.approval` read) and
  aligned `ConfigStore` on `parking_lot` like the rest of the app state.

---

## 🎨 Composer & chat polish

- **Auto-growing input** — the composer expands as you type (up to ~10 lines)
  instead of showing a single cramped line.
- **Collapsed tool activity** — tool calls, their results and the model's
  reasoning now fold into one-line rows you can expand on demand, so the
  transcript stays short. Reloaded sessions collapse them too.
- **Copy button** on assistant messages.
- **Smarter tool summaries** — the collapsed row shows the meaningful argument
  (path / command / pattern) rather than the first field.
- **Keyboard hints** — `Enter` to send, `Shift+Enter` for a new line.
- **Sticky auto-scroll** — the chat follows new output only while you're already
  near the bottom; scroll up to read and it won't yank you back.

---

## 🔧 Internal

- Update source default is now **`anonsrirat/Volt_coder`**.
- Repository initialised: source, CI workflow and docs are under version control.
- Tests: **13/13** passing.

---

## 📦 Install

1. Download **`Volt_0.2.1_x64-setup.exe`** (recommended) or the `.msi`.
2. Run the installer — existing installs will also see an in-app update banner.

> The updater reads GitHub Releases directly, so no signing key is required for
> the update flow.
