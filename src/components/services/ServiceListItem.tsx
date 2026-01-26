import {
  Play,
  Square,
  RefreshCw,
  Settings,
  ExternalLink,
  ScrollText,
  Database,
} from "lucide-react";
import type { ServiceInfo, SupportedService } from "@/types";
import {
  SERVICE_STATUS_COLORS,
  SERVICE_STATUS_BG_COLORS,
  SERVICE_STATUS_LABELS,
  SERVICE_TYPE_ICONS,
} from "@/types/service";

interface ServiceListItemProps {
  service?: ServiceInfo;
  supportedService: SupportedService;
  onStart: () => void;
  onStop: () => void;
  onRestart: () => void;
  onViewLogs: () => void;
  onSettings: () => void;
  isLoading?: boolean;
}

export function ServiceListItem({
  service,
  supportedService,
  onStart,
  onStop,
  onRestart,
  onViewLogs,
  onSettings,
  isLoading,
}: ServiceListItemProps) {
  const isInstalled = service?.installed ?? false;
  const status = service?.status ?? "notinstalled";
  const isRunning = status === "running";
  const isNotInstalled = status === "notinstalled" || !isInstalled;

  const statusDotClass = SERVICE_STATUS_BG_COLORS[status];
  const statusTextClass = SERVICE_STATUS_COLORS[status];

  return (
    <div className="flex items-center gap-4 px-4 py-3 hover:bg-slate-800/50 border-b border-slate-700 last:border-b-0">
      {/* Status Dot */}
      <div className="w-2 flex-shrink-0">
        <span className={`block w-2 h-2 rounded-full ${statusDotClass}`} />
      </div>

      {/* Type Icon */}
      <div className="w-10 flex-shrink-0 text-center">
        <span className="text-lg">
          {SERVICE_TYPE_ICONS[supportedService.serviceType]}
        </span>
      </div>

      {/* Name & Description */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-medium text-white truncate">
            {supportedService.name}
          </span>
          {service?.pid && (
            <span className="text-xs text-slate-500">PID: {service.pid}</span>
          )}
        </div>
        <span className="text-xs text-slate-500 truncate block">
          {supportedService.description}
        </span>
      </div>

      {/* Status */}
      <div className="w-24 flex-shrink-0 text-center">
        <span className={`text-sm ${statusTextClass}`}>
          {SERVICE_STATUS_LABELS[status]}
        </span>
      </div>

      {/* Port */}
      <div className="w-16 text-center flex-shrink-0">
        <span className="font-mono text-sm text-slate-300">
          {service?.port ?? supportedService.defaultPort}
        </span>
      </div>

      {/* Quick Actions */}
      <div className="w-24 flex items-center justify-center gap-1 flex-shrink-0">
        <button
          onClick={onViewLogs}
          disabled={isNotInstalled}
          className="p-1.5 rounded hover:bg-slate-700 disabled:opacity-30 disabled:cursor-not-allowed text-slate-400 hover:text-white transition-colors"
          title="View Logs"
        >
          <ScrollText size={16} />
        </button>
        <button
          onClick={onSettings}
          disabled={isNotInstalled || !service?.configFiles?.length}
          className="p-1.5 rounded hover:bg-slate-700 disabled:opacity-30 disabled:cursor-not-allowed text-slate-400 hover:text-white transition-colors"
          title="Settings"
        >
          <Settings size={16} />
        </button>
        {isRunning && service?.id === "apache" && (
          <a
            href={`http://localhost:${service.port}`}
            target="_blank"
            rel="noopener noreferrer"
            className="p-1.5 rounded hover:bg-slate-700 text-blue-400 hover:text-blue-300 transition-colors"
            title="Open in Browser"
          >
            <ExternalLink size={16} />
          </a>
        )}
        {isRunning && service?.id === "mariadb" && (
          <a
            href="http://localhost:8080/phpmyadmin"
            target="_blank"
            rel="noopener noreferrer"
            className="p-1.5 rounded hover:bg-slate-700 text-orange-400 hover:text-orange-300 transition-colors"
            title="Open phpMyAdmin"
          >
            <Database size={16} />
          </a>
        )}
      </div>

      {/* Control */}
      <div className="w-24 flex items-center justify-center flex-shrink-0">
        {isRunning ? (
          <div className="flex items-center gap-1">
            <button
              onClick={onStop}
              disabled={isLoading}
              className="p-1.5 rounded bg-red-600/20 hover:bg-red-600/40 disabled:opacity-50 text-red-400 transition-colors"
              title="Stop"
            >
              <Square size={16} />
            </button>
            <button
              onClick={onRestart}
              disabled={isLoading}
              className="p-1.5 rounded bg-slate-700 hover:bg-slate-600 disabled:opacity-50 text-slate-300 transition-colors"
              title="Restart"
            >
              <RefreshCw size={16} />
            </button>
          </div>
        ) : (
          <button
            onClick={onStart}
            disabled={isLoading || isNotInstalled}
            className="p-1.5 rounded bg-green-600/20 hover:bg-green-600/40 disabled:opacity-30 disabled:cursor-not-allowed text-green-400 transition-colors"
            title={isNotInstalled ? "Not Installed" : "Start"}
          >
            <Play size={16} />
          </button>
        )}
      </div>
    </div>
  );
}
