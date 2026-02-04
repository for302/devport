import { useState, useCallback } from "react";
import { useServiceStore, useActivityLogStore } from "@/stores";
import { getServiceDisplayName } from "@/constants";

/**
 * Service action types
 */
export type ServiceAction = "start" | "stop" | "restart";

/**
 * Return type for the useServiceControl hook
 */
export interface UseServiceControlReturn {
  /** ID of the service currently being operated on */
  loadingServiceId: string | null;
  /** The action currently being performed */
  loadingAction: ServiceAction | null;
  /** Check if a specific service is loading */
  isServiceLoading: (serviceId: string) => boolean;
  /** Start a service */
  handleStartService: (serviceId: string) => Promise<void>;
  /** Stop a service */
  handleStopService: (serviceId: string) => Promise<void>;
  /** Restart a service */
  handleRestartService: (serviceId: string) => Promise<void>;
  /** Execute any service action */
  executeAction: (action: ServiceAction, serviceId: string) => Promise<void>;
}

/**
 * Hook for managing service control operations with loading state and activity logging
 *
 * @example
 * ```tsx
 * const { handleStartService, handleStopService, isServiceLoading } = useServiceControl();
 *
 * return (
 *   <button
 *     onClick={() => handleStartService("apache")}
 *     disabled={isServiceLoading("apache")}
 *   >
 *     Start Apache
 *   </button>
 * );
 * ```
 */
export function useServiceControl(): UseServiceControlReturn {
  const [loadingServiceId, setLoadingServiceId] = useState<string | null>(null);
  const [loadingAction, setLoadingAction] = useState<ServiceAction | null>(null);

  const { startService, stopService, restartService } = useServiceStore();
  const { addLog } = useActivityLogStore();

  const isServiceLoading = useCallback(
    (serviceId: string) => loadingServiceId === serviceId,
    [loadingServiceId]
  );

  const executeAction = useCallback(
    async (action: ServiceAction, serviceId: string) => {
      const serviceName = getServiceDisplayName(serviceId);
      setLoadingServiceId(serviceId);
      setLoadingAction(action);

      const actionVerb = {
        start: "Starting",
        stop: "Stopping",
        restart: "Restarting",
      }[action];

      const actionPastTense = {
        start: "started",
        stop: "stopped",
        restart: "restarted",
      }[action];

      addLog(serviceName, `${actionVerb} service...`, "info");

      try {
        switch (action) {
          case "start":
            await startService(serviceId);
            break;
          case "stop":
            await stopService(serviceId);
            break;
          case "restart":
            await restartService(serviceId);
            break;
        }
        addLog(serviceName, `Service ${actionPastTense} successfully`, "success");
      } catch (err) {
        addLog(serviceName, `Failed to ${action}: ${err}`, "error");
        throw err; // Re-throw so callers can handle if needed
      } finally {
        setLoadingServiceId(null);
        setLoadingAction(null);
      }
    },
    [startService, stopService, restartService, addLog]
  );

  const handleStartService = useCallback(
    (serviceId: string) => executeAction("start", serviceId),
    [executeAction]
  );

  const handleStopService = useCallback(
    (serviceId: string) => executeAction("stop", serviceId),
    [executeAction]
  );

  const handleRestartService = useCallback(
    (serviceId: string) => executeAction("restart", serviceId),
    [executeAction]
  );

  return {
    loadingServiceId,
    loadingAction,
    isServiceLoading,
    handleStartService,
    handleStopService,
    handleRestartService,
    executeAction,
  };
}
