import { ShieldAlert, Check, X } from "lucide-react";
import { useState } from "react";
import { useChatStore } from "../lib/store";
import { useT } from "../lib/i18n";
import { respondApproval } from "../lib/ipc";
import type { ApprovalRequestEvent } from "../lib/types";

/** Asks the user to approve a privileged tool call (write_file / run_command). */
export function ApprovalCard({ approval }: { approval: ApprovalRequestEvent }) {
  const { setApproval } = useChatStore();
  const t = useT();
  const [acting, setActing] = useState(false);

  async function decide(approved: boolean) {
    setActing(true);
    try {
      await respondApproval(approval.tool_call_id, approved);
      setApproval(null);
    } finally {
      setActing(false);
    }
  }

  const isWrite = approval.tool_name === "write_file" || approval.tool_name === "apply_patch";
  const isPatch = approval.tool_name === "apply_patch";
  const preview = approval.preview?.trim();

  return (
    <div className="my-4 rounded-xl border border-amber-500/30 bg-amber-500/[0.07] overflow-hidden animate-fade-in">
      <div className="flex items-center gap-2 px-4 py-2.5 bg-amber-500/10 border-b border-amber-500/20">
        <ShieldAlert size={16} className="text-amber-400" />
        <span className="text-sm font-medium text-amber-200">{t("approval.required")}</span>
        <span className="text-xs text-amber-300/70 font-mono ml-auto">{approval.tool_name}</span>
      </div>

      <div className="p-4 space-y-3">
        <p className="text-xs tx-muted">
          {isWrite ? t("approval.wantModify") : t("approval.wantRun")}
        </p>

        {preview ? (
          <div className="space-y-2">
            {isPatch && (approval.arguments.path as string) && (
              <div className="text-xs tx font-mono">📄 {approval.arguments.path as string}</div>
            )}
            <pre
              className={
                "text-[11px] font-mono surface-input rounded-lg p-3 max-h-64 overflow-auto tx-muted whitespace-pre"
              }
            >
              {preview}
            </pre>
          </div>
        ) : isWrite ? (
          <div className="space-y-2">
            {(approval.arguments.path as string) && (
              <div className="text-xs tx font-mono">📄 {approval.arguments.path as string}</div>
            )}
            {(approval.arguments.content as string) && (
              <pre className="text-[11px] font-mono surface-input rounded-lg p-3 max-h-48 overflow-auto tx-muted">
                {(approval.arguments.content as string).slice(0, 2000)}
              </pre>
            )}
            {(approval.arguments.replace as string) && (
              <div className="text-[11px] font-mono space-y-1">
                <pre className="bg-red-500/10 text-red-300 rounded p-2 overflow-auto max-h-32">
                  − {(approval.arguments.search as string)?.slice(0, 800)}
                </pre>
                <pre className="bg-emerald-500/10 text-emerald-300 rounded p-2 overflow-auto max-h-32">
                  + {(approval.arguments.replace as string)?.slice(0, 800)}
                </pre>
              </div>
            )}
          </div>
        ) : (
          <pre className="text-xs font-mono surface-input rounded-lg p-3 text-emerald-300 overflow-x-auto">
            $ {approval.arguments.command as string}
          </pre>
        )}

        <div className="flex gap-2 pt-1">
          <button
            className="btn bg-emerald-600 text-white hover:bg-emerald-500 flex-1"
            onClick={() => decide(true)}
            disabled={acting}
          >
            <Check size={15} /> {t("approval.approve")}
          </button>
          <button className="btn-ghost border bd-strong flex-1" onClick={() => decide(false)} disabled={acting}>
            <X size={15} /> {t("approval.deny")}
          </button>
        </div>
      </div>
    </div>
  );
}
