import { RefreshCw } from "lucide-react";
import { usePortStore } from "@/stores";
import { PortTable } from "../ports/PortTable";

export function PortsView() {
  const scanPorts = usePortStore((state) => state.scanPorts);
  const isScanning = usePortStore((state) => state.isScanning);
  const lastScanned = usePortStore((state) => state.lastScanned);

  const formatLastScanned = () => {
    if (!lastScanned) return "Never";
    const date = new Date(lastScanned);
    return date.toLocaleTimeString();
  };

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-white">Port Dashboard</h1>
          <p className="text-sm text-slate-400">
            Last scanned: {formatLastScanned()}
          </p>
        </div>
        <button
          onClick={() => scanPorts()}
          disabled={isScanning}
          className={`
            flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors
            ${isScanning
              ? "bg-slate-700 text-slate-400 cursor-not-allowed"
              : "bg-blue-600 hover:bg-blue-700 text-white"
            }
          `}
        >
          <RefreshCw size={18} className={isScanning ? "animate-spin" : ""} />
          {isScanning ? "Scanning..." : "Refresh"}
        </button>
      </div>

      <PortTable />
    </div>
  );
}
