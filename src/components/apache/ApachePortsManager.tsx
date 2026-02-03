import { useState, useEffect, useCallback } from "react";
import {
  RefreshCw,
  Plus,
  AlertTriangle,
  Server,
  Loader2,
  Play,
  Square,
  RotateCcw,
} from "lucide-react";
import { useApacheConfigStore, useServiceStore, useActivityLogStore } from "@/stores";
import { ApachePortListItem } from "./ApachePortListItem";
import { ApachePortModal } from "./ApachePortModal";
import type { ApachePortEntry } from "@/types";

interface ApachePortsManagerProps {
  compact?: boolean;
}

export function ApachePortsManager({ compact = false }: ApachePortsManagerProps) {
  const {
    ports,
    isLoading,
    error,
    needsRestart,
    fetchPorts,
    fetchApacheBasePath,
    deleteVHost,
    setNeedsRestart,
    clearError,
  } = useApacheConfigStore();

  const { services, startService, stopService, restartService } = useServiceStore();
  const { addLog } = useActivityLogStore();
  const apacheService = services.find((s) => s.id === "apache");
  const isApacheRunning = apacheService?.status === "running";
  const isApacheInstalled = !!apacheService;

  const [isServiceLoading, setIsServiceLoading] = useState(false);
  const [serviceAction, setServiceAction] = useState<"start" | "stop" | "restart" | null>(null);

  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editEntry, setEditEntry] = useState<ApachePortEntry | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<ApachePortEntry | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  useEffect(() => {
    fetchApacheBasePath();
    fetchPorts();
  }, [fetchApacheBasePath, fetchPorts]);

  const handleRefresh = useCallback(() => {
    fetchPorts();
    setNeedsRestart(false);
  }, [fetchPorts, setNeedsRestart]);

  const handleStartApache = useCallback(async () => {
    setIsServiceLoading(true);
    setServiceAction("start");
    addLog("Apache", "Starting service...", "info");
    try {
      await startService("apache");
      addLog("Apache", "Service started successfully", "success");
      setNeedsRestart(false);
    } catch (err) {
      addLog("Apache", `Failed to start: ${err}`, "error");
    } finally {
      setIsServiceLoading(false);
      setServiceAction(null);
    }
  }, [startService, addLog, setNeedsRestart]);

  const handleStopApache = useCallback(async () => {
    setIsServiceLoading(true);
    setServiceAction("stop");
    addLog("Apache", "Stopping service...", "info");
    try {
      await stopService("apache");
      addLog("Apache", "Service stopped successfully", "success");
    } catch (err) {
      addLog("Apache", `Failed to stop: ${err}`, "error");
    } finally {
      setIsServiceLoading(false);
      setServiceAction(null);
    }
  }, [stopService, addLog]);

  const handleRestartApache = useCallback(async () => {
    setIsServiceLoading(true);
    setServiceAction("restart");
    addLog("Apache", "Restarting service...", "info");
    try {
      await restartService("apache");
      addLog("Apache", "Service restarted successfully", "success");
      setNeedsRestart(false);
    } catch (err) {
      addLog("Apache", `Failed to restart: ${err}`, "error");
    } finally {
      setIsServiceLoading(false);
      setServiceAction(null);
    }
  }, [restartService, addLog, setNeedsRestart]);

  const handleAdd = () => {
    setEditEntry(null);
    setIsModalOpen(true);
  };

  const handleEdit = (entry: ApachePortEntry) => {
    setEditEntry(entry);
    setIsModalOpen(true);
  };

  const handleCloseModal = () => {
    setIsModalOpen(false);
    setEditEntry(null);
  };

  const handleDelete = (entry: ApachePortEntry) => {
    setDeleteConfirm(entry);
  };

  const handleConfirmDelete = async () => {
    if (!deleteConfirm) return;
    setIsDeleting(true);
    try {
      await deleteVHost(deleteConfirm.id);
      setDeleteConfirm(null);
    } catch (err) {
      console.error("Failed to delete VirtualHost:", err);
    } finally {
      setIsDeleting(false);
    }
  };

  // Loading state
  if (isLoading && ports.length === 0) {
    return (
      <div className={compact ? "mt-6" : ""}>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-white">Apache VirtualHosts</h2>
        </div>
        <div className="bg-slate-800 rounded-lg border border-slate-700 p-8">
          <div className="flex items-center justify-center">
            <Loader2 size={24} className="animate-spin text-blue-400" />
            <span className="ml-2 text-slate-400">Loading Apache configuration...</span>
          </div>
        </div>
      </div>
    );
  }

  // No Apache installed state
  if (!isLoading && error && error.includes("not found")) {
    return null;
  }

  return (
    <div className={compact ? "mt-6" : ""}>
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <h2 className="text-lg font-semibold text-white">Apache VirtualHosts</h2>

          {/* Apache Service Control Buttons */}
          {isApacheInstalled && (
            <div className="flex items-center gap-1 px-2 py-1 bg-slate-800 rounded-lg border border-slate-700">
              {isServiceLoading ? (
                <div className="flex items-center gap-2 px-2">
                  <Loader2 size={14} className="text-yellow-400 animate-spin" />
                  <span className="text-xs text-yellow-400">
                    {serviceAction === "start" ? "Starting..." :
                     serviceAction === "stop" ? "Stopping..." : "Restarting..."}
                  </span>
                </div>
              ) : isApacheRunning ? (
                <>
                  {/* Restart button - only when running */}
                  <button
                    onClick={handleRestartApache}
                    disabled={isServiceLoading}
                    className="p-1.5 rounded hover:bg-slate-700 text-yellow-400 hover:text-yellow-300 transition-colors disabled:opacity-50"
                    title="Restart Apache"
                  >
                    <RotateCcw size={16} />
                  </button>
                  {/* Stop button - enabled when running */}
                  <button
                    onClick={handleStopApache}
                    disabled={isServiceLoading}
                    className="p-1.5 rounded hover:bg-red-500/20 text-red-400 hover:text-red-300 transition-colors disabled:opacity-50"
                    title="Stop Apache"
                  >
                    <Square size={16} />
                  </button>
                  <span className="ml-1 px-1.5 py-0.5 bg-green-500/20 text-green-400 text-xs rounded">
                    Running
                  </span>
                </>
              ) : (
                <>
                  {/* Play button - only when stopped */}
                  <button
                    onClick={handleStartApache}
                    disabled={isServiceLoading}
                    className="p-1.5 rounded hover:bg-green-500/20 text-green-400 hover:text-green-300 transition-colors disabled:opacity-50"
                    title="Start Apache"
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
                  <span className="ml-1 px-1.5 py-0.5 bg-slate-700 text-slate-400 text-xs rounded">
                    Stopped
                  </span>
                </>
              )}
            </div>
          )}

          {needsRestart && isApacheRunning && (
            <span className="flex items-center gap-1 px-2 py-0.5 bg-yellow-500/20 text-yellow-400 text-xs rounded-full">
              <AlertTriangle size={12} />
              Restart Required
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleRefresh}
            disabled={isLoading}
            className="p-1.5 text-slate-400 hover:text-blue-400 hover:bg-slate-700 rounded transition-colors disabled:opacity-50"
            title="Refresh"
          >
            <RefreshCw size={16} className={isLoading ? "animate-spin" : ""} />
          </button>
          <button
            onClick={handleAdd}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors"
          >
            <Plus size={16} />
            Add VHost
          </button>
        </div>
      </div>

      {/* Error Message */}
      {error && !error.includes("not found") && (
        <div className="mb-4 p-3 bg-red-500/10 border border-red-500/30 rounded-lg flex items-center justify-between">
          <span className="text-sm text-red-400">{error}</span>
          <button
            onClick={clearError}
            className="text-red-400 hover:text-red-300 text-sm"
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Content */}
      {ports.length === 0 ? (
        <div className="text-center py-12 bg-slate-800 rounded-lg border border-slate-700">
          <Server size={48} className="mx-auto text-slate-600 mb-4" />
          <h3 className="text-lg font-medium text-slate-400 mb-2">No VirtualHosts configured</h3>
          <p className="text-sm text-slate-500">
            Click "Add VHost" to create one
          </p>
        </div>
      ) : (
        <div className="bg-slate-800 rounded-lg border border-slate-700 overflow-hidden">
          {/* Header Row - matches Projects header */}
          <div className="flex items-center gap-4 px-4 py-2 bg-slate-900 border-b border-slate-700 text-xs text-slate-400 uppercase tracking-wider">
            <div className="w-2 flex-shrink-0" />
            <div className="w-16 flex-shrink-0">Framework</div>
            <div className="w-32 min-w-[8rem] flex-shrink-0">Domain</div>
            <div className="hidden lg:flex flex-1 min-w-0">Document Root</div>
            <div className="w-16 text-center flex-shrink-0">Port</div>
            <div className="w-32 text-center flex-shrink-0">Quick Actions</div>
            <div className="w-px h-4 flex-shrink-0" />
            <div className="w-16 text-center flex-shrink-0">Config</div>
            <div className="w-px h-4 flex-shrink-0" />
            <div className="w-16 text-center flex-shrink-0">Edit</div>
          </div>
          {/* List Items */}
          <div>
            {ports.map((entry) => (
              <ApachePortListItem
                key={entry.id}
                entry={entry}
                onEdit={handleEdit}
                onDelete={handleDelete}
              />
            ))}
          </div>
        </div>
      )}

      {/* Modal */}
      <ApachePortModal
        isOpen={isModalOpen}
        onClose={handleCloseModal}
        editEntry={editEntry}
      />

      {/* Delete Confirmation Dialog */}
      {deleteConfirm && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-slate-800 rounded-lg w-full max-w-md mx-4 p-6">
            <h3 className="text-lg font-semibold text-white mb-2">
              Delete VirtualHost
            </h3>
            <p className="text-slate-400 mb-4">
              Are you sure you want to delete the VirtualHost for{" "}
              <span className="text-white font-medium">
                {deleteConfirm.domain}:{deleteConfirm.port}
              </span>
              ?
            </p>
            <p className="text-sm text-yellow-400 mb-4">
              This action cannot be undone. The DocumentRoot folder will not be deleted.
            </p>
            <div className="flex justify-end gap-3">
              <button
                onClick={() => setDeleteConfirm(null)}
                className="px-4 py-2 rounded-lg text-slate-300 hover:bg-slate-700 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleConfirmDelete}
                disabled={isDeleting}
                className="px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg text-white font-medium
                  transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
              >
                {isDeleting && <Loader2 size={16} className="animate-spin" />}
                Delete
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
