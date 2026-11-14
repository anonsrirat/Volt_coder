# ⚡ Volt v0.2.3 — Release Notes

**Release date:** 2026-11-14
**Focus:** one command finishes the whole job — no more stopping to ask you to
"continue".

---

## 🚀 Fixed: it no longer stops and blocks your work

Volt used to end a turn when it hit the step limit, showing **`max iterations
reached`** and leaving you to type *"ทำต่อ" / "continue"* yourself. That is gone.

- **Auto-continue** — reaching the step budget is now only a *checkpoint*.
  Volt rolls into a fresh window and keeps working **automatically**.
- A turn ends only when the task is actually **done**, a **loop** is detected,
  or **you** stop it.
- The system prompt now instructs the model to work **autonomously to
  completion** and never pause to ask for confirmation.
- A runaway guard still exists (window × 8) but a normal task never reaches it.
- **Settings → Agent → Steps before auto-continue** (10–300, default 100).
  Legacy `25` / `50` values are migrated to `100`.
- Tests: **25/25** passing (`cd src-tauri && cargo test`).

---

# Volt v0.2.2 — Release Notes

**Release date:** 2026-11-13
**Focus:** full-access autonomy, a stable agent loop, a live process Canvas and a
much faster start.

---

## ✨ New

### 🔓 Full access — no approval clicks
`Full access` is now the **default**. Volt reads, writes and runs commands
end-to-end without interrupting you. The destructive-command blocklist
(`rm -rf /`, `format`, `shutdown`, ...) still applies as a safety net.

You can turn it off in **Settings → General → Approval policy**; the individual
*ask before write / run / read* toggles then take over again.

### 🗺️ Live phase Canvas
The right-hand panel now shows the agent's process in real time, even when no
explicit plan exists:

- **Thinking → Reading → Editing → Running → Searching → Done**
- each row shows the file, command or pattern being worked on
- the current phase spins, finished phases turn green, failures turn red
- the Canvas opens automatically when you send a message

### 📝 One summary instead of a wall of tool calls
Reads, writes and tool actions are **hidden from the chat by default**. When a
turn finishes you get a single compact chip, e.g.:

> ✓ **14 actions** · 3 files · 2 commands · 6 reads

Prefer the old behaviour? Enable **Settings → General → Show every tool action**.

---

## 🐛 Fixes

### `⚠ max iterations reached` is gone
- The default limit was raised **25 → 50**, and existing configs still on 25 are
  migrated automatically.
- If a limit is ever reached the turn ends with a calm notice, **not** a red
  error banner.
- Two loop guards stop runaway turns early:
  - the same tool-call batch repeating 3× in a row
  - the same single call executing more than 4× across the turn

### Commands no longer “fail” or hang
The shell runner waited for the process to exit **before** draining stdout and
stderr. Any command producing more than the OS pipe buffer (`npm test`,
`cargo build`, ...) deadlocked until the timeout. Output is now read
concurrently, so long builds complete normally.

- timeout raised **60s → 120s** (per-call override still available)
- the child is killed on timeout instead of lingering in the background
- the system prompt now warns the model that Windows uses `cmd /C`
- the prompt asks the model to read a failure, fix it and retry at most once —
  never repeat the same failing action

---

## ⚡ Startup speed

- **Static HTML preloader** — the branded loader paints before the JS bundle,
  so there is no white flash and no silent “hang”.
- **Google Fonts removed** — the blocking `@import` to `fonts.googleapis.com`
  was removed; Volt now uses system fonts.
- **Parallel boot** — settings, providers and sessions load concurrently; the
  folder scan runs *after* the window is interactive.
- **Lazy explorer scan** — the initial tree is scanned one level deep instead of
  four; subfolders load on demand.
- **Blocking work moved off the UI thread** — `open_project` / `current_project`
  scan on a blocking worker.
- **Boot failsafe** — the overlay always disappears within ~2.5s.
- Window background is painted `#0a0a0f` from the first frame.

---

## 🎨 UI polish

- Canvas activity timeline with phase icons and live/failed states.
- Settings: **Full access** toggle and **Chat transcript → Show every tool
  action** toggle.
- Opening a file switches the right panel from Canvas to the file preview.
- New Thai/English strings for phases, summaries and notices.
- Context meter, thinking picker, copy button and keyboard hints retained.

---

## 🔧 Internal

- `ApprovalPolicy` gained `full_access`; `Settings` gained
  `show_tool_activity`; `max_iterations` is clamped to `5..=200`.
- Agent emits `ai://phase` and `ai://notice` events.
- Tests: **21/21** passing (`cd src-tauri && cargo test`).

---

## 📦 Install

1. Download **`Volt_0.2.2_x64-setup.exe`** (recommended) or the `.msi`.
2. Run the installer — the in-app update banner also picks this release up
   automatically.

> Existing `settings.json` files are migrated on first launch (full access on,
> max iterations raised). No manual steps needed.