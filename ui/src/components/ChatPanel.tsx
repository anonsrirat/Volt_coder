import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { Send, Square, Sparkles, Minimize2, Brain, ChevronDown, Check, Info } from "lucide-react";
import { useChatStore, useSettingsStore, useWorkspaceStore } from "../lib/store";
import { useT } from "../lib/i18n";
import { MessageBubble } from "./MessageBubble";
import { ApprovalCard } from "./ApprovalCard";
import { sendMessage, stopGeneration, listSessions, createSession } from "../lib/ipc";
import type { ThinkingLevel } from "../lib/types";

const THINKING_LEVELS: ThinkingLevel[] = ["auto", "off", "low", "medium", "high"];

export function ChatPanel() {
  const {
    messages, streamingText, reasoning, toolCards, approval, isStreaming, sessionId,
    error, compactedNotice, contextInfo, notice, begin, pushUser, finish, cancel, setSessionId,
    setSessions, setError,
  } = useChatStore();
  const { activeProvider } = useSettingsStore();
  const t = useT();
  const [input, setInput] = useState("");
  const [thinking, setThinking] = useState<ThinkingLevel>("auto");
  const scrollRef = useRef<HTMLDivElement>(null);
  const taRef = useRef<HTMLTextAreaElement>(null);
  const pinnedRef = useRef(true);

  // Auto-grow the textarea to fit its content (up to a cap).
  useLayoutEffect(() => {
    const el = taRef.current;
    if (!el) return;
    el.style.height = "0px";
    const next = Math.min(el.scrollHeight, 240);
    el.style.height = `${Math.max(next, 46)}px`;
  }, [input]);

  // Only auto-scroll while the user is already near the bottom.
  const scrollToBottom = useCallback(() => {
    const el = scrollRef.current;
    if (el && pinnedRef.current) el.scrollTop = el.scrollHeight;
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages, streamingText, toolCards, scrollToBottom]);

  function onScroll() {
    const el = scrollRef.current;
    if (!el) return;
    pinnedRef.current = el.scrollHeight - el.scrollTop - el.clientHeight < 120;
  }

  async function onSubmit() {
    const text = input.trim();
    if (!text || isStreaming || !activeProvider) return;

    let sid = sessionId;
    if (!sid) {
      const s = await createSession(text.slice(0, 40), activeProvider);
      sid = s.id;
      setSessionId(sid);
      setSessions(await listSessions());
    }

    pushUser(text);
    setInput("");
    pinnedRef.current = true;
    begin();
    // Bring the Canvas forward so the user can watch each phase live.
    useWorkspaceStore.getState().toggleCanvas(true);
    try {
      await sendMessage(sid, text, activeProvider, thinking);
    } catch (e) {
      setError(String(e));
    } finally {
      finish();
    }
  }

  async function onStop() {
    if (sessionId) await stopGeneration(sessionId);
    cancel();
  }

  function onKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      onSubmit();
    }
  }

  const empty = messages.length === 0 && !streamingText && !isStreaming;

  const noticeText =
    notice?.kind === "max_iterations"
      ? t("notice.max_iterations")
      : notice?.kind === "loop_guard"
        ? t("notice.loop_guard")
        : notice?.kind === "cancelled"
          ? t("notice.cancelled")
          : notice
            ? t("notice.info", { message: notice.message })
            : "";

  return (
    <section className="flex-1 min-w-0 flex flex-col">
      <div ref={scrollRef} onScroll={onScroll} className="flex-1 overflow-y-auto px-6 py-6">
        <div className="mx-auto max-w-3xl w-full space-y-1">
          {empty && <Welcome />}

          {messages.map((m, i) => (
            <MessageBubble key={i} message={m} />
          ))}

          {(isStreaming || streamingText) && (
            <MessageBubble
              message={{ role: "assistant", content: streamingText }}
              live
              reasoning={reasoning}
              toolCards={toolCards}
            />
          )}

          {approval && <ApprovalCard approval={approval} />}

          {error && (
            <div className="card border-red-500/30 bg-red-500/10 text-red-300 text-sm my-3">
              ⚠ {error}
            </div>
          )}
        </div>
      </div>

      {/* Composer */}
      <div className="shrink-0 border-t bd p-4 glass">
        <div className="mx-auto max-w-3xl">
          {compactedNotice && (
            <div className="mb-2 flex items-center gap-1.5 text-[11px] tx-faint">
              <Minimize2 size={12} className="accent-text" />
              {compactedNotice}
            </div>
          )}

          {noticeText && (
            <div className="mb-2 flex items-center gap-1.5 text-[11px] text-amber-300/90">
              <Info size={12} className="shrink-0" />
              {noticeText}
            </div>
          )}

          <div
            className="relative rounded-xl border bd-strong surface-input focus-within:accent-border transition"
            style={{ boxShadow: "none" }}
          >
            <textarea
              ref={taRef}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={onKeyDown}
              rows={1}
              placeholder={
                activeProvider ? t("chat.placeholder") : t("chat.placeholderNoProvider")
              }
              disabled={!activeProvider}
              className="w-full resize-none bg-transparent px-4 pt-3 pb-11 text-sm outline-none tx overflow-y-auto"
              style={{ minHeight: "46px", maxHeight: "240px" }}
            />

            {/* toolbar row */}
            <div className="absolute left-2 bottom-2 flex items-center gap-1">
              <ThinkingPicker value={thinking} onChange={setThinking} />
            </div>

            <div className="absolute right-2 bottom-2 flex items-center gap-2">
              {isStreaming ? (
                <button className="btn-ghost !px-2.5 text-red-300" onClick={onStop}>
                  <Square size={14} /> {t("chat.stop")}
                </button>
              ) : (
                <button
                  className="btn-primary !px-3"
                  onClick={onSubmit}
                  disabled={!input.trim() || !activeProvider}
                >
                  <Send size={15} />
                </button>
              )}
            </div>
          </div>

          <ContextMeter info={contextInfo} />
          <p className="mt-2 text-[11px] tx-faint text-center">
            {t("chat.note")} · <kbd className="tx-muted">Enter</kbd> {t("chat.sendKey")} ·{" "}
            <kbd className="tx-muted">Shift+Enter</kbd> {t("chat.newlineKey")}
          </p>
        </div>
      </div>
    </section>
  );
}

