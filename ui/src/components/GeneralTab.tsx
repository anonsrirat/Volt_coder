import { useEffect, useState } from "react";
import { SlidersHorizontal, Minimize2 } from "lucide-react";
import type { Settings } from "../lib/types";
import { updateSettings } from "../lib/ipc";
import { useT } from "../lib/i18n";
import { useSettingsStore } from "../lib/store";

export function GeneralTab() {
  const t = useT();
  const { settings, setSettings } = useSettingsStore();
  const [local, setLocal] = useState<Settings | null>(settings);

  useEffect(() => setLocal(settings), [settings]);

  if (!local) return <div className="tx-faint text-sm">Loading…</div>;

  async function patch(p: Parameters<typeof updateSettings>[0]) {
    const next = await updateSettings(p);
    setSettings(next);
  }

  return (
    <div className="space-y-5">
      <div className="flex items-center gap-2">
        <SlidersHorizontal size={15} className="accent-text" />
        <span className="label">{t("settings.general")}</span>
      </div>

      {/* Approval policy */}
      <div className="card space-y-3">
        <span className="text-xs font-semibold tx">{t("general.approval")}</span>
        <Toggle
          label={t("general.approvalWrite")}
          checked={local.approval.write_file}
          onChange={(v) => {
            const approval = { ...local.approval, write_file: v };
            setLocal({ ...local, approval });
            patch({ approval });
          }}
        />
        <Toggle
          label={t("general.approvalRun")}
          checked={local.approval.run_command}
          onChange={(v) => {
            const approval = { ...local.approval, run_command: v };
            setLocal({ ...local, approval });
            patch({ approval });
          }}
        />
        <Toggle
          label={t("general.approvalRead")}
          checked={local.approval.read_file}
          onChange={(v) => {
            const approval = { ...local.approval, read_file: v };
            setLocal({ ...local, approval });
            patch({ approval });
          }}
        />
      </div>

      {/* Compaction */}
      <div className="card space-y-3">
        <div className="flex items-center gap-2">
          <Minimize2 size={13} className="accent-text" />
          <span className="text-xs font-semibold tx">{t("general.compaction")}</span>
        </div>
        <Toggle
          label={t("general.compactionEnable")}
          checked={local.compaction.enabled}
          onChange={(v) => {
            const compaction = { ...local.compaction, enabled: v };
            setLocal({ ...local, compaction });
            patch({ compaction });
          }}
        />
        <div>
          <div className="flex items-center justify-between text-xs mb-1.5">
            <span className="tx-muted">{t("general.compactionThreshold")}</span>
            <span className="mono accent-text">
              {Math.round(local.compaction.threshold * 100)}%
            </span>
          </div>
          <input
            type="range"
            min={0.3}
            max={0.95}
            step={0.05}
            value={local.compaction.threshold}
            onChange={(e) =>
              setLocal({
                ...local,
                compaction: { ...local.compaction, threshold: Number(e.target.value) },
              })
            }
            onMouseUp={() => patch({ compaction: local.compaction })}
            className="w-full accent-[var(--accent)]"
          />
        </div>
      </div>

      {/* Agent */}
      <div className="card space-y-3">
        <span className="text-xs font-semibold tx">Agent</span>
        <div>
          <div className="flex items-center justify-between text-xs mb-1.5">
            <span className="tx-muted">{t("general.maxIterations")}</span>
            <span className="mono accent-text">{local.max_iterations}</span>
          </div>
          <input
            type="range"
            min={5}
            max={60}
            step={5}
            value={local.max_iterations}
            onChange={(e) => setLocal({ ...local, max_iterations: Number(e.target.value) })}
            onMouseUp={() => patch({ max_iterations: local.max_iterations })}
            className="w-full accent-[var(--accent)]"
          />
        </div>
        <Toggle
          label={t("general.autoUpdate")}
          checked={local.auto_check_updates}
          onChange={(v) => {
            setLocal({ ...local, auto_check_updates: v });
            patch({ auto_check_updates: v });
          }}
        />
      </div>
    </div>
  );
}

function Toggle({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="flex items-center justify-between gap-3 cursor-pointer">
      <span className="text-xs tx-muted">{label}</span>
      <button
        type="button"
        onClick={() => onChange(!checked)}
        className={`relative h-5 w-9 shrink-0 rounded-full transition ${
          checked ? "accent-bg" : "surface-input border bd-strong"
        }`}
      >
        <span
          className={`absolute top-0.5 h-4 w-4 rounded-full bg-white transition-all ${
            checked ? "left-[18px]" : "left-0.5"
          }`}
        />
      </button>
    </label>
  );
}
