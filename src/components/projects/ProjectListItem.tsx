import { Play, Square, RotateCcw, Folder, Edit, Trash2, Code2, Terminal, Globe, FolderOpen, Database, Loader2 } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import type { Project } from "@/types";
import { useProcessStore, useUiStore, useServiceStore, useActivityLogStore } from "@/stores";

interface ProjectListItemProps {
  project: Project;
}

const projectTypeColors: Record<string, string> = {
  tauri: "bg-yellow-500 text-black",
  electron: "bg-blue-400 text-white",
  nextjs: "bg-black text-white",
  vite: "bg-purple-500 text-white",
  react: "bg-cyan-500 text-white",
  vue: "bg-emerald-500 text-white",
  angular: "bg-red-500 text-white",
  svelte: "bg-orange-500 text-white",
  python: "bg-yellow-600 text-black",
  django: "bg-green-700 text-white",
  flask: "bg-gray-600 text-white",
  fastapi: "bg-teal-500 text-white",
  node: "bg-green-500 text-white",
  express: "bg-gray-700 text-white",
  unknown: "bg-slate-600 text-white",
};

export function ProjectListItem({ project }: ProjectListItemProps) {
  const isRunning = useProcessStore((state) => state.isProjectRunning(project.id));
  const isLoading = useProcessStore((state) => state.isLoading[project.id] || false);
  const startProject = useProcessStore((state) => state.startProject);
  const stopProject = useProcessStore((state) => state.stopProject);
  const restartProject = useProcessStore((state) => state.restartProject);
  const openModal = useUiStore((state) => state.openModal);
  const services = useServiceStore((state) => state.services);
  const addLog = useActivityLogStore((state) => state.addLog);
  const apacheService = services.find((s) => s.id === "apache");
  const apachePort = apacheService?.port || 8080;

  const typeColor = projectTypeColors[project.projectType] || projectTypeColors.unknown;

  const handleStart = async () => {
    addLog(project.name, "Starting project...", "info");
    try {
      await startProject(project.id);
      addLog(project.name, `Started on port ${project.port}`, "success");
    } catch (err) {
      addLog(project.name, `Failed to start: ${err}`, "error");
    }
  };

  const handleStop = async () => {
    addLog(project.name, "Stopping project...", "info");
    try {
      await stopProject(project.id);
      addLog(project.name, "Project stopped", "success");
    } catch (err) {
      addLog(project.name, `Failed to stop: ${err}`, "error");
    }
  };

  const handleRestart = async () => {
    addLog(project.name, "Restarting project...", "info");
    try {
      await restartProject(project.id);
      addLog(project.name, `Restarted on port ${project.port}`, "success");
    } catch (err) {
      addLog(project.name, `Failed to restart: ${err}`, "error");
    }
  };

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
    <div className="relative">
      <div className="flex items-center gap-4 px-4 py-3 bg-slate-800 hover:bg-slate-750 border-b border-slate-700">
        {/* Status indicator */}
        <div className={`w-2 h-2 rounded-full flex-shrink-0 ${isLoading ? "bg-yellow-500 animate-pulse" : isRunning ? "bg-green-500 animate-pulse" : "bg-slate-500"}`} />

      {/* Project Type Badge */}
      <span className={`px-2 py-0.5 rounded text-xs font-medium flex-shrink-0 w-16 text-center ${typeColor}`}>
        {project.projectType.toUpperCase().slice(0, 6)}
      </span>

      {/* Project Name - fixed width, no shrink */}
      <div className="w-32 min-w-[8rem] flex-shrink-0">
        <h3 className="font-medium text-white truncate" title={project.name}>{project.name}</h3>
      </div>

      {/* Path - flexible, shrinks first */}
      <div className="hidden lg:flex items-center gap-1 text-sm text-slate-400 flex-1 min-w-0">
        <Folder size={14} className="flex-shrink-0" />
        <span className="truncate" title={project.path}>{project.path}</span>
      </div>

      {/* Port */}
      <span className="font-mono text-sm text-white bg-slate-700 px-2 py-0.5 rounded w-16 text-center flex-shrink-0">
        {project.port}
      </span>

      {/* Quick Actions */}
      <div className="flex items-center gap-1 flex-shrink-0">
        <button
          onClick={handleOpenVscode}
          className="p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors"
          title="VS Code"
        >
          <Code2 size={16} />
        </button>
        <button
          onClick={handleOpenTerminal}
          className="p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors"
          title="Terminal"
        >
          <Terminal size={16} />
        </button>
        <button
          onClick={handleOpenBrowser}
          className="p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors"
          title="Browser"
        >
          <Globe size={16} />
        </button>
        <button
          onClick={handleOpenExplorer}
          className="p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors"
          title="Explorer"
        >
          <FolderOpen size={16} />
        </button>
        <button
          onClick={handleOpenPhpMyAdmin}
          className="p-1.5 rounded hover:bg-slate-700 text-orange-400 hover:text-orange-300 transition-colors"
          title="Database"
        >
          <Database size={16} />
        </button>
      </div>

      {/* Divider */}
      <div className="w-px h-6 bg-slate-700 flex-shrink-0" />

      {/* Control Buttons */}
      <div className="flex items-center gap-1 flex-shrink-0">
        {isLoading ? (
          <div className="p-1.5">
            <Loader2 size={16} className="text-yellow-400 animate-spin" />
          </div>
        ) : isRunning ? (
          <>
            {/* Restart button - only when running */}
            <button
              onClick={handleRestart}
              disabled={isLoading}
              className="p-1.5 rounded hover:bg-slate-700 text-yellow-400 hover:text-yellow-300 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              title="Restart"
            >
              <RotateCcw size={16} />
            </button>
            {/* Stop button - enabled when running */}
            <button
              onClick={handleStop}
              disabled={isLoading}
              className="p-1.5 rounded hover:bg-red-500/20 text-red-400 hover:text-red-300 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              title="Stop"
            >
              <Square size={16} />
            </button>
          </>
        ) : (
          <>
            {/* Play button - only when stopped */}
            <button
              onClick={handleStart}
              disabled={isLoading}
              className="p-1.5 rounded hover:bg-green-500/20 text-green-400 hover:text-green-300 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              title="Start"
            >
              <Play size={16} />
            </button>
            {/* Stop button - disabled when stopped */}
            <button
              disabled
              className="p-1.5 rounded text-slate-600 cursor-not-allowed"
              title="Not running"
            >
              <Square size={16} />
            </button>
          </>
        )}
      </div>

      {/* Divider */}
      <div className="w-px h-6 bg-slate-700 flex-shrink-0" />

      {/* Edit/Delete */}
      <div className="flex items-center gap-1 flex-shrink-0">
        <button
          onClick={() => openModal("editProject", { projectId: project.id })}
          disabled={isLoading}
          className="p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          title="Edit"
        >
          <Edit size={16} />
        </button>
        <button
          onClick={() => openModal("confirmDelete", { projectId: project.id })}
          disabled={isLoading}
          className="p-1.5 rounded hover:bg-red-500/20 text-slate-400 hover:text-red-400 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          title="Delete"
        >
          <Trash2 size={16} />
        </button>
      </div>
      </div>

      {/* Progress bar for loading state */}
      {isLoading && (
        <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-slate-700 overflow-hidden">
          <div className="h-full bg-yellow-500 animate-pulse" style={{ width: '100%' }}>
            <div className="h-full bg-gradient-to-r from-yellow-500 via-yellow-300 to-yellow-500 animate-[shimmer_1.5s_infinite]" />
          </div>
        </div>
      )}
    </div>
  );
}
