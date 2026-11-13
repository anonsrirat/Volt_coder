// Central UI state, split into focused Zustand stores.

import { create } from "zustand";
import type {
  ApprovalRequestEvent,
  ContextInfo,
  FileNode,
  Message,
  PhaseEvent,
  Plan,
  ProviderView,
  Session,
  Settings,
  ThemeMode,
  ToolCallEvent,
  ToolResultEvent,
  UpdateInfo,
  UsageEvent,
} from "./types";

// ---------------------------------------------------------------- chat store

/** A live tool-call card shown inline in the transcript. */
export interface ToolCard {
  id: string;
  name: string;
  arguments: Record<string, unknown>;
  result?: string;
  isError?: boolean;
  running: boolean;
}

/** One slice of live agent progress, shown in the Canvas timeline. */
export interface PhaseEntry {
  id: string;
  phase: string;
  detail: string;
  status: "running" | "done" | "error";
}

export interface Notice {
  kind: string;
  message: string;
}

interface ChatState {
  sessionId: string | null;
  sessions: Session[];
  messages: Message[];
  streamingText: string;
  reasoning: string;
  toolCards: ToolCard[];
  phases: PhaseEntry[];
  notice: Notice | null;
  approval: ApprovalRequestEvent | null;
  usage: UsageEvent | null;
  isStreaming: boolean;
  error: string | null;
  compactedNotice: string | null;
  contextInfo: ContextInfo | null;

  setSessionId: (id: string | null) => void;
  setSessions: (s: Session[]) => void;
  setMessages: (m: Message[]) => void;
  pushUser: (text: string) => void;

  begin: () => void;
  appendToken: (sessionId: string, text: string) => void;
  appendReasoning: (text: string) => void;
  pushToolCall: (e: ToolCallEvent) => void;
  pushToolResult: (e: ToolResultEvent) => void;
  upsertPhase: (e: PhaseEvent) => void;
  setNotice: (n: Notice | null) => void;
  setApproval: (a: ApprovalRequestEvent | null) => void;
  setUsage: (u: UsageEvent) => void;
  setError: (msg: string | null) => void;
  setCompacted: (notice: string | null) => void;
  setContextInfo: (info: ContextInfo | null) => void;
  finish: () => void;
  cancel: () => void;
  reset: () => void;
}

export const useChatStore = create<ChatState>((set) => ({
  sessionId: null,
  sessions: [],
  messages: [],
  streamingText: "",
  reasoning: "",
  toolCards: [],
  phases: [],
  notice: null,
  approval: null,
  usage: null,
  isStreaming: false,
  error: null,
  compactedNotice: null,
  contextInfo: null,

  setSessionId: (id) => set({ sessionId: id }),
  setSessions: (sessions) => set({ sessions }),
  setMessages: (messages) => set({ messages }),
  pushUser: (text) =>
    set((s) => ({ messages: [...s.messages, { role: "user", content: text }] })),

  begin: () =>
    set({
      isStreaming: true,
      streamingText: "",
      reasoning: "",
      toolCards: [],
      phases: [],
      notice: null,
      error: null,
      compactedNotice: null,
    }),

  appendToken: (_sessionId, text) =>
    set((s) => ({ streamingText: s.streamingText + text })),
  appendReasoning: (text) => set((s) => ({ reasoning: s.reasoning + text })),

  pushToolCall: (e) =>
    set((s) => {
      if (s.toolCards.some((c) => c.id === e.id)) return s;
      return {
        toolCards: [
          ...s.toolCards,
          { id: e.id, name: e.name, arguments: e.arguments, running: true },
        ],
      };
    }),

  pushToolResult: (e) =>
    set((s) => ({
      toolCards: s.toolCards.map((c) =>
        c.id === e.id ? { ...c, result: e.result, isError: e.is_error, running: false } : c,
      ),
    })),

  upsertPhase: (e) =>
    set((s) => {
      const entry: PhaseEntry = {
        id: e.id,
        phase: e.phase,
        detail: e.detail,
        status: e.status,
      };
      const idx = s.phases.findIndex((p) => p.id === entry.id);
      if (idx === -1) {
        const next = [...s.phases, entry];
        // Keep the timeline bounded so a very long run cannot grow forever.
        return { phases: next.length > 60 ? next.slice(next.length - 60) : next };
      }
      const next = s.phases.slice();
      next[idx] = entry;
      return { phases: next };
    }),

  setNotice: (notice) => set({ notice }),

  setApproval: (approval) => set({ approval }),
  setUsage: (usage) => set({ usage }),
  setError: (error) => set({ error, isStreaming: false }),
  setCompacted: (compactedNotice) => set({ compactedNotice }),
  setContextInfo: (contextInfo) => set({ contextInfo }),

  finish: () =>
    set((s) => {
      const next = [...s.messages];
      const text = s.streamingText.trim();
      if (text || s.toolCards.length) {
        next.push({
          role: "assistant",
          content: text,
          tool_calls: s.toolCards.map((c) => ({
            id: c.id,
            name: c.name,
            arguments: c.arguments,
          })),
        });
      }
      return {
        messages: next,
        streamingText: "",
        reasoning: "",
        toolCards: [],
        isStreaming: false,
        approval: null,
      };
    }),

  cancel: () => set({ isStreaming: false, streamingText: "", toolCards: [] }),

  reset: () =>
    set({
      messages: [],
      streamingText: "",
      reasoning: "",
      toolCards: [],
      phases: [],
      notice: null,
      approval: null,
      usage: null,
      error: null,
      isStreaming: false,
      compactedNotice: null,
      contextInfo: null,
    }),
}));

