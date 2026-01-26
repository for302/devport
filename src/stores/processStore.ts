import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type { ProcessInfo, HealthStatus } from "@/types";
import * as tauriCommands from "@/services/tauriCommands";

interface ProcessState {
  processes: Record<string, ProcessInfo>;
  healthStatuses: Record<string, HealthStatus>;
  isLoading: Record<string, boolean>;
}

interface ProcessActions {
  startProject: (projectId: string) => Promise<ProcessInfo>;
  stopProject: (projectId: string) => Promise<void>;
  restartProject: (projectId: string) => Promise<ProcessInfo>;
  setProcessInfo: (projectId: string, info: ProcessInfo | null) => void;
  checkHealth: (projectId: string, url: string) => Promise<HealthStatus>;
  isProjectRunning: (projectId: string) => boolean;
  getProcessInfo: (projectId: string) => ProcessInfo | undefined;
}

type ProcessStore = ProcessState & ProcessActions;

export const useProcessStore = create<ProcessStore>()(
  immer((set, get) => ({
    processes: {},
    healthStatuses: {},
    isLoading: {},

    startProject: async (projectId: string) => {
      set((state) => {
        state.isLoading[projectId] = true;
      });

      try {
        const processInfo = await tauriCommands.startProject(projectId);
        set((state) => {
          state.processes[projectId] = processInfo;
          state.isLoading[projectId] = false;
        });
        return processInfo;
      } catch (error) {
        set((state) => {
          state.isLoading[projectId] = false;
        });
        throw error;
      }
    },

    stopProject: async (projectId: string) => {
      set((state) => {
        state.isLoading[projectId] = true;
      });

      try {
        await tauriCommands.stopProject(projectId);
        set((state) => {
          delete state.processes[projectId];
          state.isLoading[projectId] = false;
        });
      } catch (error) {
        set((state) => {
          state.isLoading[projectId] = false;
        });
        throw error;
      }
    },

    restartProject: async (projectId: string) => {
      set((state) => {
        state.isLoading[projectId] = true;
      });

      try {
        const processInfo = await tauriCommands.restartProject(projectId);
        set((state) => {
          state.processes[projectId] = processInfo;
          state.isLoading[projectId] = false;
        });
        return processInfo;
      } catch (error) {
        set((state) => {
          state.isLoading[projectId] = false;
        });
        throw error;
      }
    },

    setProcessInfo: (projectId: string, info: ProcessInfo | null) => {
      set((state) => {
        if (info) {
          state.processes[projectId] = info;
        } else {
          delete state.processes[projectId];
        }
      });
    },

    checkHealth: async (projectId: string, url: string) => {
      try {
        const status = await tauriCommands.checkHealth(projectId, url);
        set((state) => {
          state.healthStatuses[projectId] = status;
        });
        return status;
      } catch (error) {
        const errorStatus: HealthStatus = {
          projectId,
          isHealthy: false,
          statusCode: null,
          responseTimeMs: null,
          error: error instanceof Error ? error.message : "Unknown error",
          checkedAt: new Date().toISOString(),
        };
        set((state) => {
          state.healthStatuses[projectId] = errorStatus;
        });
        return errorStatus;
      }
    },

    isProjectRunning: (projectId: string) => {
      return !!get().processes[projectId];
    },

    getProcessInfo: (projectId: string) => {
      return get().processes[projectId];
    },
  }))
);
