import { useState, useEffect } from "react";
import {
  Check,
  X,
  Circle,
  Play,
  Square,
  RotateCw,
  FileText,
  Settings,
  ChevronDown,
  ChevronRight,
  Lock,
  Globe,
  Download,
} from "lucide-react";
import type { InventoryItem, ServiceInfo } from "@/types";
import {
  INSTALL_SOURCE_LABELS,
  INSTALL_SOURCE_COLORS,
  SERVICE_STATUS_BG_COLORS,
  SERVICE_STATUS_LABELS,
} from "@/types";
import { getApachePorts, type ApachePortEntry } from "@/services/tauriCommands";

interface InventoryItemRowProps {
  item: InventoryItem;
  service?: ServiceInfo;
  onStart?: () => void;
  onStop?: () => void;
  onRestart?: () => void;
  onViewLogs?: () => void;
  onSettings?: () => void;
  onInstall?: () => void;
  isLoading?: boolean;
  loadingAction?: "start" | "stop" | "restart" | null;
}

export function InventoryItemRow({
  item,
  service,
  onStart,
  onStop,
  onRestart,
  onViewLogs,
  onSettings,
  onInstall,
  isLoading = false,
  loadingAction = null,
}: InventoryItemRowProps) {
  const isControllable = !!service;
  const isRunning = service?.status === "running";
  const isStopped = service?.status === "stopped";
  const isApache = item.id === "apache";

  const [isPortsExpanded, setIsPortsExpanded] = useState(false);
  const [apachePorts, setApachePorts] = useState<ApachePortEntry[]>([]);
  const [isLoadingPorts, setIsLoadingPorts] = useState(false);

  // Fetch Apache ports when expanded
  useEffect(() => {
    if (isApache && isPortsExpanded) {
      setIsLoadingPorts(true);
      getApachePorts()
        .then((ports) => setApachePorts(ports))
        .catch((err) => console.error("Failed to load Apache ports:", err))
        .finally(() => setIsLoadingPorts(false));
    }
  }, [isApache, isPortsExpanded]);

  // Auto-expand when Apache starts running
  useEffect(() => {
    if (isApache && isRunning) {
      setIsPortsExpanded(true);
    }
  }, [isApache, isRunning]);

  const handlePortBadgeClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsPortsExpanded(!isPortsExpanded);
  };

  return (
    <div className="flex flex-col border-b border-slate-700/50 last:border-b-0">
      <div className="flex items-center gap-3 px-4 py-3 hover:bg-slate-800/50 transition-colors">
        {/* Status indicator */}
        <div className="w-2 flex-shrink-0">
          {item.isInstalled ? (
            isControllable ? (
              <Circle
                className={`w-2 h-2 fill-current ${SERVICE_STATUS_BG_COLORS[service.status].replace("bg-", "text-")}`}
              />
            ) : item.isRunning ? (
              <Circle className="w-2 h-2 fill-green-500 text-green-500" />
            ) : (
              <Circle className="w-2 h-2 fill-slate-500 text-slate-500" />
            )
          ) : (
            <Circle className="w-2 h-2 fill-transparent text-slate-600" />
          )}
        </div>

        {/* Name - wider with min-width */}
        <div className="w-32 min-w-[8rem] flex-shrink-0">
          <span
            className={`font-medium truncate block ${item.isInstalled ? "text-white" : "text-slate-500"}`}
            title={item.name}
          >
            {item.name}
          </span>
        </div>

        {/* Version */}
        <div className="w-20 flex-shrink-0 text-center">
          {item.version ? (
            <span className="text-sm text-slate-300 font-mono truncate block" title={item.version}>
              {item.version}
            </span>
          ) : (
            <span className="text-sm text-slate-600">-</span>
          )}
        </div>

        {/* Install Source */}
        <div className="w-20 flex-shrink-0 text-center">
          {item.isInstalled ? (
            <span
              className={`text-xs px-2 py-0.5 rounded ${INSTALL_SOURCE_COLORS[item.installSource]} text-white`}
            >
              {INSTALL_SOURCE_LABELS[item.installSource]}
            </span>
          ) : (
            <span className="text-xs text-slate-600">-</span>
          )}
        </div>

        {/* Port */}
        <div className="w-14 flex-shrink-0 text-center">
          {item.port ? (
            <span className="text-sm text-slate-400 font-mono">{item.port}</span>
          ) : (
            <span className="text-sm text-slate-600">-</span>
          )}
        </div>

        {/* Status */}
        <div className="w-20 flex-shrink-0 text-center">
          {isControllable ? (
            <span
              className={`flex items-center justify-center gap-1.5 text-sm ${
                isRunning
                  ? "text-green-400"
                  : isStopped
                    ? "text-slate-400"
                    : "text-yellow-400"
              }`}
            >
              {SERVICE_STATUS_LABELS[service.status]}
            </span>
          ) : item.isInstalled ? (
            <span className="flex items-center justify-center gap-1.5 text-sm text-green-400">
              <Check size={14} />
              Installed
            </span>
          ) : (
            <span className="flex items-center justify-center gap-1.5 text-sm text-slate-500">
              <X size={14} />
              Not Found
            </span>
          )}
        </div>

        {/* PORT Badge - Apache only */}
        {isApache && item.isInstalled && (
          <button
            onClick={handlePortBadgeClick}
            className={`flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium transition-colors ${
              isRunning
                ? "bg-green-600/30 text-green-400 border border-green-600/50 hover:bg-green-600/40"
                : "bg-slate-700 text-slate-400 border border-slate-600 hover:bg-slate-600"
            }`}
            title={isPortsExpanded ? "Hide ports" : "Show ports"}
          >
            {isPortsExpanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
            PORT
            {apachePorts.length > 0 && (
              <span className="text-xs opacity-70">({apachePorts.length})</span>
            )}
          </button>
        )}

        {/* Flexible spacer */}
        <div className="flex-1" />

        {/* Actions & Controls - Only for controllable services */}
        {isControllable && item.isInstalled && (
          <div className="flex items-center gap-2">
            {/* Quick Actions */}
            <div className="flex items-center gap-1">
              <button
                onClick={onViewLogs}
                className="p-1.5 text-slate-400 hover:text-blue-400 hover:bg-slate-700 rounded transition-colors"
                title="View Logs"
              >
                <FileText size={14} />
              </button>
              <button
                onClick={onSettings}
                className="p-1.5 text-slate-400 hover:text-blue-400 hover:bg-slate-700 rounded transition-colors"
                title="Settings"
              >
                <Settings size={14} />
              </button>
            </div>

            {/* Separator */}
            <div className="w-px h-6 bg-slate-700" />

            {/* Control buttons - always visible */}
            <div className="flex items-center gap-1">
              {/* Start button - only when stopped */}
              {!isRunning && (
                <button
                  onClick={onStart}
                  disabled={isLoading}
                  className={`p-1.5 rounded transition-colors ${
                    isLoading && loadingAction === "start"
                      ? "text-green-400 bg-slate-700"
                      : "text-slate-400 hover:text-green-400 hover:bg-slate-700"
                  } disabled:opacity-50`}
                  title="Start"
                >
                  <Play size={14} className={isLoading && loadingAction === "start" ? "animate-pulse" : ""} />
                </button>
              )}

              {/* Restart button - only when running */}
              {isRunning && (
                <button
                  onClick={onRestart}
                  disabled={isLoading}
                  className={`p-1.5 rounded transition-colors ${
                    isLoading && loadingAction === "restart"
                      ? "text-yellow-400 bg-slate-700"
                      : "text-slate-400 hover:text-yellow-400 hover:bg-slate-700"
                  } disabled:opacity-50`}
                  title="Restart"
                >
                  <RotateCw size={14} className={isLoading && loadingAction === "restart" ? "animate-spin" : ""} />
                </button>
              )}

              {/* Stop button - always visible, disabled when stopped */}
              <button
                onClick={onStop}
                disabled={!isRunning || isLoading}
                className={`p-1.5 rounded transition-colors ${
                  isLoading && loadingAction === "stop"
                    ? "text-red-400 bg-slate-700"
                    : isRunning
                      ? "text-slate-400 hover:text-red-400 hover:bg-slate-700"
                      : "text-slate-600 cursor-not-allowed"
                } disabled:opacity-50`}
                title={isRunning ? "Stop" : "Service is stopped"}
              >
                <Square size={14} className={isLoading && loadingAction === "stop" ? "animate-pulse" : ""} />
              </button>
            </div>
          </div>
        )}

        {/* Install button for non-installed items */}
        {!item.isInstalled && onInstall && (
          <button
            onClick={onInstall}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded-lg transition-colors"
            title={`Install ${item.name}`}
          >
            <Download size={14} />
            Install
          </button>
        )}

        {/* Empty space for non-controllable installed items */}
        {!isControllable && item.isInstalled && <div className="w-32 flex-shrink-0" />}
      </div>

      {/* Progress bar for loading state */}
      {isLoading && isControllable && (
        <div className="px-4 pb-2">
          <div className="h-1 bg-slate-700 rounded-full overflow-hidden">
            <div
              className={`h-full rounded-full animate-pulse ${
                loadingAction === "start"
                  ? "bg-green-500"
                  : loadingAction === "stop"
                    ? "bg-red-500"
                    : "bg-yellow-500"
              }`}
              style={{
                width: "100%",
                animation: "progress-indeterminate 1.5s ease-in-out infinite",
              }}
            />
          </div>
          <style>{`
            @keyframes progress-indeterminate {
              0% { transform: translateX(-100%); }
              50% { transform: translateX(0%); }
              100% { transform: translateX(100%); }
            }
          `}</style>
        </div>
      )}

      {/* Apache Ports List - Expandable */}
      {isApache && isPortsExpanded && item.isInstalled && (
        <div className="px-4 pb-3 bg-slate-900/50">
          <div className="ml-5 pl-4 border-l-2 border-slate-700">
            <div className="text-xs text-slate-500 uppercase tracking-wider mb-2 pt-2">
              Virtual Hosts
            </div>
            {isLoadingPorts ? (
              <div className="flex items-center gap-2 py-2 text-sm text-slate-400">
                <div className="w-4 h-4 border-2 border-slate-500 border-t-transparent rounded-full animate-spin" />
                Loading configuration...
              </div>
            ) : apachePorts.length === 0 ? (
              <div className="py-2 text-sm text-slate-500">
                No virtual hosts found in config
              </div>
            ) : (
              <div className="space-y-1">
                {apachePorts.map((portEntry, idx) => (
                  <div
                    key={`${portEntry.port}-${portEntry.domain}-${idx}`}
                    className={`py-1.5 px-3 rounded flex items-center gap-3 ${
                      isRunning ? "bg-slate-800/70" : "bg-slate-800/40"
                    }`}
                    title={portEntry.serverAlias.length > 0 ? `Alias: ${portEntry.serverAlias.join(", ")}` : undefined}
                  >
                    {/* Port Icon */}
                    {portEntry.isSsl ? (
                      <Lock size={14} className="text-yellow-500 flex-shrink-0" />
                    ) : (
                      <Globe size={14} className="text-blue-400 flex-shrink-0" />
                    )}

                    {/* Port Number */}
                    <span className={`font-mono text-sm font-semibold w-12 flex-shrink-0 ${
                      isRunning ? "text-white" : "text-slate-400"
                    }`}>
                      {portEntry.port}
                    </span>

                    {/* URL - clickable when running */}
                    {isRunning ? (
                      <a
                        href={portEntry.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-sm text-blue-400 hover:text-blue-300 hover:underline truncate min-w-0"
                        title={portEntry.url}
                      >
                        {portEntry.url}
                      </a>
                    ) : (
                      <span className="text-sm text-slate-500 truncate min-w-0" title={portEntry.url}>
                        {portEntry.url}
                      </span>
                    )}

                    {/* Separator */}
                    <span className="text-slate-600 flex-shrink-0">│</span>

                    {/* DocumentRoot */}
                    <span className="text-xs text-slate-400 font-mono truncate min-w-0 flex-1" title={portEntry.documentRoot}>
                      {portEntry.documentRoot}
                    </span>

                    {/* Status dot when running */}
                    {isRunning && (
                      <span className="flex items-center gap-1.5 text-xs text-green-400 flex-shrink-0">
                        <span className="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
                        Active
                      </span>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
