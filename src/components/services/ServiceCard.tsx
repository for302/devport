import {
  Play,
  Square,
  RefreshCw,
  Settings,
  ExternalLink,
  ScrollText,
  Database,
} from "lucide-react";
import type { ServiceInfo } from "@/types";
import {
  SERVICE_STATUS_COLORS,
  SERVICE_STATUS_BG_COLORS,
  SERVICE_STATUS_LABELS,
  SERVICE_TYPE_ICONS,
} from "@/types/service";

interface ServiceCardProps {
  service: ServiceInfo;
  onStart: () => void;
  onStop: () => void;
  onRestart: () => void;
  onViewLogs: () => void;
  onSettings: () => void;
  isLoading?: boolean;
}

export function ServiceCard({
  service,
  onStart,
  onStop,
  onRestart,
  onViewLogs,
  onSettings,
  isLoading,
}: ServiceCardProps) {
  const isRunning = service.status === "running";
  const hasError = service.status === "error";
  const isUnhealthy = service.status === "unhealthy";
  const isNotInstalled = service.status === "notinstalled" || !service.installed;

  const statusDotClass = SERVICE_STATUS_BG_COLORS[service.status];
  const statusTextClass = SERVICE_STATUS_COLORS[service.status];

  return (
    <div className="bg-slate-900 border border-slate-800 rounded-lg p-4">
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center gap-3">
          <span className="text-2xl">
            {SERVICE_TYPE_ICONS[service.serviceType]}
          </span>
          <div>
            <h3 className="text-lg font-semibold text-white">{service.name}</h3>
            <div className="flex items-center gap-2 mt-1">
              <span className={`w-2 h-2 rounded-full ${statusDotClass}`} />
              <span className={`text-sm ${statusTextClass}`}>
                {SERVICE_STATUS_LABELS[service.status]}
              </span>
              {service.pid && (
                <span className="text-xs text-slate-500">
                  PID: {service.pid}
                </span>
              )}
            </div>
          </div>
        </div>
        <div className="text-right">
          <span className="text-sm text-slate-400">Port</span>
          <p className="text-lg font-mono text-white">{service.port}</p>
        </div>
      </div>

      {hasError && service.errorMessage && (
        <div className="mb-4 p-2 bg-red-900/20 border border-red-800 rounded text-sm text-red-400">
          {service.errorMessage}
        </div>
      )}

      {isUnhealthy && (
        <div className="mb-4 p-2 bg-yellow-900/20 border border-yellow-800 rounded text-sm text-yellow-400">
          Service is running but not responding to health checks
        </div>
      )}

      {isNotInstalled && (
        <div className="mb-4 p-2 bg-slate-800/50 border border-slate-700 rounded text-sm text-slate-400">
          Service is not installed. Install the required binaries to use this service.
        </div>
      )}

      <div className="flex flex-wrap gap-2">
        {isRunning ? (
          <>
            <button
              onClick={onStop}
              disabled={isLoading}
              className="flex items-center gap-1.5 px-3 py-1.5 bg-red-600 hover:bg-red-700 disabled:opacity-50 text-white text-sm rounded transition-colors"
            >
              <Square size={14} />
              Stop
            </button>
            <button
              onClick={onRestart}
              disabled={isLoading}
              className="flex items-center gap-1.5 px-3 py-1.5 bg-slate-700 hover:bg-slate-600 disabled:opacity-50 text-white text-sm rounded transition-colors"
            >
              <RefreshCw size={14} />
              Restart
            </button>
          </>
        ) : (
          <button
            onClick={onStart}
            disabled={isLoading || isNotInstalled}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-green-600 hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed text-white text-sm rounded transition-colors"
          >
            <Play size={14} />
            Start
          </button>
        )}

        <button
          onClick={onViewLogs}
          className="flex items-center gap-1.5 px-3 py-1.5 bg-slate-700 hover:bg-slate-600 text-white text-sm rounded transition-colors"
        >
          <ScrollText size={14} />
          Logs
        </button>

        <button
          onClick={onSettings}
          disabled={!service.configFiles || service.configFiles.length === 0}
          className="flex items-center gap-1.5 px-3 py-1.5 bg-slate-700 hover:bg-slate-600 disabled:opacity-50 disabled:cursor-not-allowed text-white text-sm rounded transition-colors"
          title={service.configFiles?.length ? `${service.configFiles.length} config file(s)` : 'No config files'}
        >
          <Settings size={14} />
          {service.configFiles?.length > 0 && (
            <span className="text-xs text-slate-400">
              {service.configFiles.length}
            </span>
          )}
        </button>

        {isRunning && service.id === "apache" && (
          <a
            href={`http://localhost:${service.port}`}
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-1.5 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded transition-colors ml-auto"
          >
            <ExternalLink size={14} />
            Open
          </a>
        )}

        {isRunning && service.id === "mariadb" && (
          <a
            href="http://localhost:8080/phpmyadmin"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-1.5 px-3 py-1.5 bg-orange-600 hover:bg-orange-700 text-white text-sm rounded transition-colors ml-auto"
            title="Open phpMyAdmin"
          >
            <Database size={14} />
            phpMyAdmin
          </a>
        )}
      </div>

      <div className="mt-4 pt-4 border-t border-slate-800 flex items-center gap-4 text-xs text-slate-500">
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={service.autoStart}
            onChange={() => {}}
            className="rounded border-slate-600 bg-slate-800 text-blue-600 focus:ring-blue-500"
          />
          Auto Start
        </label>
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={service.autoRestart}
            onChange={() => {}}
            className="rounded border-slate-600 bg-slate-800 text-blue-600 focus:ring-blue-500"
          />
          Auto Restart
        </label>
        {service.lastStarted && (
          <span className="ml-auto">
            Started: {new Date(service.lastStarted).toLocaleString()}
          </span>
        )}
      </div>
    </div>
  );
}
