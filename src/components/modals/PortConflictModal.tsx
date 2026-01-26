import { AlertTriangle, X } from "lucide-react";
import { useUiStore, usePortStore } from "@/stores";

export function PortConflictModal() {
  const closeModal = useUiStore((state) => state.closeModal);
  const modalData = useUiStore((state) => state.modalData);
  const port = modalData.port as number;
  const portInfo = usePortStore((state) => state.getPortByNumber(port));

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-slate-800 rounded-lg w-full max-w-md mx-4">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-700">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-yellow-500/20 rounded-lg">
              <AlertTriangle size={24} className="text-yellow-400" />
            </div>
            <h2 className="text-lg font-semibold text-white">Port Conflict</h2>
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
          <p className="text-slate-300">
            Port <span className="font-mono font-bold text-white">{port}</span> is already in use.
          </p>

          {portInfo && (
            <div className="bg-slate-900 rounded-lg p-4 space-y-2">
              <div className="flex justify-between text-sm">
                <span className="text-slate-400">Process</span>
                <span className="text-white">{portInfo.processName || "Unknown"}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-slate-400">PID</span>
                <span className="font-mono text-white">{portInfo.pid || "Unknown"}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-slate-400">State</span>
                <span className="text-white">{portInfo.state}</span>
              </div>
            </div>
          )}

          <p className="text-sm text-slate-400">
            Please stop the process using this port, or change the port in the project settings.
          </p>
        </div>

        {/* Actions */}
        <div className="flex justify-end gap-3 px-6 py-4 border-t border-slate-700">
          <button
            onClick={closeModal}
            className="px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-lg text-white transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
