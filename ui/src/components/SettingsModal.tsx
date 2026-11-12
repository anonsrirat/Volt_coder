import { useEffect, useState } from "react";
import {
  X, Plus, Trash2, Zap, Loader2, CheckCircle2, XCircle, Eye, EyeOff,
  Server, Palette, SlidersHorizontal, Plug, Sparkles, Info,
} from "lucide-react";
import clsx from "clsx";
import { useSettingsStore } from "../lib/store";
import { useT } from "../lib/i18n";
import {
  deleteProvider, listProviders, saveProvider, setActiveProvider, testConnection,
} from "../lib/ipc";
import type { ProviderKind, ProviderView } from "../lib/types";
import { AppearanceTab } from "./AppearanceTab";
import { GeneralTab } from "./GeneralTab";
import { McpTab } from "./McpTab";
import { PluginsTab } from "./PluginsTab";
import { AboutTab } from "./AboutTab";

const PRESETS: Record<
  ProviderKind,
  { label: string; base_url: string; model: string; hint: string }
> = {
  openai_compat: {
    label: "OpenAI-compatible",
    base_url: "https://api.openai.com/v1",
    model: "gpt-4o",
    hint: "OpenAI, OpenRouter, Groq, DeepSeek, Together, Ollama, LM Studio, vLLM…",
  },
  anthropic: {
    label: "Anthropic (Claude)",
    base_url: "https://api.anthropic.com/v1",
    model: "claude-3-5-sonnet-latest",
    hint: "Claude Messages API",
  },
  gemini: {
    label: "Google Gemini",
    base_url: "https://generativelanguage.googleapis.com/v1beta",
    model: "gemini-1.5-pro",
    hint: "Google Generative Language API",
  },
  custom: {
    label: "Custom endpoint",
    base_url: "http://localhost:8000/v1",
    model: "my-model",
    hint: "Any OpenAI-shape endpoint with custom headers",
  },
};

export function SettingsModal() {
  const { providers, activeProvider, toggleSettings, setProviders, setActiveProvider, settingsTab, setTab } =
    useSettingsStore();
  const t = useT();
  const [editing, setEditing] = useState<ProviderDraft | null>(null);

  useEffect(() => {
    listProviders().then(setProviders).catch(() => {});
  }, [setProviders]);

  const tabs = [
    { id: "providers", icon: Server, label: t("settings.providers") },
    { id: "appearance", icon: Palette, label: t("settings.appearance") },
    { id: "general", icon: SlidersHorizontal, label: t("settings.general") },
    { id: "mcp", icon: Plug, label: t("settings.mcp") },
    { id: "plugins", icon: Sparkles, label: t("settings.plugins") },
    { id: "about", icon: Info, label: t("settings.about") },
  ];

  return (
    <div className="fixed inset-0 z-40 flex">
      <div className="flex-1 bg-black/50 backdrop-blur-sm" onClick={() => toggleSettings(false)} />

      <div className="w-[680px] max-w-full glass border-l bd-strong flex flex-col animate-fade-in">
        {/* header */}
        <div className="h-12 shrink-0 flex items-center justify-between px-5 border-b bd">
          <h2 className="font-semibold tx">{t("settings.title")}</h2>
          <button className="btn-ghost !p-1.5" onClick={() => toggleSettings(false)}>
            <X size={16} />
          </button>
        </div>

        <div className="flex-1 flex min-h-0">
          {/* tab rail */}
          <nav className="w-44 shrink-0 border-r bd p-2 space-y-0.5">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => {
                  setTab(tab.id);
                  setEditing(null);
                }}
                className={clsx(
                  "w-full flex items-center gap-2.5 rounded-lg px-3 py-2 text-sm transition text-left",
                  settingsTab === tab.id
                    ? "accent-soft-bg accent-text font-medium"
                    : "tx-muted hover:accent-soft-bg",
                )}
              >
                <tab.icon size={15} />
                {tab.label}
              </button>
            ))}
          </nav>

          {/* content */}
          <div className="flex-1 overflow-y-auto p-5">
            {settingsTab === "providers" && (
              <ProvidersTab
                providers={providers}
                activeProvider={activeProvider}
                editing={editing}
                setEditing={setEditing}
                setProviders={setProviders}
                setActiveProvider={setActiveProvider}
                t={t}
              />
            )}
            {settingsTab === "appearance" && <AppearanceTab />}
            {settingsTab === "general" && <GeneralTab />}
            {settingsTab === "mcp" && <McpTab />}
            {settingsTab === "plugins" && <PluginsTab />}
            {settingsTab === "about" && <AboutTab />}
          </div>
        </div>
      </div>
    </div>
  );
}

