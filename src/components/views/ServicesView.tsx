import { useEffect, useState, useCallback } from "react";
import {
  RefreshCw,
  ExternalLink,
  Database,
  CheckCircle,
  XCircle,
  Loader2,
  ToggleLeft,
  ToggleRight,
  Search,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useInventoryStore, useServiceStore, useActivityLogStore } from "@/stores";
import { InventorySection } from "@/components/inventory";
import { ServiceLogViewer, ServiceActivityLog } from "@/components/services";
import { CATEGORY_META, type InventoryCategory } from "@/types";

interface PhpMyAdminStatus {
  isAvailable: boolean;
  statusCode: number | null;
  responseTimeMs: number | null;
  error: string | null;
  checkedAt: string;
  url: string;
}

// Map CATEGORY_META keys to InventoryCategory types
const CATEGORY_KEY_TO_TYPE: Record<string, InventoryCategory> = {
  runtimes: "runtime",
  webServers: "webServer",
  databases: "database",
  buildTools: "buildTool",
  frameworks: "framework",
  packageManagers: "packageManager",
  devTools: "devTool",
};

// Map service IDs to display names
const SERVICE_DISPLAY_NAMES: Record<string, string> = {
  apache: "Apache",
  mariadb: "MariaDB",
};

export function ServicesView() {
  const [phpMyAdminStatus, setPhpMyAdminStatus] =
    useState<PhpMyAdminStatus | null>(null);
  const [isCheckingPhpMyAdmin, setIsCheckingPhpMyAdmin] = useState(false);
  const [loadingServiceId, setLoadingServiceId] = useState<string | null>(null);
  const [loadingAction, setLoadingAction] = useState<"start" | "stop" | "restart" | null>(null);
  const [logViewerState, setLogViewerState] = useState<{
    isOpen: boolean;
    serviceId: string;
    serviceName: string;
  } | null>(null);
  const [isActivityLogExpanded, setIsActivityLogExpanded] = useState(true);

  const {
    inventory,
    isScanning,
    error,
    showInstalledOnly,
    searchQuery,
    scanInventory,
    setShowInstalledOnly,
    setSearchQuery,
    clearError,
  } = useInventoryStore();

  const {
    services,
    fetchServices,
    startService,
    stopService,
    restartService,
  } = useServiceStore();

  const { addLog } = useActivityLogStore();

  const checkPhpMyAdminStatus = async () => {
    setIsCheckingPhpMyAdmin(true);
    try {
      const status = await invoke<PhpMyAdminStatus>("check_phpmyadmin_status");
      setPhpMyAdminStatus(status);
    } catch (err) {
      console.error("Failed to check phpMyAdmin status:", err);
      setPhpMyAdminStatus(null);
    } finally {
      setIsCheckingPhpMyAdmin(false);
    }
  };

  useEffect(() => {
    scanInventory();
    fetchServices();
    // Add initial log
    addLog("System", "Development environment scan started", "info");
  }, [scanInventory, fetchServices, addLog]);

  // Check phpMyAdmin status when Apache is running
  useEffect(() => {
    const apacheService = services.find((s) => s.id === "apache");
    if (apacheService?.status === "running") {
      checkPhpMyAdminStatus();
    } else {
      setPhpMyAdminStatus(null);
    }
  }, [services]);

  const handleRescan = () => {
    addLog("System", "Rescanning development environment...", "info");
    scanInventory();
  };

  // Service control handlers with activity logging
  const handleStartService = useCallback(
    async (serviceId: string) => {
      const serviceName = SERVICE_DISPLAY_NAMES[serviceId] || serviceId;
      setLoadingServiceId(serviceId);
      setLoadingAction("start");
      addLog(serviceName, "Starting service...", "info");

      try {
        await startService(serviceId);
        addLog(serviceName, "Service started successfully", "success");
      } catch (err) {
        addLog(serviceName, `Failed to start: ${err}`, "error");
      } finally {
        setLoadingServiceId(null);
        setLoadingAction(null);
      }
    },
    [startService, addLog]
  );

  const handleStopService = useCallback(
    async (serviceId: string) => {
      const serviceName = SERVICE_DISPLAY_NAMES[serviceId] || serviceId;
      setLoadingServiceId(serviceId);
      setLoadingAction("stop");
      addLog(serviceName, "Stopping service...", "info");

      try {
        await stopService(serviceId);
        addLog(serviceName, "Service stopped successfully", "success");
      } catch (err) {
        addLog(serviceName, `Failed to stop: ${err}`, "error");
      } finally {
        setLoadingServiceId(null);
        setLoadingAction(null);
      }
    },
    [stopService, addLog]
  );

  const handleRestartService = useCallback(
    async (serviceId: string) => {
      const serviceName = SERVICE_DISPLAY_NAMES[serviceId] || serviceId;
      setLoadingServiceId(serviceId);
      setLoadingAction("restart");
      addLog(serviceName, "Restarting service...", "info");
      addLog(serviceName, "Stopping service...", "info");

      try {
        await restartService(serviceId);
        addLog(serviceName, "Service restarted successfully", "success");
      } catch (err) {
        addLog(serviceName, `Failed to restart: ${err}`, "error");
      } finally {
        setLoadingServiceId(null);
        setLoadingAction(null);
      }
    },
    [restartService, addLog]
  );

  const handleViewLogs = useCallback((serviceId: string, serviceName: string) => {
    setLogViewerState({ isOpen: true, serviceId, serviceName });
  }, []);

  const handleCloseLogViewer = useCallback(() => {
    setLogViewerState(null);
  }, []);

  const handleSettings = useCallback((serviceId: string) => {
    const serviceName = SERVICE_DISPLAY_NAMES[serviceId] || serviceId;
    addLog(serviceName, "Opening settings...", "info");
    // TODO: Open settings modal for the service
    console.log("Open settings for service:", serviceId);
  }, [addLog]);

  const apacheService = services.find((s) => s.id === "apache");
  const isApacheRunning = apacheService?.status === "running";
  const apachePort = apacheService?.port || 80;
  const phpMyAdminUrl =
    phpMyAdminStatus?.url ||
    (apachePort === 80
      ? "http://localhost/phpmyadmin"
      : `http://localhost:${apachePort}/phpmyadmin`);

  // Calculate totals
  const totalInstalled = inventory
    ? inventory.runtimes.filter((i) => i.isInstalled).length +
      inventory.webServers.filter((i) => i.isInstalled).length +
      inventory.databases.filter((i) => i.isInstalled).length +
      inventory.buildTools.filter((i) => i.isInstalled).length +
      inventory.frameworks.filter((i) => i.isInstalled).length +
      inventory.packageManagers.filter((i) => i.isInstalled).length +
      inventory.devTools.filter((i) => i.isInstalled).length
    : 0;

  const totalItems = inventory
    ? inventory.runtimes.length +
      inventory.webServers.length +
      inventory.databases.length +
      inventory.buildTools.length +
      inventory.frameworks.length +
      inventory.packageManagers.length +
      inventory.devTools.length
    : 0;

  return (
    <div className="h-full flex flex-col">
      {/* Main content area - scrollable */}
      <div className="flex-1 overflow-auto p-6">
        <div className="max-w-5xl mx-auto">
          {/* Header */}
          <div className="flex items-center justify-between mb-6">
            <div>
              <h1 className="text-2xl font-bold text-white">
                Development Environment
              </h1>
              <p className="text-slate-400 mt-1">
                {totalInstalled} / {totalItems} tools installed
                {inventory?.scanDurationMs && (
                  <span className="text-slate-500 ml-2">
                    (scanned in {inventory.scanDurationMs}ms)
                  </span>
                )}
              </p>
            </div>
            <div className="flex items-center gap-3">
              {/* Search */}
              <div className="relative">
                <Search
                  size={16}
                  className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400"
                />
                <input
                  type="text"
                  placeholder="Search tools..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-9 pr-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:border-blue-500 w-48"
                />
              </div>

              {/* Installed Only Toggle */}
              <button
                onClick={() => setShowInstalledOnly(!showInstalledOnly)}
                className={`flex items-center gap-2 px-3 py-2 rounded-lg transition-colors ${
                  showInstalledOnly
                    ? "bg-blue-600/20 text-blue-400 border border-blue-600/50"
                    : "bg-slate-800 text-slate-400 border border-slate-700 hover:bg-slate-700"
                }`}
              >
                {showInstalledOnly ? (
                  <ToggleRight size={18} />
                ) : (
                  <ToggleLeft size={18} />
                )}
                Installed Only
              </button>

              {/* Rescan Button */}
              <button
                onClick={handleRescan}
                disabled={isScanning}
                className="flex items-center gap-2 px-4 py-2 bg-slate-800 hover:bg-slate-700 disabled:opacity-50 text-white rounded-lg transition-colors"
              >
                <RefreshCw size={18} className={isScanning ? "animate-spin" : ""} />
                Rescan
              </button>
            </div>
          </div>

          {/* Error */}
          {error && (
            <div className="mb-4 p-4 bg-red-900/20 border border-red-800 rounded-lg flex items-center justify-between">
              <span className="text-red-400">{error}</span>
              <button
                onClick={clearError}
                className="text-red-400 hover:text-red-300"
              >
                Dismiss
              </button>
            </div>
          )}

          {/* Loading state */}
          {isScanning && !inventory && (
            <div className="flex items-center justify-center py-20">
              <Loader2 size={32} className="animate-spin text-blue-400" />
              <span className="ml-3 text-slate-400">
                Scanning development environment...
              </span>
            </div>
          )}

          {/* Inventory Sections */}
          {inventory && (
            <div>
              {CATEGORY_META.map((meta) => (
                <InventorySection
                  key={meta.key}
                  title={meta.label}
                  icon={meta.icon}
                  iconColor={meta.color}
                  items={inventory[meta.key]}
                  category={CATEGORY_KEY_TO_TYPE[meta.key]}
                  showInstalledOnly={showInstalledOnly}
                  searchQuery={searchQuery}
                  services={services}
                  loadingServiceId={loadingServiceId}
                  loadingAction={loadingAction}
                  onStartService={handleStartService}
                  onStopService={handleStopService}
                  onRestartService={handleRestartService}
                  onViewLogs={handleViewLogs}
                  onSettings={handleSettings}
                />
              ))}
            </div>
          )}

          {/* Tools Section - phpMyAdmin */}
          <div className="mt-6 p-4 bg-slate-900 border border-slate-800 rounded-lg">
            <h2 className="text-lg font-semibold text-white mb-4">Tools</h2>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-lg bg-orange-500/20 flex items-center justify-center">
                  <Database size={20} className="text-orange-400" />
                </div>
                <div>
                  <div className="flex items-center gap-2">
                    <h3 className="font-medium text-white">phpMyAdmin</h3>
                    {isApacheRunning && (
                      <span className="flex items-center gap-1">
                        {isCheckingPhpMyAdmin ? (
                          <Loader2
                            size={14}
                            className="text-slate-400 animate-spin"
                          />
                        ) : phpMyAdminStatus?.isAvailable ? (
                          <CheckCircle size={14} className="text-green-400" />
                        ) : (
                          <XCircle size={14} className="text-red-400" />
                        )}
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-slate-400">{phpMyAdminUrl}</p>
                  {phpMyAdminStatus?.responseTimeMs && (
                    <p className="text-xs text-slate-500">
                      Response: {phpMyAdminStatus.responseTimeMs}ms
                    </p>
                  )}
                </div>
              </div>
              <div className="flex items-center gap-2">
                {isApacheRunning && (
                  <button
                    onClick={checkPhpMyAdminStatus}
                    disabled={isCheckingPhpMyAdmin}
                    className="flex items-center gap-1.5 px-3 py-2 bg-slate-700 hover:bg-slate-600 disabled:opacity-50 text-white text-sm rounded-lg transition-colors"
                    title="Check phpMyAdmin status"
                  >
                    <RefreshCw
                      size={14}
                      className={isCheckingPhpMyAdmin ? "animate-spin" : ""}
                    />
                  </button>
                )}
                <a
                  href={phpMyAdminUrl}
                  target="_blank"
                  rel="noopener noreferrer"
                  className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-colors ${
                    isApacheRunning
                      ? "bg-blue-600 hover:bg-blue-700 text-white"
                      : "bg-slate-700 text-slate-400 cursor-not-allowed"
                  }`}
                  onClick={(e) => !isApacheRunning && e.preventDefault()}
                >
                  <ExternalLink size={18} />
                  Open in Browser
                </a>
              </div>
            </div>
            {!isApacheRunning && (
              <p className="mt-2 text-sm text-yellow-500">
                Apache must be running to access phpMyAdmin
              </p>
            )}
            {isApacheRunning &&
              phpMyAdminStatus &&
              !phpMyAdminStatus.isAvailable && (
                <p className="mt-2 text-sm text-red-400">
                  phpMyAdmin is not responding: {phpMyAdminStatus.error}
                </p>
              )}
          </div>

          {/* Legend */}
          <div className="mt-6 p-4 bg-slate-800/50 border border-slate-700 rounded-lg">
            <h3 className="text-sm font-medium text-slate-400 mb-2">
              Status Legend
            </h3>
            <div className="flex flex-wrap gap-4 text-sm">
              <div className="flex items-center gap-2">
                <span className="w-2 h-2 rounded-full bg-green-500" />
                <span className="text-slate-300">Running</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="w-2 h-2 rounded-full bg-slate-500" />
                <span className="text-slate-300">Installed (Stopped)</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="w-2 h-2 rounded-full border border-slate-600 bg-transparent" />
                <span className="text-slate-300">Not Installed</span>
              </div>
            </div>
          </div>

          {/* Bottom padding for activity log */}
          <div className="h-4" />
        </div>
      </div>

      {/* Activity Log Panel - Fixed at bottom */}
      <div className="flex-shrink-0 border-t border-slate-700">
        <ServiceActivityLog
          isExpanded={isActivityLogExpanded}
          onToggleExpand={() => setIsActivityLogExpanded(!isActivityLogExpanded)}
        />
      </div>

      {/* Log Viewer Modal */}
      {logViewerState?.isOpen && (
        <ServiceLogViewer
          serviceId={logViewerState.serviceId}
          serviceName={logViewerState.serviceName}
          onClose={handleCloseLogViewer}
        />
      )}
    </div>
  );
}
