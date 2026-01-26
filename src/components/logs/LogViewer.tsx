import { useEffect, useRef, useMemo } from "react";
import { Trash2, Download } from "lucide-react";
import { useLogStore, useProjectStore, useServiceStore } from "@/stores";

interface LogViewerProps {
  projectId: string;
}

export function LogViewer({ projectId }: LogViewerProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  // Check if this is a service or project
  const isService = projectId.startsWith("service:");
  const serviceId = isService ? projectId.replace("service:", "") : null;

  // Get project or service
  const project = useProjectStore((state) =>
    isService ? undefined : state.getProjectById(projectId)
  );
  const services = useServiceStore((state) => state.services);
  const service = useMemo(
    () => serviceId ? services.find((s) => s.id === serviceId) : undefined,
    [services, serviceId]
  );

  // Get logs - for projects use logStore, for services use serviceStore logs
  const projectLogs = useLogStore((state) => state.getLogs(projectId));
  const serviceLogs = useServiceStore((state) =>
    serviceId ? state.serviceLogs[`${serviceId}-stdout`] || [] : []
  );
  const logs = isService ? serviceLogs : projectLogs;

  const clearProjectLogs = useLogStore((state) => state.clearLogs);
  const clearServiceLogs = useServiceStore((state) => state.clearServiceLogs);
  const fetchServiceLogs = useServiceStore((state) => state.fetchServiceLogs);

  // Fetch service logs on mount
  useEffect(() => {
    if (isService && serviceId) {
      fetchServiceLogs(serviceId, "stdout", 200);
    }
  }, [isService, serviceId, fetchServiceLogs]);

  // Display name
  const displayName = isService ? service?.name : project?.name;

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [logs]);

  const handleClearLogs = () => {
    if (isService && serviceId) {
      clearServiceLogs(serviceId, "stdout");
    } else {
      clearProjectLogs(projectId);
    }
  };

  const handleDownloadLogs = () => {
    const content = logs.map((log) => {
      const timestamp = new Date(log.timestamp).toLocaleTimeString();
      const stream = "stream" in log ? log.stream : "stdout";
      const text = "line" in log ? log.line : "message" in log ? log.message : "";
      return `[${timestamp}] [${stream}] ${text}`;
    }).join("\n");

    const blob = new Blob([content], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${displayName || projectId}-logs.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-slate-800 bg-slate-900">
        <div>
          <h2 className="text-lg font-semibold text-white">{displayName || "Logs"}</h2>
          <p className="text-sm text-slate-400">{logs.length} entries</p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleDownloadLogs}
            disabled={logs.length === 0}
            className="flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm
              bg-slate-700 hover:bg-slate-600 text-slate-300 transition-colors
              disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Download size={16} />
            Download
          </button>
          <button
            onClick={handleClearLogs}
            disabled={logs.length === 0}
            className="flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm
              bg-red-500/20 hover:bg-red-500/30 text-red-400 transition-colors
              disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Trash2 size={16} />
            Clear
          </button>
        </div>
      </div>

      {/* Log Content */}
      <div
        ref={containerRef}
        className="flex-1 overflow-auto p-4 font-mono text-sm bg-slate-950"
      >
        {logs.length === 0 ? (
          <p className="text-slate-500 text-center py-8">
            No logs yet. Start the {isService ? "service" : "project"} to see output.
          </p>
        ) : (
          logs.map((log, index) => {
            // Handle both ProcessLog (projects) and LogEntry (services)
            const text = "line" in log ? log.line : log.message;
            const isError = "stream" in log
              ? log.stream === "stderr"
              : "level" in log && log.level === "error";

            return (
              <div
                key={index}
                className={`py-0.5 ${isError ? "text-red-400" : "text-slate-300"}`}
              >
                <span className="text-slate-600 mr-2">
                  [{new Date(log.timestamp).toLocaleTimeString()}]
                </span>
                {text}
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