// ------------------------------------------------------------------ providers

function ProvidersTab({
  providers, activeProvider, editing, setEditing, setProviders, setActiveProvider, t,
}: {
  providers: ProviderView[];
  activeProvider: string | null;
  editing: ProviderDraft | null;
  setEditing: (d: ProviderDraft | null) => void;
  setProviders: (p: ProviderView[]) => void;
  setActiveProvider: (id: string | null) => void;
  t: (k: string, v?: Record<string, string | number>) => string;
}) {
  if (editing) {
    return (
      <ProviderEditor
        draft={editing}
        onChange={setEditing}
        onCancel={() => setEditing(null)}
        onSaved={async () => {
          setEditing(null);
          const list = await listProviders();
          setProviders(list);
          const active = list.find((p) => p.is_active);
          if (active) setActiveProvider(active.id);
        }}
        t={t}
      />
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <span className="label">{t("settings.providers")}</span>
        <button className="btn-primary !py-1 !text-xs" onClick={() => setEditing(blankDraft())}>
          <Plus size={14} /> {t("settings.addProvider")}
        </button>
      </div>

      {providers.length === 0 && (
        <div className="card text-center text-sm tx-muted py-8">{t("settings.noProviders")}</div>
      )}

      {providers.map((p) => (
        <div
          key={p.id}
          className={clsx("card flex items-center gap-3 transition", p.id === activeProvider && "accent-border accent-soft-bg")}
        >
          <span
            className={clsx("h-2 w-2 rounded-full shrink-0", p.has_key ? "bg-emerald-400" : "bg-amber-400")}
            title={p.has_key ? "API key stored" : "No API key yet"}
          />
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <span className="text-sm font-medium tx truncate">{p.label}</span>
              {p.id === activeProvider && (
                <span className="text-[10px] uppercase tracking-wide accent-text accent-soft-bg px-1.5 py-0.5 rounded">
                  {t("settings.active")}
                </span>
              )}
            </div>
            <div className="text-xs tx-faint truncate">
              {PRESETS[p.kind]?.label} · {p.model}
            </div>
          </div>
          {p.id !== activeProvider && (
            <button
              className="btn-ghost !text-xs"
              onClick={async () => {
                await setActiveProvider(p.id);
                setActiveProvider(p.id);
                setProviders(await listProviders());
              }}
            >
              {t("settings.use")}
            </button>
          )}
          <button className="btn-ghost !p-2 accent-text" onClick={() => setEditing(toDraft(p))}>
            <Zap size={14} />
          </button>
          <button
            className="btn-ghost !p-2 hover:text-red-400"
            onClick={async () => {
              await deleteProvider(p.id);
              setProviders(await listProviders());
            }}
          >
            <Trash2 size={14} />
          </button>
        </div>
      ))}
    </div>
  );
}

// ------------------------------------------------------------------ drafts

interface ProviderDraft {
  id: string;
  kind: ProviderKind;
  label: string;
  base_url: string;
  model: string;
  api_key: string;
  use_bearer: boolean;
}

function blankDraft(): ProviderDraft {
  const p = PRESETS.openai_compat;
  return {
    id: `prov_${Math.random().toString(36).slice(2, 8)}`,
    kind: "openai_compat",
    label: p.label,
    base_url: p.base_url,
    model: p.model,
    api_key: "",
    use_bearer: true,
  };
}

function toDraft(p: ProviderView): ProviderDraft {
  return {
    id: p.id,
    kind: p.kind,
    label: p.label,
    base_url: p.base_url,
    model: p.model,
    api_key: "",
    use_bearer: p.use_bearer ?? true,
  };
}

function ProviderEditor({
  draft, onChange, onCancel, onSaved, t,
}: {
  draft: ProviderDraft;
  onChange: (d: ProviderDraft) => void;
  onCancel: () => void;
  onSaved: () => void;
  t: (k: string, v?: Record<string, string | number>) => string;
}) {
  const [showKey, setShowKey] = useState(false);
  const [saving, setSaving] = useState(false);
  const [test, setTest] = useState<{ ok: boolean; msg: string } | null>(null);
  const [testing, setTesting] = useState(false);

  function set<K extends keyof ProviderDraft>(k: K, v: ProviderDraft[K]) {
    onChange({ ...draft, [k]: v });
  }

  function applyKind(kind: ProviderKind) {
    const p = PRESETS[kind];
    onChange({ ...draft, kind, base_url: p.base_url, model: p.model, label: p.label });
  }

  async function onSave() {
    setSaving(true);
    try {
      await saveProvider({
        id: draft.id, kind: draft.kind, label: draft.label, base_url: draft.base_url,
        model: draft.model, use_bearer: draft.use_bearer,
        api_key: draft.api_key || undefined, make_active: true,
      });
      onSaved();
    } finally {
      setSaving(false);
    }
  }

  async function onTest() {
    setTesting(true);
    setTest(null);
    try {
      await saveProvider({
        id: draft.id, kind: draft.kind, label: draft.label, base_url: draft.base_url,
        model: draft.model, use_bearer: draft.use_bearer, api_key: draft.api_key || undefined,
      });
      const res = await testConnection(draft.id);
      setTest({ ok: res.ok, msg: `${res.message} (${res.latency_ms}ms)` });
    } catch (e) {
      setTest({ ok: false, msg: String(e) });
    } finally {
      setTesting(false);
    }
  }

  return (
    <div className="space-y-4 animate-fade-in">
      <div className="flex items-center justify-between">
        <span className="label">{t("settings.editProvider")}</span>
        <button className="btn-ghost !text-xs" onClick={onCancel}>
          ← {t("settings.back")}
        </button>
      </div>

      <div>
        <label className="label block mb-1.5">{t("settings.type")}</label>
        <div className="grid grid-cols-2 gap-2">
          {(Object.keys(PRESETS) as ProviderKind[]).map((k) => (
            <button
              key={k}
              onClick={() => applyKind(k)}
              className={clsx(
                "rounded-lg border px-3 py-2 text-xs text-left transition",
                draft.kind === k ? "accent-border accent-soft-bg tx" : "bd tx-muted hover:bd-strong",
              )}
            >
              {PRESETS[k].label}
            </button>
          ))}
        </div>
        <p className="mt-1.5 text-[11px] tx-faint">{PRESETS[draft.kind].hint}</p>
      </div>

      <Field label={t("settings.name")}>
        <input className="input" value={draft.label} onChange={(e) => set("label", e.target.value)} />
      </Field>

      <Field label={t("settings.baseUrl")}>
        <input className="input mono text-xs" value={draft.base_url} onChange={(e) => set("base_url", e.target.value)} />
      </Field>

      <Field label={t("settings.model")}>
        <input className="input mono text-xs" value={draft.model} onChange={(e) => set("model", e.target.value)} />
      </Field>

      <Field label={t("settings.apiKey")}>
        <div className="relative">
          <input
            className="input mono text-xs pr-10"
            type={showKey ? "text" : "password"}
            placeholder="sk-…"
            value={draft.api_key}
            onChange={(e) => set("api_key", e.target.value)}
          />
          <button
            className="absolute right-2 top-1/2 -translate-y-1/2 tx-faint hover:tx"
            onClick={() => setShowKey((v) => !v)}
            type="button"
          >
            {showKey ? <EyeOff size={15} /> : <Eye size={15} />}
          </button>
        </div>
        <p className="mt-1 text-[11px] tx-faint">{t("settings.keyHint")}</p>
      </Field>

      <label className="flex items-center gap-2 text-xs tx-muted cursor-pointer">
        <input
          type="checkbox"
          checked={draft.use_bearer}
          onChange={(e) => set("use_bearer", e.target.checked)}
          className="accent-[var(--accent)]"
        />
        {t("settings.useBearer")}
      </label>

      {test && (
        <div
          className={clsx(
            "rounded-lg px-3 py-2 text-xs flex items-center gap-2",
            test.ok ? "bg-emerald-500/10 text-emerald-300" : "bg-red-500/10 text-red-300",
          )}
        >
          {test.ok ? <CheckCircle2 size={14} /> : <XCircle size={14} />}
          {test.msg}
        </div>
      )}

      <div className="flex gap-2 pt-2">
        <button className="btn-ghost border bd-strong" onClick={onTest} disabled={testing}>
          {testing ? <Loader2 size={14} className="animate-spin" /> : <Zap size={14} />}
          {t("settings.test")}
        </button>
        <button className="btn-primary flex-1" onClick={onSave} disabled={saving}>
          {saving ? <Loader2 size={14} className="animate-spin" /> : null}
          {t("settings.save")}
        </button>
      </div>
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div>
      <label className="label block mb-1.5">{label}</label>
      {children}
    </div>
  );
}
