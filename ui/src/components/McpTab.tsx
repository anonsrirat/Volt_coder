import { useEffect, useState } from "react";
import { Plug, Plus, Trash2, RefreshCw, Loader2, CheckCircle2 } from "lucide-react";
import clsx from "clsx";
import type { McpServerConfig, McpTool } from "../lib/types";
import {
  listMcpServers,
  saveMcpServer,
  deleteMcpServer,
  connectMcpServer,
  disconnectMcpServer,
  listMcpTools,
} from "../lib/ipc";
import { useT } from "../lib/i18n";

const PRESETS: { name: string; command: string; args: string[] }[] = [
  {
    name: "Filesystem",
    command: "npx",
    args: ["-y", "@modelcontextprotocol/server-filesystem", "."],
  },
  { name: "Git", command: "npx", args: ["-y", "@modelcontextprotocol/server-git"] },
  { name: "Fetch", command: "npx", args: ["-y", "@modelcontextprotocol/server-fetch"] },
  { name: "Memory", command: "npx", args: ["-y", "@modelcontextprotocol/server-memory"] },
];

export function McpTab() {
  const t = useT();
  const [servers, setServers] = useState<McpServerConfig[]>([]);
  const [tools, setTools] = useState<McpTool[]>([]);
  const [editing, setEditing] = useState<McpServerConfig | null>(null);
  const [connecting, setConnecting] = useState<string | null>(null);

  const refresh = async () => {
    setServers(await listMcpServers());
    setTools(await listMcpTools());
  };

  useEffect(() => {
    refresh();
  }, []);

  async function onConnect(s: McpServerConfig) {
    setConnecting(s.id);
    try {
      await connectMcpServer(s.id);
      await refresh();
    } catch (e) {
      alert(String(e));
    } finally {
      setConnecting(null);
    }
  }

  async function onDisconnect(s: McpServerConfig) {
    await disconnectMcpServer(s.id);
    await refresh();
  }

  const isConnected = (id: string) => tools.some((tl) => tl.server_id === id);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Plug size={15} className="accent-text" />
          <span className="label">{t("settings.mcp")}</span>
        </div>
        <button
          className="btn-primary !py-1 !text-xs"
          onClick={() =>
            setEditing({
              id: `mcp_${Math.random().toString(36).slice(2, 8)}`,
              name: "New MCP Server",
              transport: "stdio",
              command: "npx",
              args: ["-y", "@modelcontextprotocol/server-filesystem", "."],
              url: null,
              enabled: true,
            })
          }
        >
          <Plus size={14} /> {t("mcp.add")}
        </button>
      </div>

      {servers.length === 0 && !editing && (
        <div className="card text-center text-sm tx-muted py-8">{t("mcp.empty")}</div>
      )}

      {editing && (
        <McpEditor
          draft={editing}
          onChange={setEditing}
          onCancel={() => setEditing(null)}
          onSaved={async () => {
            setEditing(null);
            await refresh();
          }}
        />
      )}

      {!editing &&
        servers.map((s) => (
          <div key={s.id} className="card">
            <div className="flex items-center gap-3">
              <span
                className={clsx(
                  "h-2 w-2 rounded-full shrink-0",
                  isConnected(s.id) ? "bg-emerald-400" : "bg-slate-500",
                )}
              />
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium tx truncate">{s.name}</span>
                  {isConnected(s.id) && (
                    <span className="text-[10px] accent-text accent-soft-bg px-1.5 py-0.5 rounded">
                      {t("mcp.connected")}
                    </span>
                  )}
                </div>
                <div className="text-[11px] tx-faint mono truncate">
                  {s.transport === "stdio" ? s.command : s.url}
                </div>
              </div>
              {isConnected(s.id) ? (
                <button className="btn-ghost !text-xs" onClick={() => onDisconnect(s)}>
                  {t("mcp.disconnect")}
                </button>
              ) : (
                <button
                  className="btn-ghost !text-xs accent-text"
                  onClick={() => onConnect(s)}
                  disabled={connecting === s.id}
                >
                  {connecting === s.id ? (
                    <Loader2 size={13} className="animate-spin" />
                  ) : (
                    <RefreshCw size={13} />
                  )}
                  {t("mcp.connect")}
                </button>
              )}
              <button
                className="btn-ghost !p-2"
                onClick={() => setEditing(s)}
              >
                <Plus size={13} className="rotate-45" />
              </button>
              <button
                className="btn-ghost !p-2 hover:text-red-400"
                onClick={async () => {
                  await deleteMcpServer(s.id);
                  await refresh();
                }}
              >
                <Trash2 size={13} />
              </button>
            </div>

            {/* discovered tools */}
            {isConnected(s.id) && (
              <div className="mt-2 pt-2 border-t bd space-y-0.5">
                {tools
                  .filter((tl) => tl.server_id === s.id)
                  .map((tl) => (
                    <div key={tl.name} className="flex items-center gap-1.5 text-[11px]">
                      <CheckCircle2 size={11} className="text-emerald-400" />
                      <span className="mono tx-muted">{tl.name}</span>
                    </div>
                  ))}
              </div>
            )}
          </div>
        ))}
    </div>
  );
}

