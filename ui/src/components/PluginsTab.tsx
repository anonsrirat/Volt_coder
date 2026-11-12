import { useEffect, useState } from "react";
import { Sparkles, Plus, Trash2, Loader2 } from "lucide-react";
import clsx from "clsx";
import type { PluginConfig } from "../lib/types";
import { listPlugins, savePlugin, togglePlugin, deletePlugin } from "../lib/ipc";
import { useT } from "../lib/i18n";

/** Skills & Plugins tab — prompt-injected behaviours triggerable via slash commands. */
export function PluginsTab() {
  const t = useT();
  const [plugins, setPlugins] = useState<PluginConfig[]>([]);
  const [editing, setEditing] = useState<PluginConfig | null>(null);

  const refresh = async () => setPlugins(await listPlugins());
  useEffect(() => {
    refresh();
  }, []);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Sparkles size={15} className="accent-text" />
          <span className="label">{t("settings.plugins")}</span>
        </div>
        <button
          className="btn-primary !py-1 !text-xs"
          onClick={() =>
            setEditing({
              id: `plugin_${Math.random().toString(36).slice(2, 8)}`,
              name: "New Skill",
              description: "",
              command: "/skill",
              prompt: "",
              tags: [],
              enabled: true,
            })
          }
        >
          <Plus size={14} /> {t("plugins.add")}
        </button>
      </div>

      {plugins.length === 0 && !editing && (
        <div className="card text-center text-sm tx-muted py-8">{t("plugins.empty")}</div>
      )}

      {editing && (
        <PluginEditor
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
        plugins.map((p) => (
          <div key={p.id} className="card">
            <div className="flex items-start gap-3">
              <button
                onClick={async () => {
                  await togglePlugin(p.id, !p.enabled);
                  await refresh();
                }}
                className={clsx(
                  "mt-0.5 h-4 w-4 shrink-0 rounded border transition",
                  p.enabled ? "accent-bg accent-border" : "bd-strong",
                )}
                title={p.enabled ? "Disable" : "Enable"}
              />
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium tx">{p.name}</span>
                  {p.command && (
                    <span className="text-[10px] mono accent-text accent-soft-bg px-1.5 py-0.5 rounded">
                      {p.command}
                    </span>
                  )}
                </div>
                {p.description && (
                  <p className="text-xs tx-faint mt-0.5">{p.description}</p>
                )}
              </div>
              <button
                className="btn-ghost !p-1.5 accent-text"
                onClick={() => setEditing(p)}
              >
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" />
                  <path d="M18.5 2.5a2.12 2.12 0 013 3L12 15l-4 1 1-4z" />
                </svg>
              </button>
              <button
                className="btn-ghost !p-1.5 hover:text-red-400"
                onClick={async () => {
                  await deletePlugin(p.id);
                  await refresh();
                }}
              >
                <Trash2 size={13} />
              </button>
            </div>
          </div>
        ))}
    </div>
  );
}

function PluginEditor({
  draft,
  onChange,
  onCancel,
  onSaved,
}: {
  draft: PluginConfig;
  onChange: (d: PluginConfig) => void;
  onCancel: () => void;
  onSaved: () => void;
}) {
  const t = useT();
  const [saving, setSaving] = useState(false);

  function set<K extends keyof PluginConfig>(k: K, v: PluginConfig[K]) {
    onChange({ ...draft, [k]: v });
  }

  return (
    <div className="card space-y-3 animate-fade-in">
      <div className="flex items-center justify-between">
        <span className="label">Skill</span>
        <button className="btn-ghost !text-xs" onClick={onCancel}>
          ← {t("settings.back")}
        </button>
      </div>

      <div>
        <label className="label block mb-1.5">{t("settings.name")}</label>
        <input className="input" value={draft.name} onChange={(e) => set("name", e.target.value)} />
      </div>

      <div>
        <label className="label block mb-1.5">{t("plugins.command")}</label>
        <input
          className="input mono text-xs"
          value={draft.command ?? ""}
          onChange={(e) => set("command", e.target.value)}
          placeholder="/docker"
        />
      </div>

      <div>
        <label className="label block mb-1.5">Description</label>
        <input
          className="input"
          value={draft.description}
          onChange={(e) => set("description", e.target.value)}
        />
      </div>

      <div>
        <label className="label block mb-1.5">{t("plugins.prompt")}</label>
        <textarea
          className="input mono text-xs"
          rows={6}
          value={draft.prompt}
          onChange={(e) => set("prompt", e.target.value)}
          placeholder="You are a Docker expert. Always write multi-stage Dockerfiles…"
        />
      </div>

      <button
        className="btn-primary w-full"
        disabled={saving}
        onClick={async () => {
          setSaving(true);
          try {
            await savePlugin(draft);
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
