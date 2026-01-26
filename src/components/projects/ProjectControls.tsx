import { Play, Square, RotateCw, Loader2 } from "lucide-react";
import type { Project } from "@/types";
import { useProcessStore, usePortStore, useUiStore } from "@/stores";

interface ProjectControlsProps {
  project: Project;
}

export function ProjectControls({ project }: ProjectControlsProps) {
  const isRunning = useProcessStore((state) => state.isProjectRunning(project.id));
  const isLoading = useProcessStore((state) => state.isLoading[project.id]);
  const startProject = useProcessStore((state) => state.startProject);
  const stopProject = useProcessStore((state) => state.stopProject);
  const restartProject = useProcessStore((state) => state.restartProject);
  const checkPortAvailable = usePortStore((state) => state.checkPortAvailable);
  const openModal = useUiStore((state) => state.openModal);

  const handleStart = async () => {
    // Check if port is available
    const isAvailable = await checkPortAvailable(project.port);
    if (!isAvailable) {
      openModal("portConflict", {
        projectId: project.id,
        port: project.port,
      });
      return;
    }

    try {
      await startProject(project.id);
    } catch (error) {
      console.error("Failed to start project:", error);
    }
  };

  const handleStop = async () => {
    try {
      await stopProject(project.id);
    } catch (error) {
      console.error("Failed to stop project:", error);
    }
  };

  const handleRestart = async () => {
    try {
      await restartProject(project.id);
    } catch (error) {
      console.error("Failed to restart project:", error);
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-2">
        <Loader2 size={20} className="animate-spin text-blue-400" />
      </div>
    );
  }

  return (
    <div className="flex gap-2">
      {isRunning ? (
        <>
          <button
            onClick={handleStop}
            className="flex-1 flex items-center justify-center gap-2 py-2 rounded-lg
              bg-red-500/20 hover:bg-red-500/30 text-red-400 font-medium transition-colors"
          >
            <Square size={16} />
            Stop
          </button>
          <button
            onClick={handleRestart}
            className="flex-1 flex items-center justify-center gap-2 py-2 rounded-lg
              bg-yellow-500/20 hover:bg-yellow-500/30 text-yellow-400 font-medium transition-colors"
          >
            <RotateCw size={16} />
            Restart
          </button>
        </>
      ) : (
        <button
          onClick={handleStart}
          className="flex-1 flex items-center justify-center gap-2 py-2 rounded-lg
            bg-green-500/20 hover:bg-green-500/30 text-green-400 font-medium transition-colors"
        >
          <Play size={16} />
          Start
        </button>
      )}
    </div>
  );
}
