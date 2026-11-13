import { useEffect, useState } from "react";
import { Plus, MessageSquare, Trash2, FolderOpen } from "lucide-react";
import clsx from "clsx";
import { open } from "@tauri-apps/plugin-dialog";
import { useChatStore, useWorkspaceStore } from "../lib/store";
import { useT } from "../lib/i18n";
import {
  createSession,
  deleteSession,
  loadSession,
  listSessions,
  openProject,
} from "../lib/ipc";
import type { Session } from "../lib/types";

export function Sidebar() {
  const { sessions, sessionId, setSessions, setSessionId, setMessages, reset } =
    useChatStore();
  const { setProject } = useWorkspaceStore();
  const t = useT();
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    listSessions().then(setSessions).catch(() => {});
  }, [setSessions]);

  async function onOpenFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected !== "string") return;
    setBusy(true);
    try {
      const info = await openProject(selected);
      setProject(info.root, info.name, info.tree);
    } finally {
      setBusy(false);
    }
  }

  async function onNewChat() {
    const s = await createSession("New chat");
    setSessions(await listSessions());
    reset();
    setSessionId(s.id);
  }

  async function onSelect(s: Session) {
    reset();
    setSessionId(s.id);
    const detail = await loadSession(s.id);
    setMessages(detail.messages);
  }

  async function onDelete(e: React.MouseEvent, id: string) {
    e.stopPropagation();
    await deleteSession(id);
    setSessions(await listSessions());
    if (sessionId === id) {
      setSessionId(null);
      setMessages([]);
    }
  }

  return (
    <aside className="w-60 shrink-0 flex flex-col glass border-r bd">
      <div className="p-3 space-y-2">
        <button className="btn-primary w-full" onClick={onNewChat}>
          <Plus size={16} /> {t("sidebar.newChat")}
        </button>
        <button className="btn-ghost w-full border bd-strong" onClick={onOpenFolder} disabled={busy}>
          <FolderOpen size={16} /> {t("sidebar.openFolder")}
        </button>
      </div>

      <div className="px-3 pb-1">
        <span className="label">{t("sidebar.history")}</span>
      </div>

      <div className="flex-1 overflow-y-auto px-2 pb-3 space-y-0.5">
        {sessions.length === 0 && (
          <p className="px-2 py-4 text-xs tx-faint text-center">{t("sidebar.noConversations")}</p>
        )}
        {sessions.map((s) => (
          <button
            key={s.id}
            onClick={() => onSelect(s)}
            className={clsx(
              "group w-full flex items-center gap-2 rounded-lg px-2.5 py-2 text-sm text-left transition",
              sessionId === s.id
                ? "accent-soft-bg tx border accent-border"
                : "tx-muted hover:accent-soft-bg",
            )}
          >
            <MessageSquare size={14} className="shrink-0 opacity-70" />
            <span className="flex-1 truncate">{s.title}</span>
            <Trash2
              size={13}
              className="opacity-0 group-hover:opacity-60 hover:!opacity-100 hover:text-red-400 transition"
              onClick={(e) => onDelete(e, s.id)}
            />
          </button>
        ))}
      </div>
    </aside>
  );
}
