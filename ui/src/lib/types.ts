// Shared types mirroring the Rust core's serialised structs.

export type ProviderKind = "openai_compat" | "anthropic" | "gemini" | "custom";

export interface ProviderConfig {
  id: string;
  kind: ProviderKind;
  label: string;
  base_url: string;
  model: string;
  key_ref?: string | null;
  headers?: Record<string, string>;
  use_bearer?: boolean;
  temperature?: number | null;
  max_tokens?: number | null;
  context_window?: number;
}

export interface ProviderView extends ProviderConfig {
  has_key: boolean;
  is_active: boolean;
}

export interface TestResult {
  ok: boolean;
  latency_ms: number;
  message: string;
}

export interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  children?: FileNode[] | null;
}

export interface ProjectInfo {
  root: string;
  name: string;
  tree: FileNode[];
}

export interface FileContent {
  path: string;
  content: string;
  language: string;
}

export type Role = "system" | "user" | "assistant" | "tool";

export interface ToolCall {
  id: string;
  name: string;
  arguments: Record<string, unknown>;
}

export interface Message {
  role: Role;
  content: string;
  tool_calls?: ToolCall[];
  tool_call_id?: string | null;
  name?: string | null;
}

export interface Session {
  id: string;
  title: string;
  provider_id: string | null;
  created_at: number;
  updated_at: number;
}

export interface SessionDetail {
  session: Session;
  messages: Message[];
}

// --- settings ---

export type ThemeMode = "dark" | "light" | "system";

export interface ApprovalPolicy {
  write_file: boolean;
  run_command: boolean;
  read_file: boolean;
}

export interface CompactionConfig {
  enabled: boolean;
  threshold: number;
  keep_recent_turns: number;
}

export interface McpServerConfig {
  id: string;
  name: string;
  transport: "stdio" | "sse" | "http";
  command?: string | null;
  args: string[];
  url?: string | null;
  enabled: boolean;
}

export interface McpTool {
  name: string;
  description: string;
  inputSchema: unknown;
  server_id?: string;
}

export interface PluginConfig {
  id: string;
  name: string;
  description: string;
  command?: string | null;
  prompt: string;
  tags: string[];
  enabled: boolean;
}

export interface Settings {
  providers: ProviderConfig[];
  active_provider: string | null;
  theme: ThemeMode;
  accent: number;
  language: "en" | "th";
  workspace_root: string | null;
  approval: ApprovalPolicy;
  compaction: CompactionConfig;
  max_iterations: number;
  auto_check_updates: boolean;
  last_update_check: number;
  mcp_servers: McpServerConfig[];
  plugins: PluginConfig[];
}

export interface AppInfo {
  version: string;
  github_repo: string;
  os: string;
}

// --- updater ---

export interface UpdateInfo {
  available: boolean;
  current: string;
  latest: string;
  notes: string;
  published_at: string;
  download_url: string | null;
  asset_name: string | null;
  asset_size: number | null;
  html_url: string;
}

// --- plan / canvas ---

export type StepStatus = "pending" | "in_progress" | "completed";

export interface PlanStep {
  id: string;
  title: string;
  description: string;
  status: StepStatus;
}

export interface Plan {
  title: string;
  summary: string;
  steps: PlanStep[];
}

// --- streaming / agent events ---

export interface ToolCallEvent {
  session_id: string;
  id: string;
  name: string;
  arguments: Record<string, unknown>;
}

export interface ToolResultEvent {
  id: string;
  name: string;
  result: string;
  is_error: boolean;
}

export interface ApprovalRequestEvent {
  session_id: string;
  tool_call_id: string;
  tool_name: string;
  arguments: Record<string, unknown>;
  /** Human-readable preview (numbered file preview / unified diff / command). */
  preview?: string;
}

export interface UsageEvent {
  prompt: number;
  completion: number;
}

export interface CompactedEvent {
  session_id: string;
  summarised: number;
  tokens_before: number;
}

// --- thinking / reasoning depth ---

export type ThinkingLevel = "off" | "auto" | "low" | "medium" | "high";

// --- context usage ---

export interface ContextInfo {
  context_window: number;
  used_tokens: number;
  threshold: number;
}
