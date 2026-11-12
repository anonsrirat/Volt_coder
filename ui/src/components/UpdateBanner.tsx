import { useState } from "react";
import { Download, Sparkles, X, Loader2 } from "lucide-react";
import { useSettingsStore } from "../lib/store";
import { useT } from "../lib/i18n";
import { downloadUpdate, installUpdate } from "../lib/ipc";

/** Slim banner that appears when a newer release is found on GitHub. */
export function UpdateBanner() {
  const { update, updateDismissed, dismissUpdate, toggleSettings } = useSettingsStore();
  const t = useT();
  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState("");

  if (!update?.available || updateDismissed) return null;

  async function onInstall() {
    if (!update?.download_url) return;
    setBusy(true);
    setProgress(t("about.downloading"));
    try {
      const filename = update.asset_name ?? `Volt_${update.latest}_x64-setup.exe`;
      const path = await downloadUpdate(update.download_url, filename);
      setProgress("");
      await installUpdate(path);
    } catch (e) {
      setProgress(String(e));
      setBusy(false);
    }
  }

  return (
    <div className="shrink-0 flex items-center gap-3 px-4 py-2 text-sm accent-bg/10 border-b accent-border"
      style={{ background: "var(--accent-soft)" }}>
      <Sparkles size={15} className="accent-text shrink-0" />
      <span className="tx text-xs flex-1">
        {t("update.newVersion", { version: update.latest })}
        {progress && <span className="tx-muted ml-2">· {progress}</span>}
      </span>
      <button
        className="btn-primary !py-1 !text-xs"
        onClick={onInstall}
        disabled={busy}
      >
        {busy ? <Loader2 size={13} className="animate-spin" /> : <Download size={13} />}
        {t("about.download")}
      </button>
      <button
        className="btn-ghost !py-1 !text-xs"
        onClick={() => toggleSettings(true, "about")}
      >
        {t("update.view")}
      </button>
      <button className="btn-ghost !p-1" onClick={dismissUpdate} title={t("update.later")}>
        <X size={14} />
      </button>
    </div>
  );
}
