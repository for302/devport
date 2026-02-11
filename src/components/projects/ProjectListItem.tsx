import { memo } from "react";
import { Play, Square, RotateCcw, Folder, Edit, Trash2, Code2, Terminal, Globe, FolderOpen, Github, Loader2 } from "lucide-react";
import type { Project } from "@/types";
import { useProcessStore, useUiStore, useActivityLogStore } from "@/stores";
import { useProjectActions, useBuildElapsedTime } from "@/hooks";
import { getProjectTypeColor } from "@/constants";

interface ProjectListItemProps {
  project: Project;
}

export const ProjectListItem = memo(function ProjectListItem({ project }: ProjectListItemProps) {
  const isRunning = useProcessStore((state) => state.isProjectRunning(project.id));
  const isLoading = useProcessStore((state) => state.isLoading[project.id] || false);
  const buildStatus = useProcessStore((state) => state.buildStatuses[project.id]);
  const startProject = useProcessStore((state) => state.startProject);
  const stopProject = useProcessStore((state) => state.stopProject);
  const restartProject = useProcessStore((state) => state.restartProject);
  const openModal = useUiStore((state) => state.openModal);
  const addLog = useActivityLogStore((state) => state.addLog);

  const elapsedTime = useBuildElapsedTime(project.id);

  // Use centralized project actions hook
  const { openInVscode, openInTerminal, openInBrowser, openInExplorer, openGitHub } = useProjectActions(project);

  const typeColor = getProjectTypeColor(project.projectType);

  const handleStart = async () => {
    addLog(project.name, "Starting project...", "info");
    try {
      const processInfo = await startProject(project.id);
      const actualPort = processInfo.port || project.port;
      addLog(project.name, actualPort > 0 ? `Started on port ${actualPort}` : "Started", "success");

      // Auto-open browser for web projects only (not app mode)
      if (actualPort > 0 && project.launchMode !== "app") {
        // Wait for server to start before opening browser
        setTimeout(() => {
          openInBrowser();
          addLog(project.name, `Opening http://localhost:${actualPort}`, "info");
        }, 2000);
      }
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
      const processInfo = await restartProject(project.id);
      const actualPort = processInfo.port || project.port;
      addLog(project.name, actualPort > 0 ? `Restarted on port ${actualPort}` : "Restarted", "success");
    } catch (err) {
      addLog(project.name, `Failed to restart: ${err}`, "error");
    }
  };

  return (
    <div className="relative">
      <div className="flex items-center gap-4 px-4 py-3 bg-slate-800 hover:bg-slate-750 border-b border-slate-700">
        {/* Status indicator */}
        <div className={`w-2 h-2 rounded-full flex-shrink-0 ${
          buildStatus === "compiling" ? "bg-orange-500 animate-pulse" :
          buildStatus === "error" ? "bg-red-500" :
          buildStatus === "starting" || buildStatus === "compiled" || isLoading
            ? "bg-yellow-500 animate-pulse" :
          isRunning ? "bg-green-500 animate-pulse" :
          "bg-slate-500"
        }`} />

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
        {project.port > 0 ? project.port : "—"}
      </span>

      {/* Build/Start status with elapsed time */}
      {(buildStatus || isLoading) && !isRunning && (
        <div className="flex items-center gap-2 flex-shrink-0">
          <span className={`text-xs font-medium ${
            buildStatus === "error" ? "text-red-400" :
            buildStatus === "compiling" ? "text-orange-400 animate-pulse" :
            "text-yellow-400 animate-pulse"
          }`}>
            {buildStatus === "error" ? "Build Error" :
             buildStatus === "compiling" ? "Building..." :
             buildStatus === "compiled" ? "Launching..." :
             "Starting..."}
          </span>
          {elapsedTime && buildStatus !== "error" && (
            <span className="text-xs text-slate-500 font-mono tabular-nums">{elapsedTime}</span>
          )}
        </div>
      )}

      {/* Quick Actions */}
      <div className="flex items-center gap-1 flex-shrink-0">
        <button
          onClick={openInVscode}
          className="p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors"
          title="VS Code"
        >
          <Code2 size={16} />
        </button>
        <button
          onClick={openInTerminal}
          className="p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors"
          title="Terminal"
        >
          <Terminal size={16} />
        </button>
        {project.port > 0 && (
        <button
          onClick={openInBrowser}
          className="p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors"
          title="Browser"
        >
          <Globe size={16} />
        </button>
        )}
        <button
          onClick={openInExplorer}
          className="p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors"
          title="Explorer"
        >
          <FolderOpen size={16} />
        </button>
        <button
          onClick={project.githubUrl ? openGitHub : undefined}
          disabled={!project.githubUrl}
          className={`p-1.5 rounded transition-colors ${
            project.githubUrl
              ? "hover:bg-slate-700 text-white hover:text-slate-200"
              : "text-slate-600 cursor-not-allowed"
          }`}
          title={project.githubUrl ? "Open GitHub" : "No GitHub repository"}
        >
          <Github size={16} />
        </button>
      </div>

      {/* Divider */}
      <div className="w-px h-6 bg-slate-700 flex-shrink-0" />

      {/* Control Buttons */}
      <div className="flex items-center gap-1 flex-shrink-0">
        {isLoading ? (
          <div className="flex items-center gap-1.5 p-1.5">
            <Loader2 size={16} className="text-yellow-400 animate-spin" />
            {elapsedTime && (
              <span className="text-xs text-slate-500 font-mono tabular-nums">{elapsedTime}</span>
            )}
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
});
