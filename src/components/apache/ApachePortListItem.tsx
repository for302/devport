import { useState, useEffect } from "react";
import { Globe, FolderOpen, Edit, Trash2, Lock, ExternalLink, Folder } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import type { ApachePortEntry } from "@/types";

interface ApachePortListItemProps {
  entry: ApachePortEntry;
  onEdit: (entry: ApachePortEntry) => void;
  onDelete: (entry: ApachePortEntry) => void;
}

export function ApachePortListItem({ entry, onEdit, onDelete }: ApachePortListItemProps) {
  const [siteTitle, setSiteTitle] = useState<string | null>(null);

  // Fetch site title from index file
  useEffect(() => {
    const fetchTitle = async () => {
      try {
        const title = await invoke<string | null>("get_site_title", {
          documentRoot: entry.documentRoot,
        });
        setSiteTitle(title);
      } catch (error) {
        console.error("Failed to fetch site title:", error);
      }
    };

    // Only fetch if domain is localhost or generic
    if (entry.domain === "localhost" || entry.domain === "127.0.0.1") {
      fetchTitle();
    }
  }, [entry.documentRoot, entry.domain]);

  // Display name: site title if available, otherwise domain
  const displayName = (entry.domain === "localhost" || entry.domain === "127.0.0.1")
    ? siteTitle || entry.domain
    : entry.domain;

  const handleOpenBrowser = async () => {
    try {
      await invoke("open_in_browser", { url: entry.url });
    } catch (error) {
      console.error("Failed to open browser:", error);
    }
  };

  const handleOpenFolder = async () => {
    try {
      await invoke("open_file_explorer", { path: entry.documentRoot });
    } catch (error) {
      console.error("Failed to open folder:", error);
    }
  };

  const canEdit = entry.configFile === "httpd-vhosts.conf" || entry.configFile === "httpd.conf";

  return (
    <div className="flex items-center gap-4 px-4 py-3 bg-slate-800 hover:bg-slate-750 border-b border-slate-700">
      {/* Status indicator - SSL */}
      <div className={`w-2 h-2 rounded-full flex-shrink-0 ${entry.isSsl ? "bg-green-500" : "bg-blue-500"}`} />

      {/* Framework/Language Badge */}
      <span
        className={`px-2 py-0.5 rounded text-xs font-medium flex-shrink-0 w-16 text-center truncate ${
          entry.framework === "Laravel" ? "bg-red-500/20 text-red-400" :
          entry.framework === "CodeIgniter" ? "bg-orange-500/20 text-orange-400" :
          entry.framework === "WordPress" ? "bg-sky-500/20 text-sky-400" :
          entry.framework === "Symfony" ? "bg-yellow-500/20 text-yellow-400" :
          entry.framework === "Next.js" ? "bg-white/20 text-white" :
          entry.framework === "Nuxt" ? "bg-green-500/20 text-green-400" :
          entry.framework === "Vue" ? "bg-emerald-500/20 text-emerald-400" :
          entry.framework === "React" ? "bg-cyan-500/20 text-cyan-400" :
          entry.framework === "Node.js" ? "bg-lime-500/20 text-lime-400" :
          entry.framework === "PHP" ? "bg-indigo-500/20 text-indigo-400" :
          entry.framework === "HTML" ? "bg-amber-500/20 text-amber-400" :
          "bg-slate-500/20 text-slate-400"
        }`}
        title={entry.framework}
      >
        {entry.framework}
      </span>

      {/* Name - Domain or Site Title */}
      <div className="w-32 min-w-[8rem] flex-shrink-0">
        <h3
          className={`font-medium truncate ${siteTitle ? "text-white" : "text-slate-400"}`}
          title={siteTitle ? `${siteTitle} (${entry.domain})` : entry.domain}
        >
          {displayName}
        </h3>
      </div>

      {/* Path - Document Root */}
      <div className="hidden lg:flex items-center gap-1 text-sm text-slate-400 flex-1 min-w-0">
        <Folder size={14} className="flex-shrink-0" />
        <span className="truncate" title={entry.documentRoot}>
          {entry.documentRoot}
        </span>
      </div>

      {/* Port */}
      <span className={`font-mono text-sm px-2 py-0.5 rounded w-16 text-center flex-shrink-0 ${
        entry.isSsl
          ? "bg-green-500/20 text-green-400"
          : "bg-blue-500/20 text-blue-400"
      }`}>
        {entry.port}
      </span>

      {/* Quick Actions */}
      <div className="flex items-center gap-1 flex-shrink-0 w-32 justify-center">
        <button
          onClick={handleOpenBrowser}
          className="p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-blue-400 transition-colors"
          title="Open in Browser"
        >
          <ExternalLink size={16} />
        </button>
        <button
          onClick={handleOpenFolder}
          className="p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-yellow-400 transition-colors"
          title="Open Folder"
        >
          <FolderOpen size={16} />
        </button>
        {entry.isSsl ? (
          <Lock size={16} className="p-1.5 text-green-400" />
        ) : (
          <Globe size={16} className="p-1.5 text-slate-600" />
        )}
      </div>

      {/* Divider */}
      <div className="w-px h-6 bg-slate-700 flex-shrink-0" />

      {/* Control - placeholder to match Projects */}
      <div className="flex items-center gap-1 flex-shrink-0 w-16 justify-center">
        <span className="text-xs text-slate-500">{entry.configFile.replace("httpd-", "").replace(".conf", "")}</span>
      </div>

      {/* Divider */}
      <div className="w-px h-6 bg-slate-700 flex-shrink-0" />

      {/* Edit/Delete */}
      <div className="flex items-center gap-1 flex-shrink-0 w-16 justify-center">
        <button
          onClick={() => onEdit(entry)}
          disabled={!canEdit}
          className={`p-1.5 rounded transition-colors ${
            canEdit
              ? "hover:bg-slate-700 text-slate-400 hover:text-slate-200"
              : "text-slate-600 cursor-not-allowed"
          }`}
          title={canEdit ? "Edit" : "Can only edit vhosts from httpd-vhosts.conf"}
        >
          <Edit size={16} />
        </button>
        <button
          onClick={() => onDelete(entry)}
          disabled={!canEdit}
          className={`p-1.5 rounded transition-colors ${
            canEdit
              ? "hover:bg-red-500/20 text-slate-400 hover:text-red-400"
              : "text-slate-600 cursor-not-allowed"
          }`}
          title={canEdit ? "Delete" : "Can only delete vhosts from httpd-vhosts.conf"}
        >
          <Trash2 size={16} />
        </button>
      </div>
    </div>
  );
}
