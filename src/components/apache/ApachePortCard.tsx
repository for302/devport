import {
  ExternalLink,
  Pencil,
  Trash2,
  FolderOpen,
  Lock,
  Globe,
} from "lucide-react";
import type { ApachePortEntry } from "@/types";

interface ApachePortCardProps {
  entry: ApachePortEntry;
  onEdit: (entry: ApachePortEntry) => void;
  onDelete: (entry: ApachePortEntry) => void;
  onOpenFolder: (path: string) => void;
}

export function ApachePortCard({
  entry,
  onEdit,
  onDelete,
  onOpenFolder,
}: ApachePortCardProps) {
  const handleOpenBrowser = () => {
    window.open(entry.url, "_blank", "noopener,noreferrer");
  };

  return (
    <div className="bg-slate-800 border border-slate-700 rounded-lg p-4 hover:border-slate-600 transition-colors">
      <div className="flex items-start justify-between gap-4">
        <div className="flex items-start gap-3 flex-1 min-w-0">
          {/* Port Badge */}
          <div
            className={`flex-shrink-0 px-3 py-1.5 rounded-lg font-mono text-sm font-medium ${
              entry.isSsl
                ? "bg-green-500/20 text-green-400 border border-green-500/30"
                : "bg-blue-500/20 text-blue-400 border border-blue-500/30"
            }`}
          >
            {entry.port}
          </div>

          {/* Info */}
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2">
              {entry.isSsl ? (
                <Lock size={14} className="text-green-400" />
              ) : (
                <Globe size={14} className="text-blue-400" />
              )}
              <span className="text-white font-medium truncate">
                {entry.domain}
              </span>
              {entry.isSsl && (
                <span className="px-1.5 py-0.5 text-xs bg-green-500/20 text-green-400 rounded">
                  SSL
                </span>
              )}
            </div>

            <div className="mt-1 text-sm text-slate-400 truncate" title={entry.documentRoot}>
              {entry.documentRoot}
            </div>

            {entry.serverAlias.length > 0 && (
              <div className="mt-1 flex flex-wrap gap-1">
                {entry.serverAlias.map((alias, idx) => (
                  <span
                    key={idx}
                    className="px-1.5 py-0.5 text-xs bg-slate-700 text-slate-400 rounded"
                  >
                    {alias}
                  </span>
                ))}
              </div>
            )}

            <div className="mt-1 text-xs text-slate-500">
              Config: {entry.configFile}
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="flex items-center gap-1 flex-shrink-0">
          <button
            onClick={handleOpenBrowser}
            className="p-2 text-slate-400 hover:text-blue-400 hover:bg-slate-700 rounded transition-colors"
            title="Open in browser"
          >
            <ExternalLink size={16} />
          </button>
          <button
            onClick={() => onOpenFolder(entry.documentRoot)}
            className="p-2 text-slate-400 hover:text-yellow-400 hover:bg-slate-700 rounded transition-colors"
            title="Open folder"
          >
            <FolderOpen size={16} />
          </button>
          {entry.configFile === "httpd-vhosts.conf" && (
            <>
              <button
                onClick={() => onEdit(entry)}
                className="p-2 text-slate-400 hover:text-green-400 hover:bg-slate-700 rounded transition-colors"
                title="Edit"
              >
                <Pencil size={16} />
              </button>
              <button
                onClick={() => onDelete(entry)}
                className="p-2 text-slate-400 hover:text-red-400 hover:bg-slate-700 rounded transition-colors"
                title="Delete"
              >
                <Trash2 size={16} />
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
