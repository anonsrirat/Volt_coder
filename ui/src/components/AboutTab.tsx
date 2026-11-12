import { useEffect, useState } from "react";
import { Info, RefreshCw, Loader2, Download, CheckCircle2, ExternalLink } from "lucide-react";
import { appInfo, checkUpdate, downloadUpdate, installUpdate } from "../lib/ipc";
import type { AppInfo, UpdateInfo } from "../lib/types";
import { useSettingsStore } from "../lib/store";
import { useT } from "../lib/i18n";
import { openUrl } from "@tauri-apps/plugin-opener";

export function AboutTab() {
  const t = useT();
  const { setUpdate } = useSettingsStore();
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [update, setLocalUpdate] = useState<UpdateInfo | null>(null);
  const [checking, setChecking] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [message, setMessage] = useState("");

  useEffect(() => {
    appInfo().then(setInfo).catch(() => {});
  }, []);

  async function onCheck() {
    setChecking(true);
    setMessage("");
    try {
      const u = await checkUpdate();
      setLocalUpdate(u);
      setUpdate(u.available ? u : null);
      if (!u.available) setMessage(t("about.upToDate"));
    } catch (e) {
      setMessage(String(e));
    } finally {
      setChecking(false);
    }
  }

  async function onInstall() {
    if (!update?.download_url) return;
    setInstalling(true);
    try {
      const filename = update.asset_name ?? `Volt_${update.latest}_x64-setup.exe`;
      const path = await downloadUpdate(update.download_url, filename);
      await installUpdate(path);
    } catch (e) {
      setMessage(String(e));
      setInstalling(false);
    }
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2">
        <Info size={15} className="accent-text" />
        <span className="label">{t("settings.about")}</span>
      </div>

      <div className="card flex items-center gap-4">
        <div className="grid place-items-center h-14 w-14 rounded-2xl accent-bg shadow-glow">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="white">
            <path d="M13 2L4.5 13.5H11L10 22l8.5-11.5H12L13 2z" />
          </svg>
        </div>
        <div>
          <h3 className="text-lg font-semibold tx">
            Volt <span className="accent-text text-sm">v{info?.version ?? "…"}</span>
          </h3>
          <p className="text-xs tx-muted">{t("chat.welcomeSub")}</p>
          <p className="text-[10px] tx-faint mt-0.5">Rust + Tauri 2.0 · {info?.os}</p>
        </div>
      </div>

      <div className="card space-y-2">
        <div className="flex items-center justify-between text-xs">
          <span className="tx-muted">{t("about.version")}</span>
          <span className="mono tx">{info?.version ?? "—"}</span>
        </div>
        <div className="flex items-center justify-between text-xs">
          <span className="tx-muted">{t("about.repo")}</span>
          <button
            className="mono accent-text hover:underline flex items-center gap-1"
            onClick={() =>
              info?.github_repo &&
              openUrl(`https://github.com/${info.github_repo}`).catch(() => {})
            }
          >
            {info?.github_repo ?? "—"} <ExternalLink size={11} />
          </button>
        </div>
      </div>

      {/* update section */}
      <div className="card space-y-3">
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium tx">{t("about.checkUpdates")}</span>
          <button className="btn-ghost !text-xs" onClick={onCheck} disabled={checking}>
            {checking ? <Loader2 size={13} className="animate-spin" /> : <RefreshCw size={13} />}
            {t("about.checkUpdates")}
          </button>
        </div>

        {message && (
          <div className="flex items-center gap-2 text-xs text-emerald-400">
            <CheckCircle2 size={13} /> {message}
          </div>
        )}

        {update?.available && (
          <div className="rounded-lg accent-soft-bg accent-border border p-3 space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium tx">
                {t("about.updateAvailable")}: <span className="accent-text mono">v{update.latest}</span>
              </span>
              <span className="text-[10px] tx-faint">{update.published_at?.slice(0, 10)}</span>
            </div>
            {update.notes && (
              <pre className="text-[11px] mono tx-muted max-h-32 overflow-y-auto whitespace-pre-wrap">
                {update.notes.slice(0, 800)}
              </pre>
            )}
            <button className="btn-primary w-full !text-xs" onClick={onInstall} disabled={installing}>
              {installing ? <Loader2 size={13} className="animate-spin" /> : <Download size={13} />}
              {installing ? t("about.downloading") : t("about.download")}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
