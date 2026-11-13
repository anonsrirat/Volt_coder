import { useEffect, useRef } from "react";
import {
  Activity,
  Brain,
  CheckCircle2,
  Circle,
  Eye,
  ListChecks,
  Loader2,
  Pencil,
  Search,
  Terminal,
  Wrench,
  X,
  XCircle,
} from "lucide-react";
import clsx from "clsx";
import { useChatStore, usePlanStore, useWorkspaceStore, type PhaseEntry } from "../lib/store";
import { useT } from "../lib/i18n";
import type { StepStatus } from "../lib/types";

/** Right-hand canvas panel: the agent's plan and/or live phase timeline. */
export function Canvas() {
  const { plan } = usePlanStore();
  const { phases } = useChatStore();
  const { toggleCanvas } = useWorkspaceStore();
  const t = useT();
  const scrollRef = useRef<HTMLDivElement>(null);

  const planTotal = plan?.steps.length ?? 0;
  const activePlan = plan && planTotal > 0 ? plan : null;
  const showPlan = !!activePlan;
  const done = plan?.steps.filter((s) => s.status === "completed").length ?? 0;
  const pct = planTotal ? Math.round((done / planTotal) * 100) : 0;

  const running = phases.find((p) => p.status === "running");
  const latest = running ?? phases[phases.length - 1];

  // Follow the timeline while the agent is working.
  useEffect(() => {
    const el = scrollRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [phases.length, plan?.steps.length]);

  return (
    <div className="w-[340px] shrink-0 flex flex-col glass border-l bd">
      <div className="h-10 shrink-0 flex items-center gap-2 px-3 border-b bd">
        {showPlan ? (
          <ListChecks size={15} className="accent-text" />
        ) : (
          <Activity size={15} className="accent-text" />
        )}
        <span className="text-xs font-semibold">
          {showPlan ? t("canvas.title") : t("canvas.activity")}
        </span>
        {showPlan && (
          <span className="text-[10px] tx-faint">
            {done}/{planTotal}
          </span>
        )}
        <div className="flex-1" />
        {latest && (
          <span
            className={clsx(
              "text-[10px] truncate max-w-[120px]",
              latest.status === "running"
                ? "accent-text"
                : latest.status === "error"
                  ? "text-red-400"
                  : "tx-faint",
            )}
          >
            {t(`phase.${latest.phase}`)}
          </span>
        )}
        <button className="btn-ghost !p-1" onClick={() => toggleCanvas(false)}>
          <X size={14} />
        </button>
      </div>

      {!showPlan && phases.length === 0 ? (
        <div className="flex-1 grid place-items-center p-6">
          <p className="text-xs tx-faint text-center leading-relaxed max-w-[220px]">
            {t("canvas.empty")}
          </p>
        </div>
      ) : (
        <div ref={scrollRef} className="flex-1 overflow-y-auto">
          {activePlan && (
            <div className="px-4 py-3 border-b bd">
              <h3 className="text-sm font-semibold tx">{activePlan.title}</h3>
              {activePlan.summary && (
                <p className="text-xs tx-muted mt-1 leading-relaxed">{activePlan.summary}</p>
              )}
              <div className="mt-3 h-1.5 rounded-full surface-input overflow-hidden">
                <div
                  className="h-full accent-bg transition-all duration-500"
                  style={{ width: `${pct}%` }}
                />
              </div>
              <span className="text-[10px] tx-faint mt-1 inline-block">{pct}%</span>
            </div>
          )}

          {activePlan && (
            <div className="p-3 space-y-1.5">
              {activePlan.steps.map((step, i) => (
                <div
                  key={step.id}
                  className={clsx(
                    "flex gap-2.5 rounded-lg border p-2.5 transition",
                    step.status === "in_progress" ? "accent-border accent-soft-bg" : "bd",
                  )}
                >
                  <StepIcon status={step.status} />
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span className="text-[10px] tx-faint mono">
                        {String(i + 1).padStart(2, "0")}
                      </span>
                      <span
                        className={clsx(
                          "text-xs font-medium",
                          step.status === "completed" ? "tx-muted line-through" : "tx",
                        )}
                      >
                        {step.title}
                      </span>
                    </div>
                    {step.description && (
                      <p className="text-[11px] tx-faint mt-0.5 leading-snug">
                        {step.description}
                      </p>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}

          {phases.length > 0 && (
            <div className={clsx("p-3 space-y-1.5", showPlan && "border-t bd")}>
              {showPlan && (
                <div className="flex items-center gap-1.5 px-1 pb-1">
                  <Activity size={12} className="accent-text" />
                  <span className="text-[10px] uppercase tracking-wide tx-faint">
                    {t("canvas.activity")}
                  </span>
                </div>
              )}
              {phases.map((p) => (
                <PhaseRow key={p.id} entry={p} />
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function PhaseRow({ entry }: { entry: PhaseEntry }) {
  const t = useT();
  return (
    <div
      className={clsx(
        "flex gap-2.5 rounded-lg border p-2 transition",
        entry.status === "running" ? "accent-border accent-soft-bg" : "bd",
      )}
    >
      <PhaseIcon entry={entry} />
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="text-xs font-medium tx">{t(`phase.${entry.phase}`)}</span>
          {entry.status === "running" && (
            <span className="text-[10px] accent-text">{t("canvas.live")}</span>
          )}
          {entry.status === "error" && (
            <span className="text-[10px] text-red-400">{t("canvas.failed")}</span>
          )}
        </div>
        {entry.detail && (
          <p className="text-[11px] tx-faint mt-0.5 font-mono truncate">{entry.detail}</p>
        )}
      </div>
    </div>
  );
}

function PhaseIcon({ entry }: { entry: PhaseEntry }) {
  const cls = "shrink-0 mt-0.5";
  if (entry.status === "running") {
    return <Loader2 size={14} className={clsx(cls, "accent-text animate-spin")} />;
  }
  if (entry.status === "error") {
    return <XCircle size={14} className={clsx(cls, "text-red-400")} />;
  }
  switch (entry.phase) {
    case "reading":
      return <Eye size={14} className={clsx(cls, "tx-faint")} />;
    case "editing":
      return <Pencil size={14} className={clsx(cls, "tx-faint")} />;
    case "running":
      return <Terminal size={14} className={clsx(cls, "tx-faint")} />;
    case "searching":
      return <Search size={14} className={clsx(cls, "tx-faint")} />;
    case "thinking":
      return <Brain size={14} className={clsx(cls, "tx-faint")} />;
    case "done":
      return <CheckCircle2 size={14} className="shrink-0 mt-0.5 text-emerald-400" />;
    default:
      return <Wrench size={14} className={clsx(cls, "tx-faint")} />;
  }
}

function StepIcon({ status }: { status: StepStatus }) {
  if (status === "completed")
    return <CheckCircle2 size={15} className="shrink-0 text-emerald-400 mt-0.5" />;
  if (status === "in_progress")
    return <Loader2 size={15} className="shrink-0 accent-text animate-spin mt-0.5" />;
  return <Circle size={15} className="shrink-0 tx-faint mt-0.5" />;
}