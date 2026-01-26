import { useEffect, useRef } from "react";
import { Trash2, ChevronDown, ChevronUp } from "lucide-react";
import { useActivityLogStore, type ActivityLogLevel } from "@/stores";

interface ServiceActivityLogProps {
  isExpanded: boolean;
  onToggleExpand: () => void;
}

const LEVEL_COLORS: Record<ActivityLogLevel, string> = {
  info: "text-slate-300",
  success: "text-green-400",
  warning: "text-yellow-400",
  error: "text-red-400",
};

const SERVICE_COLORS: Record<string, string> = {
  Apache: "text-orange-400",
  MariaDB: "text-blue-400",
  MySQL: "text-blue-400",
  System: "text-purple-400",
};

export function ServiceActivityLog({ isExpanded, onToggleExpand }: ServiceActivityLogProps) {
  const { logs, clearLogs } = useActivityLogStore();
  const logContainerRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (logContainerRef.current && isExpanded) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
    }
  }, [logs, isExpanded]);

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString("ko-KR", {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  };

  const getServiceColor = (service: string) => {
    return SERVICE_COLORS[service] || "text-cyan-400";
  };

  return (
    <div className="bg-slate-900 border border-slate-700 rounded-lg overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 bg-slate-800 border-b border-slate-700">
        <button
          onClick={onToggleExpand}
          className="flex items-center gap-2 text-sm font-medium text-white hover:text-blue-400 transition-colors"
        >
          {isExpanded ? <ChevronDown size={16} /> : <ChevronUp size={16} />}
          Activity Log
          {logs.length > 0 && (
            <span className="text-xs text-slate-500">({logs.length})</span>
          )}
        </button>
        <button
          onClick={clearLogs}
          className="p-1 text-slate-400 hover:text-red-400 hover:bg-slate-700 rounded transition-colors"
          title="Clear logs"
        >
          <Trash2 size={14} />
        </button>
      </div>

      {/* Log content */}
      {isExpanded && (
        <div
          ref={logContainerRef}
          className="h-48 overflow-auto font-mono text-xs bg-slate-950"
        >
          {logs.length === 0 ? (
            <div className="flex items-center justify-center h-full text-slate-500">
              No activity logs yet
            </div>
          ) : (
            <div className="p-2">
              {logs.map((log) => (
                <div
                  key={log.id}
                  className={`flex gap-2 py-0.5 ${LEVEL_COLORS[log.level]}`}
                >
                  <span className="text-slate-500 shrink-0">
                    {formatTime(log.timestamp)}
                  </span>
                  <span className={`shrink-0 font-semibold ${getServiceColor(log.service)}`}>
                    [{log.service}]
                  </span>
                  <span className={log.level === "error" ? "text-red-400" : log.level === "warning" ? "text-yellow-400" : "text-slate-300"}>
                    {log.message}
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
