import { useEffect, useRef, useState, useCallback } from "react";
import { X, Trash2, Download, RefreshCw, Radio, Circle } from "lucide-react";
import { useServiceStore } from "@/stores";
import { useLogStream } from "@/hooks";

interface ServiceLogViewerProps {
  serviceId: string;
  serviceName: string;
  onClose: () => void;
}

const LOG_LEVEL_COLORS: Record<string, string> = {
  ERROR: "text-red-400",
  WARN: "text-yellow-400",
  INFO: "text-blue-400",
  DEBUG: "text-slate-400",
};

export function ServiceLogViewer({
  serviceId,
  serviceName,
  onClose,
}: ServiceLogViewerProps) {
  const [logType, setLogType] = useState<"stdout" | "stderr">("stdout");
  const [autoScroll, setAutoScroll] = useState(true);
  const [useStreaming, setUseStreaming] = useState(true);
  const logContainerRef = useRef<HTMLDivElement>(null);

  const { serviceLogs, fetchServiceLogs, clearServiceLogs } = useServiceStore();
  const pollLogs = serviceLogs[`${serviceId}-${logType}`] || [];

  // Real-time streaming hook
  const {
    entries: streamedEntries,
    isStreaming,
    startStream,
    stopStream,
    clearEntries: clearStreamedEntries,
  } = useLogStream({
    serviceId,
    logType,
    maxEntries: 1000,
    onError: (error) => console.error("Stream error:", error),
  });

  // Determine which logs to display
  const logs = useStreaming ? streamedEntries : pollLogs;

  // Start streaming when component mounts and streaming is enabled
  useEffect(() => {
    if (useStreaming && !isStreaming) {
      // First fetch initial logs, then start streaming
      fetchServiceLogs(serviceId, logType, 200).then(() => {
        startStream();
      });
    } else if (!useStreaming && isStreaming) {
      stopStream();
    }
  }, [useStreaming, serviceId, logType, isStreaming, startStream, stopStream, fetchServiceLogs]);

  // Initialize streamed entries with existing logs when switching to streaming
  useEffect(() => {
    if (useStreaming && streamedEntries.length === 0 && pollLogs.length > 0) {
      // The stream will pick up new entries; the initial fetch handles history
    }
  }, [useStreaming, streamedEntries.length, pollLogs.length]);

  // Fallback polling when not streaming
  useEffect(() => {
    if (!useStreaming) {
      fetchServiceLogs(serviceId, logType, 200);
      const interval = setInterval(() => {
        fetchServiceLogs(serviceId, logType, 200);
      }, 3000);
      return () => clearInterval(interval);
    }
  }, [serviceId, logType, fetchServiceLogs, useStreaming]);

  // Auto-scroll effect
  useEffect(() => {
    if (autoScroll && logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
    }
  }, [logs, autoScroll]);

  // Handle log type change - reset stream
  const handleLogTypeChange = useCallback(
    async (newLogType: "stdout" | "stderr") => {
      if (newLogType === logType) return;

      if (isStreaming) {
        await stopStream();
      }
      clearStreamedEntries();
      setLogType(newLogType);
    },
    [logType, isStreaming, stopStream, clearStreamedEntries]
  );

  // Toggle streaming mode
  const handleToggleStreaming = useCallback(async () => {
    if (useStreaming) {
      await stopStream();
    }
    setUseStreaming(!useStreaming);
  }, [useStreaming, stopStream]);

  const handleClear = async () => {
    await clearServiceLogs(serviceId, logType);
    clearStreamedEntries();
  };

  const handleDownload = () => {
    const content = logs
      .map((log) => `[${log.timestamp}] [${log.level}] ${log.message}`)
      .join("\n");
    const blob = new Blob([content], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${serviceName}-${logType}-${new Date().toISOString()}.log`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleRefresh = useCallback(async () => {
    if (useStreaming) {
      // For streaming mode, restart the stream
      await stopStream();
      clearStreamedEntries();
      await fetchServiceLogs(serviceId, logType, 200);
      await startStream();
    } else {
      await fetchServiceLogs(serviceId, logType, 200);
    }
  }, [useStreaming, stopStream, clearStreamedEntries, fetchServiceLogs, serviceId, logType, startStream]);

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-slate-900 border border-slate-700 rounded-lg w-[90vw] max-w-4xl h-[80vh] flex flex-col">
        <div className="flex items-center justify-between p-4 border-b border-slate-700">
          <div className="flex items-center gap-4">
            <h2 className="text-lg font-semibold text-white">
              {serviceName} Logs
            </h2>
            <div className="flex bg-slate-800 rounded p-0.5">
              <button
                onClick={() => handleLogTypeChange("stdout")}
                className={`px-3 py-1 text-sm rounded transition-colors ${
                  logType === "stdout"
                    ? "bg-slate-700 text-white"
                    : "text-slate-400 hover:text-white"
                }`}
              >
                stdout
              </button>
              <button
                onClick={() => handleLogTypeChange("stderr")}
                className={`px-3 py-1 text-sm rounded transition-colors ${
                  logType === "stderr"
                    ? "bg-slate-700 text-white"
                    : "text-slate-400 hover:text-white"
                }`}
              >
                stderr
              </button>
            </div>
            {isStreaming && (
              <span className="flex items-center gap-1.5 text-xs text-green-400">
                <span className="w-2 h-2 bg-green-400 rounded-full animate-pulse" />
                Live
              </span>
            )}
          </div>
          <div className="flex items-center gap-2">
            <label className="flex items-center gap-2 text-sm text-slate-400">
              <input
                type="checkbox"
                checked={autoScroll}
                onChange={(e) => setAutoScroll(e.target.checked)}
                className="rounded border-slate-600 bg-slate-800 text-blue-600"
              />
              Auto-scroll
            </label>
            <button
              onClick={handleToggleStreaming}
              className={`p-2 rounded transition-colors ${
                useStreaming
                  ? "text-green-400 hover:text-green-300 hover:bg-slate-800"
                  : "text-slate-400 hover:text-white hover:bg-slate-800"
              }`}
              title={useStreaming ? "Disable real-time streaming" : "Enable real-time streaming"}
            >
              {useStreaming ? <Radio size={18} /> : <Circle size={18} />}
            </button>
            <button
              onClick={handleRefresh}
              className="p-2 text-slate-400 hover:text-white hover:bg-slate-800 rounded"
              title="Refresh"
            >
              <RefreshCw size={18} />
            </button>
            <button
              onClick={handleDownload}
              className="p-2 text-slate-400 hover:text-white hover:bg-slate-800 rounded"
              title="Download"
            >
              <Download size={18} />
            </button>
            <button
              onClick={handleClear}
              className="p-2 text-slate-400 hover:text-red-400 hover:bg-slate-800 rounded"
              title="Clear logs"
            >
              <Trash2 size={18} />
            </button>
            <button
              onClick={onClose}
              className="p-2 text-slate-400 hover:text-white hover:bg-slate-800 rounded"
            >
              <X size={18} />
            </button>
          </div>
        </div>

        <div
          ref={logContainerRef}
          className="flex-1 overflow-auto p-4 font-mono text-sm"
        >
          {logs.length === 0 ? (
            <div className="text-slate-500 text-center py-8">
              No logs available
            </div>
          ) : (
            <div className="space-y-0.5">
              {logs.map((log, index) => (
                <div
                  key={index}
                  className="flex gap-2 hover:bg-slate-800/50 px-2 py-0.5 rounded"
                >
                  <span className="text-slate-500 shrink-0">
                    {new Date(log.timestamp).toLocaleTimeString()}
                  </span>
                  <span
                    className={`shrink-0 w-12 ${
                      LOG_LEVEL_COLORS[log.level] || "text-slate-400"
                    }`}
                  >
                    [{log.level}]
                  </span>
                  <span className="text-slate-300 break-all">{log.message}</span>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="p-2 border-t border-slate-700 text-xs text-slate-500 flex items-center justify-center gap-4">
          <span>{logs.length} log entries</span>
          <span className="text-slate-600">|</span>
          <span>
            {useStreaming ? (
              isStreaming ? (
                <span className="text-green-400">Streaming</span>
              ) : (
                <span className="text-yellow-400">Connecting...</span>
              )
            ) : (
              <span>Polling (3s)</span>
            )}
          </span>
        </div>
      </div>
    </div>
  );
}
