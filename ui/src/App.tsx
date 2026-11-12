import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Sidebar } from "./components/Sidebar";
import { FileTree } from "./components/FileTree";
import { CodeView } from "./components/CodeView";
import { ChatPanel } from "./components/ChatPanel";
import { Canvas } from "./components/Canvas";
import { SettingsModal } from "./components/SettingsModal";
import { UpdateBanner } from "./components/UpdateBanner";
import { useSettingsStore, useWorkspaceStore, useChatStore } from "./lib/store";
import { subscribeToAgentEvents } from "./lib/events";
import { useT } from "./lib/i18n";
import { applyTheme, watchSystemTheme, type ThemeMode } from "./lib/theme";
import {
  currentProject,
  listProviders,
  listSessions,
  getSettings,
  checkUpdate,
  contextTokens,
} from "./lib/ipc";

export default function App() {
  const { setProviders, setActiveProvider, settingsOpen, setSettings, setUpdate, theme, accent } =
    useSettingsStore();
  const { setProject, showCanvas } = useWorkspaceStore();
  const { setSessions, setSessionId } = useChatStore();
  const t = useT();
  const [booting, setBooting] = useState(true);
  const [sidebarOpen, setSidebarOpen] = useState(true);

  // Keep the context-usage meter in sync with the open conversation.
  const messageCount = useChatStore((s) => s.messages.length);
  useEffect(() => {
    refreshContext();
  }, [messageCount]);

  // Apply theme whenever it changes.
  useEffect(() => {
    applyTheme(theme as ThemeMode, accent);
    return watchSystemTheme(
      () => useSettingsStore.getState().theme as ThemeMode,
      () => useSettingsStore.getState().accent,
    );
  }, [theme, accent]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let unlistenDrop: (() => void) | undefined;

    (async () => {
      try {
        const settings = await getSettings();
        setSettings(settings);
        applyTheme(settings.theme as ThemeMode, settings.accent);
      } catch {
        /* ignore */
      }

      const providers = await listProviders();
      setProviders(providers);
      const active = providers.find((p) => p.is_active);
      if (active) setActiveProvider(active.id);

      const sessions = await listSessions();
      setSessions(sessions);
      if (sessions[0]) setSessionId(sessions[0].id);

      try {
        const project = await currentProject();
        if (project) setProject(project.root, project.name, project.tree);
      } catch {
        /* ignore */
      }

      unlisten = await subscribeToAgentEvents();
      setBooting(false);

      // Seed the composer's context-usage meter.
      refreshContext();

      // Auto-check updates on launch.
      if (useSettingsStore.getState().settings?.auto_check_updates) {
        checkUpdate()
          .then((u) => {
            if (u.available) setUpdate(u);
          })
          .catch(() => {});
      }
    })();

    listen("tauri://drag-drop", () => {}).then((u) => (unlistenDrop = u));

    return () => {
      unlisten?.();
      unlistenDrop?.();
    };
  }, [setProviders, setActiveProvider, setSessions, setSessionId, setProject, setSettings, setUpdate]);

  return (
    <div className="h-full w-full flex flex-col overflow-hidden">
      {/* Titlebar */}
      <header className="h-11 shrink-0 flex items-center gap-3 px-3 glass border-b bd z-20">
        <button
          className="btn-ghost !px-2"
          onClick={() => setSidebarOpen((v) => !v)}
          title="Toggle sidebar"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <rect x="3" y="4" width="18" height="16" rx="2" />
            <line x1="9" y1="4" x2="9" y2="20" />
          </svg>
        </button>

        <div className="flex items-center gap-2 select-none">
          <BoltLogo />
          <span className="font-semibold tracking-tight tx">Volt</span>
          <span className="text-[10px] uppercase tracking-widest accent-text/80 font-medium">
            {t("app.tagline")}
          </span>
        </div>

        <div className="flex-1" />
        <PlanButton />
        <ActiveProviderBadge />
      </header>

      <UpdateBanner />

      {/* Body */}
      <div className="flex-1 flex min-h-0">
        {sidebarOpen && <Sidebar />}
        <FileTree />
        <ChatPanel />
        {showCanvas ? <Canvas /> : <CodeView />}
      </div>

      {settingsOpen && <SettingsModal />}
      {booting && <BootOverlay label={t("app.starting")} />}
    </div>
  );
}

function refreshContext() {
  const { messages } = useChatStore.getState();
  const providerId = useSettingsStore.getState().activeProvider;
  contextTokens(messages, providerId)
    .then((info) => useChatStore.getState().setContextInfo(info))
    .catch(() => useChatStore.getState().setContextInfo(null));
}

function PlanButton() {
  const { showCanvas, toggleCanvas } = useWorkspaceStore();
  const t = useT();
  return (
    <button
      className={showCanvas ? "btn-ghost !py-1 text-xs accent-text" : "btn-ghost !py-1 text-xs"}
      onClick={() => toggleCanvas()}
      title={t("canvas.title")}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M9 11l3 3L22 4" />
        <path d="M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11" />
      </svg>
      {t("canvas.title")}
    </button>
  );
}

function ActiveProviderBadge() {
  const { providers, activeProvider, toggleSettings } = useSettingsStore();
  const active = providers.find((p) => p.id === activeProvider);
  const t = useT();
  return (
    <button
      onClick={() => toggleSettings(true, "providers")}
      className="btn-ghost !py-1 text-xs gap-2 max-w-[260px]"
      title={t("settings.providers")}
    >
      <span
        className={`h-1.5 w-1.5 rounded-full ${
          active ? (active.has_key ? "bg-emerald-400" : "bg-amber-400") : "bg-slate-500"
        }`}
      />
      {active ? (
        <span className="truncate">
          {active.label} · <span className="tx-faint">{active.model}</span>
        </span>
      ) : (
        <span className="tx-faint">{t("chat.noProvider")}</span>
      )}
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M6 9l6 6 6-6" />
      </svg>
    </button>
  );
}

function BoltLogo() {
  return (
    <div className="grid place-items-center h-6 w-6 rounded-md accent-bg shadow-glow-sm">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="white">
        <path d="M13 2L4.5 13.5H11L10 22l8.5-11.5H12L13 2z" />
      </svg>
    </div>
  );
}

function BootOverlay({ label }: { label: string }) {
  return (
    <div className="fixed inset-0 z-50 grid place-items-center" style={{ background: "var(--bg)" }}>
      <div className="flex flex-col items-center gap-3 animate-fade-in">
        <BoltLogo />
        <span className="text-sm tx-faint">{label}</span>
      </div>
    </div>
  );
}
