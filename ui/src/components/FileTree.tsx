import { useState } from "react";
import { ChevronRight, File, Folder, FolderOpen, RefreshCw } from "lucide-react";
import clsx from "clsx";
import { useWorkspaceStore } from "../lib/store";
import { useT } from "../lib/i18n";
import { readFile, listDir, currentProject, openProject } from "../lib/ipc";
import type { FileNode } from "../lib/types";

export function FileTree() {
  const { name, tree, setProject, root, setActiveFile } = useWorkspaceStore();
  const t = useT();

  async function onRefresh() {
    if (!root) return;
    const info = await currentProject();
    if (!info) return;
    const full = await openProject(info.root);
    setProject(full.root, full.name, full.tree);
  }

  return (
    <div className="w-64 shrink-0 flex flex-col glass border-r bd">
      <div className="h-10 shrink-0 flex items-center justify-between px-3 border-b bd">
        <span className="text-xs font-medium tx truncate">{name ?? t("explorer.title")}</span>
        {root && (
          <button className="btn-ghost !p-1" onClick={onRefresh} title="Refresh">
            <RefreshCw size={13} />
          </button>
        )}
      </div>

      <div className="flex-1 overflow-y-auto py-1.5 text-sm">
        {tree.length === 0 ? (
          <p className="px-3 py-6 text-xs tx-faint text-center leading-relaxed">
            {t("explorer.empty")}
          </p>
        ) : (
          tree.map((node) => (
            <TreeNode key={node.path} node={node} depth={0} onOpen={setActiveFile} />
          ))
        )}
      </div>
    </div>
  );
}

function TreeNode({
  node,
  depth,
  onOpen,
}: {
  node: FileNode;
  depth: number;
  onOpen: (path: string, content: string, lang: string) => void;
}) {
  const [open, setOpen] = useState(depth < 1);
  const [children, setChildren] = useState<FileNode[] | null>(node.children ?? null);

  async function toggle() {
    if (!node.is_dir) return;
    if (!children) {
      const kids = await listDir(node.path);
      setChildren(kids);
    }
    setOpen((v) => !v);
  }

  async function openFile() {
    if (node.is_dir) return;
    const file = await readFile(node.path);
    onOpen(file.path, file.content, file.language);
  }

  const pad = { paddingLeft: `${8 + depth * 12}px` };

  return (
    <div>
      <button
        onClick={node.is_dir ? toggle : openFile}
        style={pad}
        className="group w-full flex items-center gap-1.5 pr-2 py-[3px] text-left rounded-sm transition hover:accent-soft-bg tx-muted"
      >
        {node.is_dir ? (
          <>
            <ChevronRight
              size={13}
              className={clsx("shrink-0 opacity-60 transition-transform", open && "rotate-90")}
            />
            {open ? (
              <FolderOpen size={14} className="shrink-0 accent-text/80" />
            ) : (
              <Folder size={14} className="shrink-0 accent-text/80" />
            )}
          </>
        ) : (
          <>
            <span className="w-[13px]" />
            <File size={14} className="shrink-0 opacity-50" />
          </>
        )}
        <span className="truncate text-[13px]">{node.name}</span>
      </button>

      {node.is_dir && open && children && (
        <div>
          {children.map((child) => (
            <TreeNode key={child.path} node={child} depth={depth + 1} onOpen={onOpen} />
          ))}
        </div>
      )}
    </div>
  );
}
