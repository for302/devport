import { useState, useEffect } from "react";
import { Globe, FolderOpen, Edit, Trash2, ExternalLink, Folder, AlertTriangle, Terminal, Github } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import type { ApachePortEntry } from "@/types";

interface ApachePortListItemProps {
  entry: ApachePortEntry;
  onEdit: (entry: ApachePortEntry) => void;
  onDelete: (entry: ApachePortEntry) => void;
}

export function ApachePortListItem({ entry, onEdit, onDelete }: ApachePortListItemProps) {
  const [siteTitle, setSiteTitle] = useState<string | null>(null);
  const [folderExists, setFolderExists] = useState<boolean | null>(null);

  // Fetch site title from index file only if no name is set and domain is localhost
  useEffect(() => {
    // Skip fetching if we have a user-defined name
    if (entry.name) {
      return;
    }

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
  }, [entry.documentRoot, entry.domain, entry.name]);

  // Check if document root folder exists
  useEffect(() => {
    const checkFolder = async () => {
      try {
        const exists = await invoke<boolean>("check_document_root", {
          path: entry.documentRoot,
        });
        setFolderExists(exists);
      } catch {
        setFolderExists(null);
      }
    };
    checkFolder();
  }, [entry.documentRoot]);

  // Display name priority: user-defined name > site title > domain
  const displayName = entry.name || (
    (entry.domain === "localhost" || entry.domain === "127.0.0.1")
      ? siteTitle || entry.domain
      : entry.domain
  );
  const hasCustomName = !!entry.name;

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

  const handleOpenTerminal = async () => {
    try {
      await invoke("open_in_terminal", { path: entry.documentRoot });
    } catch (error) {
      console.error("Failed to open terminal:", error);
    }
  };

  const handleOpenServiceUrl = async () => {
    if (!entry.serviceUrl) return;
    try {
      await invoke("open_in_browser", { url: entry.serviceUrl });
    } catch (error) {
      console.error("Failed to open service URL:", error);
    }
  };

  const handleOpenGithub = async () => {
    if (!entry.githubUrl) return;
    try {
      await invoke("open_in_browser", { url: entry.githubUrl });
    } catch (error) {
      console.error("Failed to open GitHub:", error);
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

      {/* Name - User-defined name, Site Title, or Domain */}
      <div className="w-32 min-w-[8rem] flex-shrink-0">
        <h3
          className={`font-medium truncate ${hasCustomName || siteTitle ? "text-white" : "text-slate-400"}`}
          title={hasCustomName ? `${entry.name} (${entry.domain})` : (siteTitle ? `${siteTitle} (${entry.domain})` : entry.domain)}
        >
          {displayName}
        </h3>
      </div>

      {/* Path - Document Root */}
      <div className={`hidden lg:flex items-center gap-1 text-sm flex-1 min-w-0 ${folderExists === false ? "text-red-400" : "text-slate-400"}`}>
        {folderExists === false ? (
          <AlertTriangle size={14} className="flex-shrink-0 text-red-400" />
        ) : (
          <Folder size={14} className="flex-shrink-0" />
        )}
        <span className="truncate" title={entry.documentRoot}>
          {entry.documentRoot}
        </span>
        {folderExists === false && (
          <span className="flex-shrink-0 text-xs">(폴더 없음)</span>
        )}
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
      <div className="flex items-center gap-1 flex-shrink-0 w-48 justify-center">
        <button
          onClick={handleOpenFolder}
          disabled={folderExists === false}
          className={`p-1.5 rounded transition-colors ${
            folderExists === false
              ? "text-slate-600 cursor-not-allowed"
              : "hover:bg-slate-700 text-slate-400 hover:text-yellow-400"
          }`}
          title={folderExists === false ? "폴더가 존재하지 않습니다" : "Open Folder"}
        >
          <FolderOpen size={16} />
        </button>
        <button
          onClick={handleOpenTerminal}
          disabled={folderExists === false}
          className={`p-1.5 rounded transition-colors ${
            folderExists === false
              ? "text-slate-600 cursor-not-allowed"
              : "hover:bg-slate-700 text-slate-400 hover:text-green-400"
          }`}
          title={folderExists === false ? "폴더가 존재하지 않습니다" : "Open Terminal"}
        >
          <Terminal size={16} />
        </button>
        <button
          onClick={handleOpenBrowser}
          className="p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-blue-400 transition-colors"
          title="Open in Browser"
        >
          <ExternalLink size={16} />
        </button>
        <button
          onClick={handleOpenServiceUrl}
          disabled={!entry.serviceUrl}
          className={`p-1.5 rounded transition-colors ${
            entry.serviceUrl
              ? "hover:bg-slate-700 text-slate-400 hover:text-purple-400"
              : "text-slate-600 cursor-not-allowed"
          }`}
          title={entry.serviceUrl ? `Open Service: ${entry.serviceUrl}` : "Service URL 미설정"}
        >
          <Globe size={16} />
        </button>
        <button
          onClick={handleOpenGithub}
          disabled={!entry.githubUrl}
          className={`p-1.5 rounded transition-colors ${
            entry.githubUrl
              ? "hover:bg-slate-700 text-slate-400 hover:text-white"
              : "text-slate-600 cursor-not-allowed"
          }`}
          title={entry.githubUrl ? `Open GitHub: ${entry.githubUrl}` : "GitHub 저장소 없음"}
        >
          <Github size={16} />
        </button>
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
