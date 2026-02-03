import { useState } from "react";
import { Info, X, Loader2 } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import type { PortInfo, Project } from "@/types";
import { usePortStore } from "@/stores";
import { getPortDescription } from "@/utils/portDescriptions";

interface ProcessDetails {
  pid: number;
  name: string;
  path: string | null;
  commandLine: string | null;
}

interface EnrichedPortInfo extends PortInfo {
  project?: Project;
}

interface PortRowProps {
  port: EnrichedPortInfo;
}

export function PortRow({ port }: PortRowProps) {
  const [showKillConfirm, setShowKillConfirm] = useState(false);
  const [showProcessInfo, setShowProcessInfo] = useState(false);
  const [processDetails, setProcessDetails] = useState<ProcessDetails | null>(null);
  const [isLoadingDetails, setIsLoadingDetails] = useState(false);
  const [isKilling, setIsKilling] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const scanPorts = usePortStore((state) => state.scanPorts);

  const stateColors: Record<string, string> = {
    LISTENING: "text-green-400",
    ESTABLISHED: "text-blue-400",
    "TIME_WAIT": "text-yellow-400",
    "CLOSE_WAIT": "text-orange-400",
    "N/A": "text-slate-500",
  };

  const stateColor = stateColors[port.state] || "text-slate-400";
  const description = getPortDescription(port.port, port.processName);

  const handleShowProcessInfo = async () => {
    if (!port.pid) return;

    setIsLoadingDetails(true);
    setError(null);

    try {
      const details = await invoke<ProcessDetails>("get_process_details", { pid: port.pid });
      setProcessDetails(details);
      setShowProcessInfo(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoadingDetails(false);
    }
  };

  const handleKillProcess = async () => {
    if (!port.pid) return;

    setIsKilling(true);
    setError(null);

    try {
      await invoke("kill_process_by_pid", { pid: port.pid });
      setShowKillConfirm(false);
      // Refresh port list after killing
      setTimeout(() => scanPorts(), 500);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsKilling(false);
    }
  };

  return (
    <>
      <tr className="border-t border-slate-700 hover:bg-slate-700/30 transition-colors">
        <td className="px-4 py-3">
          <span className="font-mono text-white">{port.port}</span>
        </td>
        <td className="px-4 py-3">
          {description ? (
            <div>
              <span className="text-blue-400 font-medium">{description.name}</span>
              <span className="text-slate-500 text-xs ml-2">{description.description}</span>
            </div>
          ) : (
            <span className="text-slate-500">-</span>
          )}
        </td>
        <td className="px-4 py-3">
          <span className="text-slate-300">{port.protocol}</span>
        </td>
        <td className="px-4 py-3">
          <span className={`font-medium ${stateColor}`}>{port.state}</span>
        </td>
        <td className="px-4 py-3">
          <span className="text-slate-300 truncate max-w-[200px] block">
            {port.processName || "-"}
          </span>
        </td>
        <td className="px-4 py-3">
          <span className="font-mono text-slate-400">{port.pid || "-"}</span>
        </td>
        <td className="px-4 py-3">
          {port.project ? (
            <span className="inline-flex items-center gap-2">
              <span className="w-2 h-2 bg-green-500 rounded-full" />
              <span className="text-white">{port.project.name}</span>
            </span>
          ) : (
            <span className="text-slate-500">-</span>
          )}
        </td>
        <td className="px-4 py-3">
          {port.pid && (
            <div className="flex items-center gap-1">
              <button
                onClick={handleShowProcessInfo}
                disabled={isLoadingDetails}
                className="p-1.5 rounded hover:bg-slate-600 text-slate-400 hover:text-slate-200 transition-colors"
                title="Process Info"
              >
                {isLoadingDetails ? (
                  <Loader2 size={16} className="animate-spin" />
                ) : (
                  <Info size={16} />
                )}
              </button>
              <button
                onClick={() => setShowKillConfirm(true)}
                className="p-1.5 rounded bg-red-500/10 hover:bg-red-500/30 text-red-400 hover:text-red-300 transition-colors"
                title="Kill Process"
              >
                <X size={16} />
              </button>
            </div>
          )}
        </td>
      </tr>

      {/* Process Info Modal */}
      {showProcessInfo && processDetails && (
        <tr>
          <td colSpan={8} className="p-0">
            <div className="mx-4 my-2 p-4 bg-slate-900 rounded-lg border border-slate-600">
              <div className="flex items-center justify-between mb-3">
                <h4 className="text-sm font-medium text-white">Process Details</h4>
                <button
                  onClick={() => setShowProcessInfo(false)}
                  className="p-1 rounded hover:bg-slate-700 text-slate-400"
                >
                  <X size={16} />
                </button>
              </div>
              <div className="space-y-2 text-sm">
                <div className="flex">
                  <span className="text-slate-400 w-24">PID:</span>
                  <span className="text-white font-mono">{processDetails.pid}</span>
                </div>
                <div className="flex">
                  <span className="text-slate-400 w-24">Name:</span>
                  <span className="text-white">{processDetails.name}</span>
                </div>
                {processDetails.path && (
                  <div className="flex">
                    <span className="text-slate-400 w-24">Path:</span>
                    <span className="text-white break-all">{processDetails.path}</span>
                  </div>
                )}
                {processDetails.commandLine && (
                  <div className="flex">
                    <span className="text-slate-400 w-24">Command:</span>
                    <span className="text-white break-all font-mono text-xs">
                      {processDetails.commandLine}
                    </span>
                  </div>
                )}
              </div>
            </div>
          </td>
        </tr>
      )}

      {/* Kill Confirmation Modal */}
      {showKillConfirm && (
        <tr>
          <td colSpan={8} className="p-0">
            <div className="mx-4 my-2 p-4 bg-red-900/20 rounded-lg border border-red-800">
              <div className="flex items-start gap-3">
                <div className="p-2 bg-red-500/20 rounded-lg">
                  <X size={20} className="text-red-400" />
                </div>
                <div className="flex-1">
                  <h4 className="text-sm font-medium text-white mb-1">
                    Confirm Process Termination
                  </h4>
                  <p className="text-sm text-slate-300 mb-1">
                    Kill <span className="font-mono text-red-400">{port.processName}</span> (PID: {port.pid})?
                  </p>
                  {!port.project && (
                    <p className="text-xs text-yellow-400 mb-3">
                      This process was not started by DevPort.
                    </p>
                  )}
                  {error && (
                    <p className="text-xs text-red-400 mb-3">{error}</p>
                  )}
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => setShowKillConfirm(false)}
                      className="px-3 py-1.5 text-sm rounded bg-slate-700 hover:bg-slate-600 text-white transition-colors"
                    >
                      Cancel
                    </button>
                    <button
                      onClick={handleKillProcess}
                      disabled={isKilling}
                      className="px-3 py-1.5 text-sm rounded bg-red-600 hover:bg-red-700 text-white transition-colors disabled:opacity-50"
                    >
                      {isKilling ? (
                        <span className="flex items-center gap-1">
                          <Loader2 size={14} className="animate-spin" />
                          Killing...
                        </span>
                      ) : (
                        "Kill Process"
                      )}
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </td>
        </tr>
      )}
    </>
  );
}
