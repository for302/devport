import { useState, useCallback } from "react";
import { Play, Square, RotateCcw, Loader2 } from "lucide-react";
import { useServiceStore, useActivityLogStore } from "@/stores";

interface ServiceControlProps {
  serviceId: string;
  serviceName: string;
  collapsed?: boolean;
}

function ServiceControl({ serviceId, serviceName, collapsed = false }: ServiceControlProps) {
  const { services, startService, stopService, restartService } = useServiceStore();
  const { addLog } = useActivityLogStore();

  const service = services.find((s) => s.id === serviceId);
  const isInstalled = !!service;
  const isRunning = service?.status === "running";

  const [isLoading, setIsLoading] = useState(false);

  const handleStart = useCallback(async () => {
    setIsLoading(true);
    addLog(serviceName, "Starting service...", "info");
    try {
      await startService(serviceId);
      addLog(serviceName, "Service started successfully", "success");
    } catch (err) {
      addLog(serviceName, `Failed to start: ${err}`, "error");
    } finally {
      setIsLoading(false);
    }
  }, [serviceId, serviceName, startService, addLog]);

  const handleStop = useCallback(async () => {
    setIsLoading(true);
    addLog(serviceName, "Stopping service...", "info");
    try {
      await stopService(serviceId);
      addLog(serviceName, "Service stopped successfully", "success");
    } catch (err) {
      addLog(serviceName, `Failed to stop: ${err}`, "error");
    } finally {
      setIsLoading(false);
    }
  }, [serviceId, serviceName, stopService, addLog]);

  const handleRestart = useCallback(async () => {
    setIsLoading(true);
    addLog(serviceName, "Restarting service...", "info");
    try {
      await restartService(serviceId);
      addLog(serviceName, "Service restarted successfully", "success");
    } catch (err) {
      addLog(serviceName, `Failed to restart: ${err}`, "error");
    } finally {
      setIsLoading(false);
    }
  }, [serviceId, serviceName, restartService, addLog]);

  if (!isInstalled) {
    return null;
  }

  // Collapsed view - just show status indicator
  if (collapsed) {
    return (
      <div
        className="flex items-center justify-center p-2"
        title={`${serviceName}: ${isRunning ? "Running" : "Stopped"}`}
      >
        <div
          className={`w-2 h-2 rounded-full ${isRunning ? "bg-green-500" : "bg-slate-500"}`}
        />
      </div>
    );
  }

  return (
    <div className="flex items-center justify-between px-3 py-1.5 bg-slate-900/50 rounded-lg">
      {/* Service Name */}
      <span className="text-sm font-medium text-slate-300 w-20">{serviceName}</span>

      {isLoading ? (
        <div className="flex items-center gap-1">
          <Loader2 size={14} className="text-yellow-400 animate-spin" />
        </div>
      ) : (
        <div className="flex items-center gap-1">
          {isRunning ? (
            <>
              {/* Restart button */}
              <button
                onClick={handleRestart}
                disabled={isLoading}
                className="p-1 rounded hover:bg-slate-700 text-yellow-400 hover:text-yellow-300 transition-colors disabled:opacity-50"
                title={`Restart ${serviceName}`}
              >
                <RotateCcw size={14} />
              </button>
              {/* Stop button */}
              <button
                onClick={handleStop}
                disabled={isLoading}
                className="p-1 rounded hover:bg-red-500/20 text-red-400 hover:text-red-300 transition-colors disabled:opacity-50"
                title={`Stop ${serviceName}`}
              >
                <Square size={14} />
              </button>
            </>
          ) : (
            <>
              {/* Play button */}
              <button
                onClick={handleStart}
                disabled={isLoading}
                className="p-1 rounded hover:bg-green-500/20 text-green-400 hover:text-green-300 transition-colors disabled:opacity-50"
                title={`Start ${serviceName}`}
              >
                <Play size={14} />
              </button>
              {/* Stop button - disabled */}
              <button
                disabled
                className="p-1 rounded text-slate-600 cursor-not-allowed"
                title="Not running"
              >
                <Square size={14} />
              </button>
            </>
          )}
          {/* Status indicator */}
          <div
            className={`w-2 h-2 rounded-full ml-1 ${isRunning ? "bg-green-500" : "bg-slate-500"}`}
            title={isRunning ? "Running" : "Stopped"}
          />
        </div>
      )}
    </div>
  );
}

// Service definitions in order
const SERVICES = [
  { id: "apache", name: "Apache" },
  { id: "mariadb", name: "MariaDB" },
] as const;

interface ServiceQuickControlsProps {
  collapsed?: boolean;
}

export function ServiceQuickControls({ collapsed = false }: ServiceQuickControlsProps) {
  const services = useServiceStore((state) => state.services);

  // Check which services are installed
  const installedServices = SERVICES.filter(
    (svc) => services.some((s) => s.id === svc.id)
  );

  if (installedServices.length === 0) {
    return null;
  }

  return (
    <div className={`space-y-1 ${collapsed ? "px-2" : ""}`}>
      {!collapsed && (
        <p className="text-xs text-slate-500 uppercase tracking-wider mb-2">Services</p>
      )}
      {installedServices.map((svc) => (
        <ServiceControl
          key={svc.id}
          serviceId={svc.id}
          serviceName={svc.name}
          collapsed={collapsed}
        />
      ))}
    </div>
  );
}