// ----------------------------------------------------------- workspace store

interface WorkspaceState {
  root: string | null;
  name: string | null;
  tree: FileNode[];
  activeFile: string | null;
  fileContent: string | null;
  language: string | null;
  showCanvas: boolean;

  setProject: (root: string, name: string, tree: FileNode[]) => void;
  setTree: (tree: FileNode[]) => void;
  setActiveFile: (path: string | null, content: string | null, language: string | null) => void;
  toggleCanvas: (open?: boolean) => void;
}

export const useWorkspaceStore = create<WorkspaceState>((set) => ({
  root: null,
  name: null,
  tree: [],
  activeFile: null,
  fileContent: null,
  language: null,
  showCanvas: true,

  setProject: (root, name, tree) => set({ root, name, tree }),
  setTree: (tree) => set({ tree }),
  setActiveFile: (activeFile, fileContent, language) =>
    set({ activeFile, fileContent, language }),
  toggleCanvas: (open) => set((s) => ({ showCanvas: open ?? !s.showCanvas })),
}));

// ------------------------------------------------------------ settings store

interface SettingsState {
  providers: ProviderView[];
  activeProvider: string | null;
  settingsOpen: boolean;
  settingsTab: string;
  theme: ThemeMode;
  accent: number;
  language: string;
  settings: Settings | null;
  update: UpdateInfo | null;
  updateDismissed: boolean;

  setProviders: (p: ProviderView[]) => void;
  setActiveProvider: (id: string | null) => void;
  toggleSettings: (open?: boolean, tab?: string) => void;
  setTab: (tab: string) => void;
  setTheme: (t: ThemeMode) => void;
  setAccent: (a: number) => void;
  setLanguage: (l: string) => void;
  setSettings: (s: Settings) => void;
  setUpdate: (u: UpdateInfo | null) => void;
  dismissUpdate: () => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  providers: [],
  activeProvider: null,
  settingsOpen: false,
  settingsTab: "providers",
  theme: "dark",
  accent: 262,
  language: "en",
  settings: null,
  update: null,
  updateDismissed: false,

  setProviders: (providers) => set({ providers }),
  setActiveProvider: (activeProvider) => set({ activeProvider }),
  toggleSettings: (open, tab) =>
    set((s) => ({
      settingsOpen: open ?? !s.settingsOpen,
      settingsTab: tab ?? s.settingsTab,
    })),
  setTab: (settingsTab) => set({ settingsTab }),
  setTheme: (theme) => set({ theme }),
  setAccent: (accent) => set({ accent }),
  setLanguage: (language) => set({ language }),
  setSettings: (settings) =>
    set({
      settings,
      theme: settings.theme,
      accent: settings.accent,
      language: settings.language,
      activeProvider: settings.active_provider,
    }),
  setUpdate: (update) => set({ update, updateDismissed: false }),
  dismissUpdate: () => set({ updateDismissed: true }),
}));

// -------------------------------------------------------------- plan / canvas

interface PlanState {
  plan: Plan | null;
  setPlan: (p: Plan | null) => void;
}

export const usePlanStore = create<PlanState>((set) => ({
  plan: null,
  setPlan: (plan) => set({ plan }),
}));
