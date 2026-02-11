import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ServiceInfo, LogEntry, LogReadResult } from "@/types";
import { useActivityLogStore } from "./activityLogStore";

interface ServiceLogPayload {
  serviceId: string;
  serviceName: string;
  line: string;
  stream: "stdout" | "stderr";
  level: string;
  timestamp: string;
}

interface ServiceState {
  services: ServiceInfo[];
  selectedServiceId: string | null;
  serviceLogs: Record<string, LogEntry[]>;
  isLoading: boolean;
  error: string | null;
}

interface ServiceActions {
  fetchServices: () => Promise<void>;
  startService: (id: string) => Promise<void>;
  stopService: (id: string) => Promise<void>;
  restartService: (id: string) => Promise<void>;
  checkHealth: (id: string) => Promise<void>;
  setSelectedService: (id: string | null) => void;
  fetchServiceLogs: (serviceId: string, logType?: string, lines?: number) => Promise<void>;
  clearServiceLogs: (serviceId: string, logType?: string) => Promise<void>;
  setAutoStart: (id: string, autoStart: boolean) => Promise<void>;
  setAutoRestart: (id: string, autoRestart: boolean) => Promise<void>;
  addServiceLog: (serviceId: string, entry: LogEntry) => void;
  clearError: () => void;
}

type ServiceStore = ServiceState & ServiceActions;

export const useServiceStore = create<ServiceStore>()(
  immer((set) => ({
    services: [],
    selectedServiceId: null,
    serviceLogs: {},
    isLoading: false,
    error: null,

    fetchServices: async () => {
      set((state) => {
        state.isLoading = true;
        state.error = null;
      });

      try {
        const services = await invoke<ServiceInfo[]>("get_services");
        set((state) => {
          state.services = services;
          state.isLoading = false;
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
          state.isLoading = false;
        });
      }
    },

    startService: async (id: string) => {
      set((state) => {
        state.isLoading = true;
        state.error = null;
      });

      try {
        const service = await invoke<ServiceInfo>("start_service", { id });
        set((state) => {
          const index = state.services.findIndex((s) => s.id === id);
          if (index !== -1) {
            state.services[index] = service;
          }
          state.isLoading = false;
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
          state.isLoading = false;
        });
      }
    },

    stopService: async (id: string) => {
      set((state) => {
        state.isLoading = true;
        state.error = null;
      });

      try {
        const service = await invoke<ServiceInfo>("stop_service", { id });
        set((state) => {
          const index = state.services.findIndex((s) => s.id === id);
          if (index !== -1) {
            state.services[index] = service;
          }
          state.isLoading = false;
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
          state.isLoading = false;
        });
      }
    },

    restartService: async (id: string) => {
      set((state) => {
        state.isLoading = true;
        state.error = null;
      });

      try {
        const service = await invoke<ServiceInfo>("restart_service", { id });
        set((state) => {
          const index = state.services.findIndex((s) => s.id === id);
          if (index !== -1) {
            state.services[index] = service;
          }
          state.isLoading = false;
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
          state.isLoading = false;
        });
      }
    },

    checkHealth: async (id: string) => {
      try {
        const status = await invoke<string>("check_service_health", { id });
        set((state) => {
          const index = state.services.findIndex((s) => s.id === id);
          if (index !== -1) {
            state.services[index].status = status as ServiceInfo["status"];
          }
        });
      } catch (error) {
        console.error("Health check failed:", error);
      }
    },

    setSelectedService: (id: string | null) => {
      set((state) => {
        state.selectedServiceId = id;
      });
    },

    fetchServiceLogs: async (serviceId: string, logType = "stdout", lines = 100) => {
      try {
        const result = await invoke<LogReadResult>("get_service_logs", {
          serviceId,
          logType,
          lines,
        });
        set((state) => {
          state.serviceLogs[`${serviceId}-${logType}`] = result.entries;
        });
      } catch (error) {
        console.error("Failed to fetch logs:", error);
        throw error; // Re-throw so caller can handle it
      }
    },

    clearServiceLogs: async (serviceId: string, logType = "stdout") => {
      try {
        await invoke("clear_service_logs", { serviceId, logType });
        set((state) => {
          state.serviceLogs[`${serviceId}-${logType}`] = [];
        });
      } catch (error) {
        console.error("Failed to clear logs:", error);
      }
    },

    setAutoStart: async (id: string, autoStart: boolean) => {
      try {
        const service = await invoke<ServiceInfo>("set_service_auto_start", {
          id,
          autoStart,
        });
        set((state) => {
          const index = state.services.findIndex((s) => s.id === id);
          if (index !== -1) {
            state.services[index] = service;
          }
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
        });
      }
    },

    setAutoRestart: async (id: string, autoRestart: boolean) => {
      try {
        const service = await invoke<ServiceInfo>("set_service_auto_restart", {
          id,
          autoRestart,
        });
        set((state) => {
          const index = state.services.findIndex((s) => s.id === id);
          if (index !== -1) {
            state.services[index] = service;
          }
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
        });
      }
    },

    addServiceLog: (serviceId: string, entry: LogEntry) => {
      set((state) => {
        const key = `${serviceId}-stdout`;
        if (!state.serviceLogs[key]) {
          state.serviceLogs[key] = [];
        }
        state.serviceLogs[key].push(entry);
        // Keep only the last 1000 entries
        if (state.serviceLogs[key].length > 1000) {
          state.serviceLogs[key] = state.serviceLogs[key].slice(-1000);
        }
      });
    },

    clearError: () => {
      set((state) => {
        state.error = null;
      });
    },
  }))
);

/**
 * Initialize the service log listener for real-time log streaming
 * @returns Unlisten function to clean up the listener
 */
export async function initServiceLogListener(): Promise<UnlistenFn> {
  return await listen<ServiceLogPayload>("service-log", (event) => {
    const { serviceId, serviceName, line, stream, level, timestamp } = event.payload;

    // Map backend level to frontend ActivityLogLevel
    const mapLevel = (lvl: string): "info" | "success" | "warning" | "error" => {
      if (lvl === "success") return "success";
      if (lvl === "error") return "error";
      if (lvl === "warning") return "warning";
      return "info";
    };

    const mappedLevel = mapLevel(level);

    // Add to serviceLogs store
    useServiceStore.getState().addServiceLog(serviceId, {
      timestamp,
      level: mappedLevel,
      message: line,
      source: serviceName,
    });

    // Add to activity log - show all stderr, errors, warnings, and success messages
    if (stream === "stderr" || level === "error" || level === "warning" || level === "success") {
      useActivityLogStore.getState().addLog(
        serviceName,
        line,
        mappedLevel
      );
    }
  });
}
