import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import clsx from "clsx";
import { Brain, ChevronRight, Wrench, CheckCircle2, XCircle, Loader2 } from "lucide-react";
import { useState } from "react";
import type { Message } from "../lib/types";
import type { ToolCard } from "../lib/store";

interface Props {
  message: Message;
  live?: boolean;
  reasoning?: string;
  toolCards?: ToolCard[];
}

export function MessageBubble({ message, live, reasoning, toolCards }: Props) {
  const isUser = message.role === "user";
  if (message.role === "tool") return <ToolMessage message={message} />;

  return (
    <div className={clsx("flex gap-3 py-3 animate-fade-in", isUser && "flex-row-reverse")}>
      <Avatar isUser={isUser} />

      <div className={clsx("min-w-0 max-w-[85%] space-y-2", isUser && "items-end flex flex-col")}>
        {reasoning && reasoning.trim() && (
          <details className="text-xs tx-faint group w-full">
            <summary className="flex gap-1.5 items-center cursor-pointer list-none select-none hover:tx-muted">
              <ChevronRight size={13} className="shrink-0 transition-transform group-open:rotate-90" />
              <Brain size={13} className="shrink-0" />
              <span>{reasoning.trim().length > 80 ? `${reasoning.trim().slice(0, 80)}…` : reasoning.trim()}</span>
            </summary>
            <div className="mt-1 ml-6 italic whitespace-pre-wrap border-l bd pl-3">
              {reasoning.trim()}
            </div>
          </details>
        )}

        {message.content?.trim() && (
          <div
            className={clsx(
              "rounded-2xl px-4 py-2.5 text-sm leading-relaxed",
              isUser ? "accent-bg text-white rounded-tr-sm" : "surface-panel border bd tx rounded-tl-sm",
            )}
          >
            {isUser ? (
              <span className="whitespace-pre-wrap">{message.content}</span>
            ) : (
              <div className="prose-volt">
                <ReactMarkdown remarkPlugins={[remarkGfm]}>{message.content}</ReactMarkdown>
                {live && <Caret />}
              </div>
            )}
          </div>
        )}

        {live && !message.content?.trim() && !reasoning?.trim() && (
          <div className="rounded-2xl px-4 py-3 surface-panel border bd">
            <Thinking />
          </div>
        )}

        {toolCards?.map((c) => (
          <ToolCardView key={c.id} card={c} />
        ))}

        {!live &&
          message.tool_calls?.map((tc) => (
            <div key={tc.id} className="text-xs tx-faint flex items-center gap-1.5 font-mono">
              <Wrench size={12} className="accent-text/70" />
              {tc.name}
            </div>
          ))}
      </div>
    </div>
  );
}

function ToolCardView({ card }: { card: ToolCard }) {
  const [open, setOpen] = useState(false);
  const status = card.running ? (
    <Loader2 size={13} className="animate-spin accent-text" />
  ) : card.isError ? (
    <XCircle size={13} className="text-red-400" />
  ) : (
    <CheckCircle2 size={13} className="text-emerald-400" />
  );

  return (
    <div className="w-full rounded-xl border bd surface-panel overflow-hidden">
      <button
        onClick={() => setOpen((v) => !v)}
        className="w-full flex items-center gap-2 px-3 py-2 text-xs hover:accent-soft-bg transition"
      >
        {status}
        <Wrench size={12} className="accent-text/70" />
        <span className="font-mono tx">{card.name}</span>
        <span className="tx-faint truncate flex-1 text-left">{summarizeArgs(card.arguments)}</span>
        <ChevronRight
          size={13}
          className={clsx("tx-faint transition-transform", open && "rotate-90")}
        />
      </button>
      {open && (
        <div className="border-t bd p-3 space-y-2">
          <pre className="text-[11px] tx-muted overflow-x-auto font-mono">
            {JSON.stringify(card.arguments, null, 2)}
          </pre>
          {card.result !== undefined && (
            <pre
              className={clsx(
                "text-[11px] overflow-x-auto font-mono max-h-64 rounded-lg p-2",
                card.isError ? "bg-red-500/10 text-red-300" : "surface-input tx",
              )}
            >
              {card.result}
            </pre>
          )}
        </div>
      )}
    </div>
  );
}

function ToolMessage({ message }: { message: Message }) {
  return (
    <div className="pl-11 pr-4 py-1">
      <div className="text-xs font-mono tx-faint border-l-2 accent-border pl-3 py-1 whitespace-pre-wrap max-h-40 overflow-y-auto">
        {message.content}
      </div>
    </div>
  );
}

function Avatar({ isUser }: { isUser: boolean }) {
  return (
    <div
      className={clsx(
        "h-7 w-7 shrink-0 rounded-lg grid place-items-center text-xs font-semibold",
        isUser ? "bg-slate-600 text-white" : "accent-bg text-white shadow-glow-sm",
      )}
    >
      {isUser ? (
        "You"
      ) : (
        <svg width="13" height="13" viewBox="0 0 24 24" fill="white">
          <path d="M13 2L4.5 13.5H11L10 22l8.5-11.5H12L13 2z" />
        </svg>
      )}
    </div>
  );
}

function Caret() {
  return (
    <span className="inline-block w-1.5 h-4 ml-0.5 -mb-0.5 accent-bg animate-pulse rounded-sm" />
  );
}

function Thinking() {
  return (
    <div className="flex gap-1">
      {[0, 150, 300].map((d) => (
        <span
          key={d}
          className="h-1.5 w-1.5 rounded-full accent-bg animate-bounce opacity-70"
          style={{ animationDelay: `${d}ms` }}
        />
      ))}
    </div>
  );
}

function summarizeArgs(args: Record<string, unknown>): string {
  const first = Object.values(args)[0];
  if (typeof first === "string") return first.slice(0, 60);
  return "";
}
