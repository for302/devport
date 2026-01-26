import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Trash2,
  AlertTriangle,
  Loader2,
  CheckCircle,
  XCircle,
  Folder,
  FileCode,
  File,
  Calendar,
  Globe,
  Shield,
  Link,
  Key,
  ChevronDown,
  ChevronUp,
} from "lucide-react";
import {
  UninstallMode,
  UninstallPreview,
  UninstallResult,
  UninstallPreviewItem,
  UninstallItemType,
  RunningProcessesInfo,
  UNINSTALL_MODE_LABELS,
  UNINSTALL_MODE_DESCRIPTIONS,
  formatUninstallBytes,
} from "@/types/uninstaller";

type UninstallStatus =
  | "idle"
  | "loading-preview"
  | "previewing"
  | "stopping"
  | "uninstalling"
  | "complete"
  | "error";

const ITEM_TYPE_ICONS: Record<UninstallItemType, React.ReactNode> = {
  executable: <FileCode size={16} />,
  directory: <Folder size={16} />,
  file: <File size={16} />,
  taskScheduler: <Calendar size={16} />,
  hostsEntry: <Globe size={16} />,
  firewallRule: <Shield size={16} />,
  shortcut: <Link size={16} />,
  registryKey: <Key size={16} />,
};

export function UninstallPanel() {
  const [selectedMode, setSelectedMode] = useState<UninstallMode>("basic");
  const [status, setStatus] = useState<UninstallStatus>("idle");
  const [preview, setPreview] = useState<UninstallPreview | null>(null);
  const [result, setResult] = useState<UninstallResult | null>(null);
  const [runningInfo, setRunningInfo] = useState<RunningProcessesInfo | null>(
    null
  );
  const [confirmChecked, setConfirmChecked] = useState(false);
  const [showPreviewDetails, setShowPreviewDetails] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Check for running processes on mount
  useEffect(() => {
    const checkRunning = async () => {
      try {
        const info = await invoke<RunningProcessesInfo>(
          "check_running_processes"
        );
        setRunningInfo(info);
      } catch (err) {
        console.error("Failed to check running processes:", err);
      }
    };
    checkRunning();
  }, []);

  // Load preview when mode changes
  useEffect(() => {
    const loadPreview = async () => {
      setStatus("loading-preview");
      setError(null);
      setConfirmChecked(false);

      try {
        const previewData = await invoke<UninstallPreview>(
          "get_uninstall_preview",
          {
            mode: selectedMode,
          }
        );
        setPreview(previewData);
        setStatus("previewing");
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        setStatus("error");
      }
    };

    loadPreview();
  }, [selectedMode]);

  const handleUninstall = useCallback(async () => {
    if (!confirmChecked) {
      setError("Please confirm that you understand the consequences.");
      return;
    }

    setStatus("stopping");
    setError(null);

    try {
      // Stop all services and projects first
      await invoke<boolean>("stop_all_for_uninstall");

      setStatus("uninstalling");

      // Perform uninstall
      const uninstallResult = await invoke<UninstallResult>("perform_uninstall", {
        mode: selectedMode,
      });

      setResult(uninstallResult);
      setStatus("complete");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setStatus("error");
    }
  }, [selectedMode, confirmChecked]);

  const handleModeChange = (mode: UninstallMode) => {
    setSelectedMode(mode);
    setResult(null);
    setError(null);
  };

  const isDestructiveMode =
    selectedMode === "fullData" || selectedMode === "systemRevert";

  return (
    <div className="space-y-6">
      {/* Header Warning */}
      <div className="p-4 bg-red-900/20 border border-red-800 rounded-lg flex items-start gap-3">
        <AlertTriangle className="text-red-400 shrink-0 mt-0.5" size={20} />
        <div>
          <p className="text-red-400 font-medium">Uninstall DevPort Manager</p>
          <p className="text-sm text-red-400/70">
            This action will remove DevPort components from your system. Please
            review the options carefully before proceeding.
          </p>
        </div>
      </div>

      {/* Running Services Warning */}
      {runningInfo?.anyRunning && (
        <div className="p-4 bg-amber-900/20 border border-amber-800 rounded-lg flex items-start gap-3">
          <AlertTriangle
            className="text-amber-400 shrink-0 mt-0.5"
            size={20}
          />
          <div>
            <p className="text-amber-400 font-medium">
              Services are currently running
            </p>
            <p className="text-sm text-amber-400/70">
              {runningInfo.apacheRunning && "Apache "}
              {runningInfo.apacheRunning && runningInfo.mariadbRunning && "and "}
              {runningInfo.mariadbRunning && "MariaDB "}
              {runningInfo.apacheRunning || runningInfo.mariadbRunning
                ? "are"
                : "Services are"}{" "}
              currently running. They will be stopped automatically before
              uninstall.
            </p>
          </div>
        </div>
      )}

      {/* Mode Selection */}
      <div className="space-y-3">
        <h3 className="text-lg font-semibold text-white">Uninstall Mode</h3>
        <div className="space-y-2">
          {(["basic", "fullData", "systemRevert"] as UninstallMode[]).map(
            (mode) => (
              <label
                key={mode}
                className={`flex items-start gap-3 p-4 rounded-lg border cursor-pointer transition-colors ${
                  selectedMode === mode
                    ? mode === "basic"
                      ? "bg-blue-900/20 border-blue-700"
                      : mode === "fullData"
                        ? "bg-amber-900/20 border-amber-700"
                        : "bg-red-900/20 border-red-700"
                    : "bg-slate-800/50 border-slate-700 hover:border-slate-600"
                }`}
              >
                <input
                  type="radio"
                  name="uninstall-mode"
                  value={mode}
                  checked={selectedMode === mode}
                  onChange={() => handleModeChange(mode)}
                  className="mt-1 w-4 h-4 text-blue-600 focus:ring-blue-500"
                />
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <span
                      className={`font-medium ${
                        mode === "basic"
                          ? "text-white"
                          : mode === "fullData"
                            ? "text-amber-400"
                            : "text-red-400"
                      }`}
                    >
                      {UNINSTALL_MODE_LABELS[mode]}
                    </span>
                    {mode === "systemRevert" && (
                      <span className="px-2 py-0.5 text-xs bg-red-800 text-red-200 rounded">
                        Admin Required
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-slate-400 mt-1">
                    {UNINSTALL_MODE_DESCRIPTIONS[mode]}
                  </p>
                </div>
              </label>
            )
          )}
        </div>
      </div>

      {/* Preview Section */}
      {status === "loading-preview" && (
        <div className="flex items-center gap-3 text-slate-400">
          <Loader2 className="animate-spin" size={20} />
          <span>Loading preview...</span>
        </div>
      )}

      {status === "previewing" && preview && (
        <div className="space-y-4">
          {/* Preview Summary */}
          <div className="p-4 bg-slate-800/50 rounded-lg">
            <div className="flex items-center justify-between mb-3">
              <h4 className="font-medium text-white">Items to be removed</h4>
              <span className="text-sm text-slate-400">
                Total size: {formatUninstallBytes(preview.totalSizeBytes)}
              </span>
            </div>

            {/* Warnings */}
            {preview.warnings.length > 0 && (
              <div className="mb-3 space-y-2">
                {preview.warnings.map((warning, index) => (
                  <div
                    key={index}
                    className="flex items-start gap-2 text-sm text-amber-400"
                  >
                    <AlertTriangle size={14} className="shrink-0 mt-0.5" />
                    <span>{warning}</span>
                  </div>
                ))}
              </div>
            )}

            {/* Preview Toggle */}
            <button
              onClick={() => setShowPreviewDetails(!showPreviewDetails)}
              className="flex items-center gap-2 text-sm text-blue-400 hover:text-blue-300"
            >
              {showPreviewDetails ? (
                <ChevronUp size={16} />
              ) : (
                <ChevronDown size={16} />
              )}
              {showPreviewDetails ? "Hide" : "Show"} detailed list (
              {preview.items.filter((item) => item.exists).length} items)
            </button>

            {/* Detailed Preview List */}
            {showPreviewDetails && (
              <div className="mt-3 space-y-1 max-h-64 overflow-y-auto">
                {preview.items.map((item, index) => (
                  <PreviewItem key={index} item={item} />
                ))}
              </div>
            )}
          </div>

          {/* Confirmation Checkbox */}
          {isDestructiveMode && (
            <label className="flex items-start gap-3 p-4 bg-red-900/10 border border-red-800/50 rounded-lg cursor-pointer">
              <input
                type="checkbox"
                checked={confirmChecked}
                onChange={(e) => setConfirmChecked(e.target.checked)}
                className="mt-1 w-4 h-4 rounded border-red-600 bg-slate-800 text-red-600 focus:ring-red-500"
              />
              <div>
                <span className="text-red-400 font-medium">
                  I understand this action is irreversible
                </span>
                <p className="text-sm text-red-400/70 mt-1">
                  {selectedMode === "fullData"
                    ? "All my projects, database backups, and log files will be permanently deleted."
                    : "All data will be deleted and system changes (hosts, firewall, Task Scheduler) will be reverted."}
                </p>
              </div>
            </label>
          )}

          {/* Error Message */}
          {error && (
            <div className="p-3 bg-red-900/20 border border-red-800 rounded-lg flex items-start gap-2">
              <XCircle className="text-red-400 shrink-0 mt-0.5" size={18} />
              <p className="text-sm text-red-400">{error}</p>
            </div>
          )}

          {/* Uninstall Button */}
          <div className="flex justify-end">
            <button
              onClick={handleUninstall}
              disabled={isDestructiveMode && !confirmChecked}
              className={`flex items-center gap-2 px-6 py-3 rounded-lg font-medium transition-colors ${
                isDestructiveMode
                  ? "bg-red-600 hover:bg-red-700 text-white disabled:bg-red-900 disabled:text-red-400"
                  : "bg-amber-600 hover:bg-amber-700 text-white"
              } disabled:cursor-not-allowed`}
            >
              <Trash2 size={20} />
              {selectedMode === "basic"
                ? "Uninstall DevPort"
                : selectedMode === "fullData"
                  ? "Uninstall and Delete All Data"
                  : "Full System Revert"}
            </button>
          </div>
        </div>
      )}

      {/* Stopping/Uninstalling State */}
      {(status === "stopping" || status === "uninstalling") && (
        <div className="p-6 bg-slate-800/50 rounded-lg flex flex-col items-center justify-center gap-4">
          <Loader2 className="animate-spin text-blue-400" size={40} />
          <div className="text-center">
            <p className="text-lg font-medium text-white">
              {status === "stopping"
                ? "Stopping services..."
                : "Uninstalling DevPort..."}
            </p>
            <p className="text-sm text-slate-400 mt-1">
              {status === "stopping"
                ? "Please wait while services are being stopped."
                : "Please do not close this window."}
            </p>
          </div>
        </div>
      )}

      {/* Completion State */}
      {status === "complete" && result && (
        <UninstallComplete result={result} />
      )}
    </div>
  );
}

interface PreviewItemProps {
  item: UninstallPreviewItem;
}

function PreviewItem({ item }: PreviewItemProps) {
  return (
    <div
      className={`flex items-center gap-3 p-2 rounded ${
        item.exists
          ? "bg-slate-700/50"
          : "bg-slate-800/30 text-slate-500"
      }`}
    >
      <span className={item.exists ? "text-slate-400" : "text-slate-600"}>
        {ITEM_TYPE_ICONS[item.itemType]}
      </span>
      <div className="flex-1 min-w-0">
        <p
          className={`text-sm truncate ${
            item.exists ? "text-white" : "text-slate-500"
          }`}
        >
          {item.name}
        </p>
        {item.path && (
          <p
            className={`text-xs truncate ${
              item.exists ? "text-slate-400" : "text-slate-600"
            }`}
          >
            {item.path}
          </p>
        )}
      </div>
      <div className="text-right shrink-0">
        {item.sizeBytes !== null && (
          <span className="text-xs text-slate-500">
            {formatUninstallBytes(item.sizeBytes)}
          </span>
        )}
        {!item.exists && (
          <span className="text-xs text-slate-600 italic">Not found</span>
        )}
      </div>
    </div>
  );
}

interface UninstallCompleteProps {
  result: UninstallResult;
}

function UninstallComplete({ result }: UninstallCompleteProps) {
  const [showDetails, setShowDetails] = useState(false);

  return (
    <div className="space-y-4">
      {result.success ? (
        <div className="p-6 bg-green-900/20 border border-green-800 rounded-lg flex flex-col items-center justify-center gap-4">
          <CheckCircle className="text-green-400" size={48} />
          <div className="text-center">
            <p className="text-lg font-medium text-green-400">
              Uninstall Complete
            </p>
            <p className="text-sm text-green-400/70 mt-1">
              DevPort Manager has been successfully removed from your system.
            </p>
            {result.requiresReboot && (
              <p className="text-sm text-amber-400 mt-2">
                A system restart may be required to complete the cleanup.
              </p>
            )}
          </div>
        </div>
      ) : (
        <div className="p-6 bg-amber-900/20 border border-amber-800 rounded-lg flex flex-col items-center justify-center gap-4">
          <AlertTriangle className="text-amber-400" size={48} />
          <div className="text-center">
            <p className="text-lg font-medium text-amber-400">
              Uninstall Completed with Issues
            </p>
            <p className="text-sm text-amber-400/70 mt-1">
              Some items could not be removed. You may need to delete them
              manually or restart your computer.
            </p>
          </div>
        </div>
      )}

      {/* Summary */}
      <div className="p-4 bg-slate-800/50 rounded-lg">
        <div className="flex items-center justify-between mb-3">
          <h4 className="font-medium text-white">Summary</h4>
          <div className="flex gap-4 text-sm">
            <span className="text-green-400">
              {result.removedItems.length} removed
            </span>
            {result.failedItems.length > 0 && (
              <span className="text-red-400">
                {result.failedItems.length} failed
              </span>
            )}
          </div>
        </div>

        <button
          onClick={() => setShowDetails(!showDetails)}
          className="flex items-center gap-2 text-sm text-blue-400 hover:text-blue-300"
        >
          {showDetails ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
          {showDetails ? "Hide" : "Show"} details
        </button>

        {showDetails && (
          <div className="mt-3 space-y-4">
            {/* Removed Items */}
            {result.removedItems.length > 0 && (
              <div>
                <p className="text-sm font-medium text-green-400 mb-2">
                  Successfully Removed
                </p>
                <div className="space-y-1 max-h-32 overflow-y-auto">
                  {result.removedItems.map((item, index) => (
                    <div
                      key={index}
                      className="flex items-center gap-2 text-sm text-slate-300"
                    >
                      <CheckCircle size={14} className="text-green-400" />
                      <span>{item.name}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Failed Items */}
            {result.failedItems.length > 0 && (
              <div>
                <p className="text-sm font-medium text-red-400 mb-2">
                  Failed to Remove
                </p>
                <div className="space-y-2 max-h-32 overflow-y-auto">
                  {result.failedItems.map((item, index) => (
                    <div key={index} className="text-sm">
                      <div className="flex items-center gap-2 text-red-300">
                        <XCircle size={14} className="text-red-400" />
                        <span>{item.name}</span>
                      </div>
                      <p className="ml-5 text-xs text-red-400/70">
                        {item.reason}
                      </p>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
