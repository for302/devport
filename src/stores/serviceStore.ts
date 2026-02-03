import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import { invoke } from "@tauri-apps/api/core";
import type { ServiceInfo, LogEntry, LogReadResult } from "@/types";

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

    clearError: () => {
      set((state) => {
        state.error = null;
      });
    },
  }))
);
