import { useState } from "react";
import { Trash2, X, Loader2 } from "lucide-react";
import { useUiStore, useProjectStore, useProcessStore } from "@/stores";

export function ConfirmDeleteModal() {
  const closeModal = useUiStore((state) => state.closeModal);
  const modalData = useUiStore((state) => state.modalData);
  const deleteProject = useProjectStore((state) => state.deleteProject);
  const getProjectById = useProjectStore((state) => state.getProjectById);
  const stopProject = useProcessStore((state) => state.stopProject);
  const isRunning = useProcessStore((state) =>
    state.isProjectRunning(modalData.projectId as string)
  );

  const projectId = modalData.projectId as string;
  const project = getProjectById(projectId);

  const [isDeleting, setIsDeleting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleDelete = async () => {
    setIsDeleting(true);
    setError(null);

    try {
      // Stop the project if it's running
      if (isRunning) {
        await stopProject(projectId);
      }

      // Delete the project
      await deleteProject(projectId);
      closeModal();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete project");
    } finally {
      setIsDeleting(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-slate-800 rounded-lg w-full max-w-md mx-4">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-700">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-red-500/20 rounded-lg">
              <Trash2 size={24} className="text-red-400" />
            </div>
            <h2 className="text-lg font-semibold text-white">Delete Project</h2>
          </div>
          <button
            onClick={closeModal}
            className="p-1 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors"
          >
            <X size={20} />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-4">
          {error && (
            <div className="p-3 bg-red-500/20 border border-red-500/50 rounded-lg text-red-400 text-sm">
              {error}
            </div>
          )}

          <p className="text-slate-300">
            Are you sure you want to delete{" "}
            <span className="font-semibold text-white">{project?.name}</span>?
          </p>

          {isRunning && (
            <div className="p-3 bg-yellow-500/20 border border-yellow-500/50 rounded-lg text-yellow-400 text-sm">
              This project is currently running and will be stopped before deletion.
            </div>
          )}

          <p className="text-sm text-slate-400">
            This will remove the project from DevPort Manager. Your project files will not be affected.
          </p>
        </div>

        {/* Actions */}
        <div className="flex justify-end gap-3 px-6 py-4 border-t border-slate-700">
          <button
            onClick={closeModal}
            disabled={isDeleting}
            className="px-4 py-2 rounded-lg text-slate-300 hover:bg-slate-700 transition-colors
              disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            onClick={handleDelete}
            disabled={isDeleting}
            className="px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg text-white font-medium
              transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isDeleting ? (
              <Loader2 size={20} className="animate-spin" />
            ) : (
              "Delete"
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