function McpEditor({
  draft,
  onChange,
  onCancel,
  onSaved,
}: {
  draft: McpServerConfig;
  onChange: (d: McpServerConfig) => void;
  onCancel: () => void;
  onSaved: () => void;
}) {
  const t = useT();
  const [saving, setSaving] = useState(false);

  function set<K extends keyof McpServerConfig>(k: K, v: McpServerConfig[K]) {
    onChange({ ...draft, [k]: v });
  }

  return (
    <div className="card space-y-3 animate-fade-in">
      <div className="flex items-center justify-between">
        <span className="label">MCP Server</span>
        <button className="btn-ghost !text-xs" onClick={onCancel}>
          ← {t("settings.back")}
        </button>
      </div>

      <div>
        <label className="label block mb-1.5">{t("settings.name")}</label>
        <input className="input" value={draft.name} onChange={(e) => set("name", e.target.value)} />
      </div>

      {/* presets */}
      <div>
        <label className="label block mb-1.5">Presets</label>
        <div className="flex flex-wrap gap-1.5">
          {PRESETS.map((p) => (
            <button
              key={p.name}
              onClick={() =>
                onChange({ ...draft, name: p.name, command: p.command, args: p.args })
              }
              className="text-xs px-2.5 py-1 rounded-full border bd hover:accent-soft-bg transition"
            >
              {p.name}
            </button>
          ))}
        </div>
      </div>

      <div>
        <label className="label block mb-1.5">{t("mcp.transport")}</label>
        <select
          className="input"
          value={draft.transport}
          onChange={(e) => set("transport", e.target.value as McpServerConfig["transport"])}
        >
          <option value="stdio">stdio</option>
          <option value="sse">sse</option>
          <option value="http">http</option>
        </select>
      </div>

      {draft.transport === "stdio" ? (
        <div>
          <label className="label block mb-1.5">{t("mcp.command")}</label>
          <input
            className="input mono text-xs"
            value={[draft.command, ...(draft.args || [])].filter(Boolean).join(" ")}
            onChange={(e) => {
              const parts = e.target.value.split(/\s+/);
              set("command", parts[0] ?? "");
              set("args", parts.slice(1));
            }}
            placeholder="npx -y @modelcontextprotocol/server-filesystem ."
          />
        </div>
      ) : (
        <div>
          <label className="label block mb-1.5">{t("mcp.url")}</label>
          <input
            className="input mono text-xs"
            value={draft.url ?? ""}
            onChange={(e) => set("url", e.target.value)}
            placeholder="http://localhost:3000/sse"
          />
        </div>
      )}

      <label className="flex items-center gap-2 text-xs tx cursor-pointer">
        <input
          type="checkbox"
          checked={draft.enabled}
          onChange={(e) => set("enabled", e.target.checked)}
          className="accent-volt-500"
        />
        {t("settings.active")}
      </label>

      <button
        className="btn-primary w-full"
        disabled={saving}
        onClick={async () => {
          setSaving(true);
          try {
            await saveMcpServer(draft, true);
            onSaved();
          } finally {
            setSaving(false);
          }
        }}
      >
        {saving && <Loader2 size={14} className="animate-spin" />}
        {t("settings.save")}
      </button>
    </div>
  );
}
