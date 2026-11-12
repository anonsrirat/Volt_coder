// Bridges Rust-emitted events (`ai://…`, `plan://…`) into the Zustand stores.

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useChatStore, usePlanStore } from "./store";
import { translate } from "./i18n";
import { useSettingsStore } from "./store";
import type {
  ApprovalRequestEvent,
  CompactedEvent,
  Plan,
  ToolCallEvent,
  ToolResultEvent,
  UsageEvent,
} from "./types";

/** Subscribe to every agent event; returns a cleanup function. */
export async function subscribeToAgentEvents(): Promise<UnlistenFn> {
  const store = useChatStore.getState;

  const unlisteners = await Promise.all([
    listen<{ session_id: string; text: string }>("ai://token", (e) => {
      store().appendToken(e.payload.session_id, e.payload.text);
    }),
    listen<{ text: string }>("ai://reasoning", (e) => {
      store().appendReasoning(e.payload.text);
    }),
    listen<ToolCallEvent>("ai://tool-call", (e) => {
      store().pushToolCall(e.payload);
    }),
    listen<ToolResultEvent>("ai://tool-result", (e) => {
      store().pushToolResult(e.payload);
    }),
    listen<ApprovalRequestEvent>("ai://approval-request", (e) => {
      store().setApproval(e.payload);
    }),
    listen<UsageEvent>("ai://usage", (e) => {
      store().setUsage(e.payload);
    }),
    listen<{ message: string }>("ai://error", (e) => {
      store().setError(e.payload.message);
    }),
    listen<CompactedEvent>("ai://compacted", (e) => {
      const lang = useSettingsStore.getState().language as "en" | "th";
      store().setCompacted(
        `${translate(lang, "canvas.compacted")} · ${e.payload.summarised} msgs`,
      );
    }),
    listen<{ session_id: string; plan: Plan }>("plan://updated", (e) => {
      usePlanStore.getState().setPlan(e.payload.plan);
    }),
    listen("ai://done", () => {
      store().finish();
    }),
  ]);

  return () => unlisteners.forEach((u) => u());
}
