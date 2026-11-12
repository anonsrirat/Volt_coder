import { CheckCircle2, Circle, Loader2, ListChecks, X } from "lucide-react";
import clsx from "clsx";
import { usePlanStore, useWorkspaceStore } from "../lib/store";
import { useT } from "../lib/i18n";
import type { StepStatus } from "../lib/types";

/** Right-hand canvas panel that renders the agent's live execution plan. */
export function Canvas() {
  const { plan } = usePlanStore();
  const { toggleCanvas } = useWorkspaceStore();
  const t = useT();

  const done = plan?.steps.filter((s) => s.status === "completed").length ?? 0;
  const total = plan?.steps.length ?? 0;
  const pct = total ? Math.round((done / total) * 100) : 0;

  return (
    <div className="w-[340px] shrink-0 flex flex-col glass border-l bd">
      <div className="h-10 shrink-0 flex items-center gap-2 px-3 border-b bd">
        <ListChecks size={15} className="accent-text" />
        <span className="text-xs font-semibold">{t("canvas.title")}</span>
        {total > 0 && (
          <span className="text-[10px] tx-faint">
            {done}/{total}
          </span>
        )}
        <div className="flex-1" />
        <button className="btn-ghost !p-1" onClick={() => toggleCanvas(false)}>
          <X size={14} />
        </button>
      </div>

      {!plan || total === 0 ? (
        <div className="flex-1 grid place-items-center p-6">
          <p className="text-xs tx-faint text-center leading-relaxed max-w-[220px]">
            {t("canvas.empty")}
          </p>
        </div>
      ) : (
        <div className="flex-1 overflow-y-auto">
          {/* header */}
          <div className="px-4 py-3 border-b bd">
            <h3 className="text-sm font-semibold tx">{plan.title}</h3>
            {plan.summary && (
              <p className="text-xs tx-muted mt-1 leading-relaxed">{plan.summary}</p>
            )}
            {/* progress bar */}
            <div className="mt-3 h-1.5 rounded-full surface-input overflow-hidden">
              <div
                className="h-full accent-bg transition-all duration-500"
                style={{ width: `${pct}%` }}
              />
            </div>
            <span className="text-[10px] tx-faint mt-1 inline-block">{pct}%</span>
          </div>

          {/* steps */}
          <div className="p-3 space-y-1.5">
            {plan.steps.map((step, i) => (
              <div
                key={step.id}
                className={clsx(
                  "flex gap-2.5 rounded-lg border p-2.5 transition",
                  step.status === "in_progress"
                    ? "accent-border accent-soft-bg"
                    : "bd",
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
        </div>
      )}
    </div>
  );
}

function StepIcon({ status }: { status: StepStatus }) {
  if (status === "completed")
    return <CheckCircle2 size={15} className="shrink-0 text-emerald-400 mt-0.5" />;
  if (status === "in_progress")
    return <Loader2 size={15} className="shrink-0 accent-text animate-spin mt-0.5" />;
  return <Circle size={15} className="shrink-0 tx-faint mt-0.5" />;
}
