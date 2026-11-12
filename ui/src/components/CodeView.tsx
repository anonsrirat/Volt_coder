import { FileCode2 } from "lucide-react";
import { useWorkspaceStore } from "../lib/store";

export function CodeView() {
  const { activeFile, fileContent, language } = useWorkspaceStore();

  if (!activeFile || fileContent === null) {
    return (
      <div className="hidden xl:flex w-[380px] shrink-0 flex-col glass border-l bd">
        <EmptyState />
      </div>
    );
  }

  const lines = fileContent.split("\n");

  return (
    <div className="hidden xl:flex w-[440px] shrink-0 flex-col glass border-l bd">
      <div className="h-10 shrink-0 flex items-center gap-2 px-3 border-b bd">
        <FileCode2 size={15} className="accent-text" />
        <span className="text-xs font-medium tx truncate flex-1">
          {activeFile.split(/[\\/]/).pop()}
        </span>
        <span className="text-[10px] uppercase tracking-wide tx-faint">{language}</span>
      </div>

      <div className="flex-1 overflow-auto font-mono text-[12.5px] leading-[1.6]">
        <table className="w-full border-collapse">
          <tbody>
            {lines.map((line, i) => (
              <tr key={i} className="hover:accent-soft-bg">
                <td className="w-10 pr-3 text-right select-none tx-faint align-top">{i + 1}</td>
                <td className="pr-4 whitespace-pre tx align-top">{line || " "}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function EmptyState() {
  return (
    <div className="flex-1 grid place-items-center p-8">
      <div className="text-center space-y-2 max-w-[240px]">
        <FileCode2 size={28} className="mx-auto tx-faint" />
        <p className="text-xs tx-faint leading-relaxed">
          Select a file from the explorer to preview it here.
        </p>
      </div>
    </div>
  );
}