/** Dropdown that picks how much reasoning budget the model gets. */
function ThinkingPicker({
  value,
  onChange,
}: {
  value: ThinkingLevel;
  onChange: (v: ThinkingLevel) => void;
}) {
  const t = useT();
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    function onDoc(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    }
    document.addEventListener("mousedown", onDoc);
    return () => document.removeEventListener("mousedown", onDoc);
  }, [open]);

  return (
    <div ref={ref} className="relative">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        title={t("chat.thinking")}
        className="flex items-center gap-1.5 rounded-lg px-2 py-1 text-xs tx-muted hover:accent-soft-bg transition"
      >
        <Brain size={13} className={value !== "off" ? "accent-text" : "opacity-60"} />
        <span className="hidden sm:inline">{t(`thinking.${value}`)}</span>
        <ChevronDown size={12} className={open ? "rotate-180 transition" : "transition"} />
      </button>

      {open && (
        <div className="absolute bottom-full left-0 mb-2 w-60 rounded-xl border bd-strong glass shadow-xl p-1 animate-fade-in z-30">
          <div className="px-2.5 py-1.5 text-[10px] uppercase tracking-wide tx-faint">
            {t("chat.thinking")}
          </div>
          {THINKING_LEVELS.map((lv) => (
            <button
              key={lv}
              type="button"
              onClick={() => {
                onChange(lv);
                setOpen(false);
              }}
              className="w-full flex items-center gap-2 rounded-lg px-2.5 py-2 text-left hover:accent-soft-bg transition"
            >
              <div className="min-w-0 flex-1">
                <div className="text-xs tx font-medium">{t(`thinking.${lv}`)}</div>
                <div className="text-[10px] tx-faint truncate">{t(`thinking.${lv}.hint`)}</div>
              </div>
              {value === lv && <Check size={14} className="accent-text shrink-0" />}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

/** Thin context-window usage bar shown under the composer. */
function ContextMeter({ info }: { info: ReturnType<typeof useChatStore.getState>["contextInfo"] }) {
  const t = useT();
  if (!info || info.context_window <= 0) return null;
  const pct = Math.min(100, Math.round((info.used_tokens / info.context_window) * 100));
  const near = pct >= Math.round(info.threshold * 100) * 0.8;
  return (
    <div className="mt-2 flex items-center gap-2">
      <div className="flex-1 h-1 rounded-full surface-input overflow-hidden">
        <div
          className={near ? "h-full bg-amber-400 transition-all" : "h-full accent-bg transition-all"}
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className="text-[10px] tx-faint tabular-nums">
        {t("chat.context")} {fmt(info.used_tokens)} / {fmt(info.context_window)} · {pct}%
      </span>
    </div>
  );
}

function fmt(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`;
  return String(n);
}

function Welcome() {
  const { activeProvider } = useSettingsStore();
  const t = useT();
  return (
    <div className="py-16 text-center animate-fade-in">
      <div className="mx-auto grid place-items-center h-14 w-14 rounded-2xl accent-bg shadow-glow mb-4">
        <svg width="28" height="28" viewBox="0 0 24 24" fill="white">
          <path d="M13 2L4.5 13.5H11L10 22l8.5-11.5H12L13 2z" />
        </svg>
      </div>
      <h1 className="text-xl font-semibold tx">Volt</h1>
      <p className="text-sm tx-muted mt-1">{t("chat.welcomeSub")}</p>
      <div className="mt-6 flex flex-wrap gap-2 justify-center max-w-md mx-auto">
        {[
          "Explain this codebase structure",
          "Add a REST endpoint for users",
          "Find and fix a bug in the auth flow",
          "Write unit tests for the parser",
        ].map((s) => (
          <span key={s} className="text-xs px-3 py-1.5 rounded-full border bd tx-faint">
            <Sparkles size={11} className="inline mr-1 opacity-60" />
            {s}
          </span>
        ))}
      </div>
      {!activeProvider && (
        <p className="mt-6 text-xs text-amber-300/90">{t("chat.noProvider")}</p>
      )}
    </div>
  );
}
