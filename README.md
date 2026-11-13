<div align="center">

# ⚡ Volt

**A fast, project-aware AI coding agent — built with Rust + Tauri 2.0**

[![Rust](https://img.shields.io/badge/Rust-1.99-000?logo=rust)](https://rust-lang.org)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8DB?logo=tauri)](https://tauri.app)
[![React](https://img.shields.io/badge/React-18-61DAFB?logo=react)](https://react.dev)
[![License](https://img.shields.io/badge/License-MIT-7c4dff)](#license)

*Chat with an AI that can actually read, edit and run your project — streamed token-by-token, with your API keys stored securely in the OS keyring.*

</div>

---

## ✨ Features

### Core
| | |
|---|---|
| 🔌 **Any provider** | OpenAI, Anthropic, Gemini, OpenRouter, Groq, DeepSeek, Ollama, LM Studio, vLLM… any OpenAI-compatible endpoint + fully custom base URL/headers |
| 🛠️ **Real agent tools** | `read_file`, `write_file`, `apply_patch`, `list_dir`, `grep`, `run_command` |
| 🛡️ **Approval gate** | Every file write / shell command must be approved by you (with a numbered preview or a real unified diff) |
| 🧠 **Thinking level** | Per-message reasoning depth (Auto · Off · Low · Medium · High) sent to the model |
| 🗜️ **Context meter** | Live token-usage bar + compaction notice, right under the composer |
| 🔐 **Secure keys** | API keys live in the OS credential store — never on disk in plaintext |
| ⚡ **Streaming** | Responses stream token-by-token over Tauri events |
| 💾 **Persistent** | Chat sessions & messages stored in SQLite |

### Platform (v0.2)
| | |
|---|---|
| 🎨 **Themes** | Dark · Light · System + **6 accent colours** (live, no restart) |
| 🌐 **i18n** | English + ไทย — switch instantly, persists to settings |
| 🗺️ **Canvas** | Live execution plan with per-step progress, rendered beside the chat |
| 🔌 **MCP support** | Connect Model Context Protocol servers (filesystem, git, fetch, memory…) — their tools appear automatically |
| 🧩 **Skills & Plugins** | Custom prompt skills triggerable via slash commands |
| 🗜️ **Context compaction** | Auto-folds older turns when nearing the model's context window |
| 🔄 **Auto-update** | Checks GitHub Releases on launch and offers a one-click update |
| 📦 **Polished installer** | Custom NSIS artwork, EN/TH language selector, Start-menu folder |

---

## 🏗️ Architecture

```
┌──────────────── UI (React + Tailwind, CSS-var themes) ────────────────┐
│  Sidebar · FileTree · Chat · Canvas · CodeView · Settings(tabs)       │
└────────────────────────────▲──────────────────────────────────────────┘
                             │ Tauri IPC (invoke / emit)
┌────────────────────────────┴──────────────────────────────────────────┐
│                          Rust Core                                     │
│  AgentLoop (ReAct) ─▶ LlmProvider ─▶ OpenAI · Anthropic · Gemini · Custom │
│       │                                                                │
│       ├─▶ ToolRegistry (read/write/patch/list/grep/run)                │
│       ├─▶ McpRegistry  (stdio JSON-RPC → external MCP servers)         │
│       ├─▶ Compaction   (token budgeting + summary fold)                │
│       └─▶ PlanStore    (canvas phases)                                 │
│  SessionStore (SQLite) · ConfigStore · OS Keyring · Updater (GitHub)   │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 📋 Prerequisites

- **Rust** ≥ 1.77 · **Node.js** ≥ 18 · **WebView2** (preinstalled on Win 11)

---

## 🚀 Quick Start

```bash
npm --prefix ui install
cd src-tauri
../ui/node_modules/.bin/tauri dev      # dev + hot reload
../ui/node_modules/.bin/tauri build    # release + installers
```

### First run
1. **Open folder** → pick your project.
2. **Settings → Providers → Add provider** → paste API key → **Test connection** → **Save**.
3. (Optional) **Settings → Appearance** → pick a theme, accent and language.
4. (Optional) **Settings → MCP Servers** → add `filesystem`, `git`, etc.
5. Start chatting — Volt edits files with your approval.

---

## 🔄 Auto-update setup (GitHub)

1. Push this repo to GitHub.
2. Set the repository in one of two ways:
   - build env var `VOLT_GITHUB_REPO=owner/repo`, **or**
   - edit the fallback in `src-tauri/src/updater.rs`.
3. Tag a release — GitHub Actions builds and publishes it:
   ```bash
   git tag v0.2.0 && git push origin v0.2.0
   ```
4. Installed copies of Volt will show an **update banner** on launch.

> [!NOTE]
> The updater reads `https://api.github.com/repos/<owner>/<repo>/releases/latest`,
> compares the tag with the running version, and downloads the `*-setup.exe` asset.

---

## 🌐 Provider presets

| Type | Base URL | Notes |
|------|----------|-------|
| `openai_compat` | `https://api.openai.com/v1` | Also OpenRouter, Groq, DeepSeek, Together, Ollama (`http://localhost:11434/v1`), LM Studio, vLLM |
| `anthropic` | `https://api.anthropic.com/v1` | Claude Messages API |
| `gemini` | `https://generativelanguage.googleapis.com/v1beta` | Google Generative Language API |
| `custom` | *any* | OpenAI shape + arbitrary headers |

---

## 🔌 MCP (Model Context Protocol)

Volt speaks JSON-RPC 2.0 over stdio to MCP servers. Add one in
**Settings → MCP Servers** (presets included) — its tools are namespaced
`server__tool` and merged into the agent automatically. MCP tools always
require approval before running.

---

## 🔐 Security model

| Risk | Mitigation |
|------|-----------|
| API key leak | OS keyring; masked in UI; never logged |
| Unwanted file changes | Approval gate on write/patch/run + MCP tools |
| Path traversal | All paths canonicalised and confined to the workspace root |
| Dangerous shell | Blocklist + 60s timeout + workspace-scoped CWD |
| Over-broad permissions | Tauri `capabilities/default.json` least privilege |

---

## 🧪 Testing

```bash
cd src-tauri && cargo test      # 21 unit tests
cd ui && npm run build          # typecheck + bundle
```

---

## 📄 License

MIT © Volt
