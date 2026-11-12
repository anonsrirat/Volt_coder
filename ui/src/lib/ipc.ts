// Typed wrappers around Tauri's `invoke` — the single place the UI talks to Rust.

import { invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  ContextInfo,
  FileContent,
  FileNode,
  McpServerConfig,
  McpTool,
  Message,
  Plan,
  PluginConfig,
  ProjectInfo,
  ProviderConfig,
  ProviderView,
  Session,
  SessionDetail,
  Settings,
  TestResult,
  UpdateInfo,
} from "./types";

// --- workspace ---

export const openProject = (path: string) =>
  invoke<ProjectInfo>("open_project", { path });

export const currentProject = () =>
  invoke<ProjectInfo | null>("current_project");

export const listDir = (path: string) =>
  invoke<FileNode[]>("list_dir", { path });

export const readFile = (path: string) =>
  invoke<FileContent>("read_file", { path });

// --- providers ---

export const listProviders = () =>
  invoke<ProviderView[]>("list_providers");

export interface SaveProviderInput extends ProviderConfig {
  api_key?: string | null;
  make_active?: boolean;
}

export const saveProvider = (input: SaveProviderInput) =>
  invoke<void>("save_provider", { input });

export const deleteProvider = (id: string) =>
  invoke<void>("delete_provider", { id });

export const setActiveProvider = (id: string) =>
  invoke<void>("set_active_provider", { id });

export const testConnection = (id: string) =>
  invoke<TestResult>("test_connection", { id });

// --- chat / agent ---

export const sendMessage = (
  sessionId: string,
  text: string,
  providerId: string,
  thinking?: string,
) => invoke<void>("send_message", { sessionId, text, providerId, thinking });

export const stopGeneration = (sessionId: string) =>
  invoke<void>("stop_generation", { sessionId });

export const respondApproval = (toolCallId: string, approved: boolean) =>
  invoke<void>("respond_approval", { toolCallId, approved });

// --- sessions ---

export const listSessions = () =>
  invoke<Session[]>("list_sessions");

export const createSession = (title?: string, providerId?: string) =>
  invoke<Session>("create_session", { title, providerId });

export const loadSession = (id: string) =>
  invoke<SessionDetail>("load_session", { id });

export const deleteSession = (id: string) =>
  invoke<void>("delete_session", { id });

// --- settings ---

export const getSettings = () =>
  invoke<Settings>("get_settings");

export interface SettingsPatch {
  theme?: string;
  accent?: number;
  language?: string;
  max_iterations?: number;
  auto_check_updates?: boolean;
  approval?: Settings["approval"];
  compaction?: Settings["compaction"];
}

export const updateSettings = (patch: SettingsPatch) =>
  invoke<Settings>("update_settings", { patch });

export const appInfo = () =>
  invoke<AppInfo>("app_info");

/** Estimate live context usage for a set of messages. */
export const contextTokens = (messages: Message[], providerId?: string | null) =>
  invoke<ContextInfo>("context_tokens", { messages, providerId: providerId ?? null });

// --- plan / canvas ---

export const getPlan = (sessionId: string) =>
  invoke<Plan | null>("get_plan", { sessionId });

export const setPlan = (sessionId: string, plan: Plan) =>
  invoke<void>("set_plan", { sessionId, plan });

export const updatePlanStep = (sessionId: string, stepId: string, status: string) =>
  invoke<void>("update_plan_step", { sessionId, stepId, status });

// --- mcp ---

export const listMcpServers = () =>
  invoke<McpServerConfig[]>("list_mcp_servers");

export const saveMcpServer = (config: McpServerConfig, connectNow = false) =>
  invoke<void>("save_mcp_server", { input: { ...config, connect_now: connectNow } });

export const deleteMcpServer = (id: string) =>
  invoke<void>("delete_mcp_server", { id });

export const connectMcpServer = (id: string) =>
  invoke<McpTool[]>("connect_mcp_server", { id });

export const connectAllMcp = () =>
  invoke<number>("connect_all_mcp");

export const disconnectMcpServer = (id: string) =>
  invoke<void>("disconnect_mcp_server", { id });

export const listMcpTools = () =>
  invoke<McpTool[]>("list_mcp_tools");

// --- plugins / skills ---

export const listPlugins = () =>
  invoke<PluginConfig[]>("list_plugins");

export const savePlugin = (plugin: PluginConfig) =>
  invoke<void>("save_plugin", { plugin });

export const togglePlugin = (id: string, enabled: boolean) =>
  invoke<void>("toggle_plugin", { id, enabled });

export const deletePlugin = (id: string) =>
  invoke<void>("delete_plugin", { id });

// --- updater ---

export const checkUpdate = () =>
  invoke<UpdateInfo>("check_update");

export const downloadUpdate = (url: string, filename: string) =>
  invoke<string>("download_update", { url, filename });

export const installUpdate = (path: string) =>
  invoke<void>("install_update", { path });
