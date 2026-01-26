import { Folder, Edit, Trash2, Code2, Terminal, Globe, FolderOpen, Database } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import type { Project } from "@/types";
import { useProcessStore, useUiStore, useServiceStore } from "@/stores";
import { ProjectControls } from "./ProjectControls";

interface ProjectCardProps {
  project: Project;
}

const projectTypeColors: Record<string, string> = {
  nextjs: "bg-black text-white",
  vite: "bg-purple-500 text-white",
  react: "bg-cyan-500 text-white",
  vue: "bg-emerald-500 text-white",
  angular: "bg-red-500 text-white",
  svelte: "bg-orange-500 text-white",
  python: "bg-yellow-500 text-black",
  django: "bg-green-700 text-white",
  flask: "bg-gray-600 text-white",
  fastapi: "bg-teal-500 text-white",
  node: "bg-green-500 text-white",
  express: "bg-gray-700 text-white",
  unknown: "bg-slate-600 text-white",
};

export function ProjectCard({ project }: ProjectCardProps) {
  const isRunning = useProcessStore((state) => state.isProjectRunning(project.id));
  const healthStatus = useProcessStore((state) => state.healthStatuses[project.id]);
  const openModal = useUiStore((state) => state.openModal);
  const services = useServiceStore((state) => state.services);
  const apacheService = services.find((s) => s.id === "apache");
  const apachePort = apacheService?.port || 8080;

  const typeColor = projectTypeColors[project.projectType] || projectTypeColors.unknown;

  const handleOpenVscode = async () => {
    try {
      await invoke("open_in_vscode", { path: project.path });
    } catch (error) {
      console.error("Failed to open VS Code:", error);
    }
  };

  const handleOpenTerminal = async () => {
    try {
      await invoke("open_in_terminal", { path: project.path });
    } catch (error) {
      console.error("Failed to open terminal:", error);
    }
  };

  const handleOpenBrowser = async () => {
    try {
      await invoke("open_in_browser", { url: `http://localhost:${project.port}` });
    } catch (error) {
      console.error("Failed to open browser:", error);
    }
  };

  const handleOpenExplorer = async () => {
    try {
      await invoke("open_file_explorer", { path: project.path });
    } catch (error) {
      console.error("Failed to open file explorer:", error);
    }
  };

  const handleOpenPhpMyAdmin = async () => {
    // Try to derive database name from project name (snake_case version)
    const dbName = project.envVars?.DB_DATABASE ||
                   project.envVars?.DATABASE_NAME ||
                   project.name.toLowerCase().replace(/[^a-z0-9]/g, "_");
    const phpMyAdminUrl = `http://localhost:${apachePort}/phpmyadmin/index.php?db=${encodeURIComponent(dbName)}`;
    try {
      await invoke("open_in_browser", { url: phpMyAdminUrl });
    } catch (error) {
      console.error("Failed to open phpMyAdmin:", error);
    }
  };

  return (
    <div className="bg-slate-800 rounded-lg border border-slate-700 overflow-hidden">
      {/* Header */}
      <div className="p-4 border-b border-slate-700">
        <div className="flex items-start justify-between">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <span className={`px-2 py-0.5 rounded text-xs font-medium ${typeColor}`}>
                {project.projectType.toUpperCase()}
              </span>
              {isRunning && (
                <span className="flex items-center gap-1">
                  <span className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
                  <span className="text-xs text-green-400">Running</span>
                </span>
              )}
            </div>
            <h3 className="text-lg font-semibold text-white truncate">
              {project.name}
            </h3>
          </div>
          <div className="flex items-center gap-1 ml-2">
            <button
              onClick={() => openModal("editProject", { projectId: project.id })}
              className="p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors"
              title="Edit project"
            >
              <Edit size={16} />
            </button>
            <button
              onClick={() => openModal("confirmDelete", { projectId: project.id })}
              className="p-1.5 rounded hover:bg-red-500/20 text-slate-400 hover:text-red-400 transition-colors"
              title="Delete project"
            >
              <Trash2 size={16} />
            </button>
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="p-4 space-y-3">
        {/* Path */}
        <div className="flex items-center gap-2 text-sm text-slate-400">
          <Folder size={14} />
          <span className="truncate" title={project.path}>
            {project.path}
          </span>
        </div>

        {/* Port */}
        <div className="flex items-center justify-between">
          <span className="text-sm text-slate-400">Port</span>
          <span className="font-mono text-white bg-slate-700 px-2 py-0.5 rounded">
            {project.port}
          </span>
        </div>

        {/* Health Status */}
        {isRunning && healthStatus && (
          <div className="flex items-center justify-between">
            <span className="text-sm text-slate-400">Health</span>
            <span
              className={`text-sm font-medium ${
                healthStatus.isHealthy ? "text-green-400" : "text-red-400"
              }`}
            >
              {healthStatus.isHealthy ? "Healthy" : "Unhealthy"}
              {healthStatus.responseTimeMs && (
                <span className="text-slate-500 ml-1">
                  ({healthStatus.responseTimeMs}ms)
                </span>
              )}
            </span>
          </div>
        )}

        {/* Quick Actions */}
        <div className="flex items-center gap-2 pt-2 border-t border-slate-700">
          <button
            onClick={handleOpenVscode}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded text-sm text-slate-400 hover:text-slate-200 hover:bg-slate-700 transition-colors"
            title="Open in VS Code"
          >
            <Code2 size={14} />
            <span>VS Code</span>
          </button>
          <button
            onClick={handleOpenTerminal}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded text-sm text-slate-400 hover:text-slate-200 hover:bg-slate-700 transition-colors"
            title="Open Terminal"
          >
            <Terminal size={14} />
            <span>Terminal</span>
          </button>
          <button
            onClick={handleOpenBrowser}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded text-sm text-slate-400 hover:text-slate-200 hover:bg-slate-700 transition-colors"
            title="Open in Browser"
          >
            <Globe size={14} />
            <span>Browser</span>
          </button>
          <button
            onClick={handleOpenExplorer}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded text-sm text-slate-400 hover:text-slate-200 hover:bg-slate-700 transition-colors"
            title="Open in File Explorer"
          >
            <FolderOpen size={14} />
            <span>Explorer</span>
          </button>
          <button
            onClick={handleOpenPhpMyAdmin}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded text-sm text-orange-400 hover:text-orange-300 hover:bg-slate-700 transition-colors"
            title="Open project database in phpMyAdmin"
          >
            <Database size={14} />
            <span>DB</span>
          </button>
        </div>
      </div>

      {/* Controls */}
      <div className="px-4 pb-4">
        <ProjectControls project={project} />
      </div>
    </div>
  );
}
