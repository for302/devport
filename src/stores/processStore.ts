import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type { ProcessInfo, HealthStatus } from "@/types";
import * as tauriCommands from "@/services/tauriCommands";
import { useProjectStore } from "./projectStore";

interface ProcessState {
  processes: Record<string, ProcessInfo>;
  healthStatuses: Record<string, HealthStatus>;
  isLoading: Record<string, boolean>;
  buildStatuses: Record<string, string>;
  buildStartTimes: Record<string, number>;
}

interface ProcessActions {
  startProject: (projectId: string) => Promise<ProcessInfo>;
  stopProject: (projectId: string) => Promise<void>;
  restartProject: (projectId: string) => Promise<ProcessInfo>;
  setProcessInfo: (projectId: string, info: ProcessInfo | null) => void;
  setBuildStatus: (projectId: string, status: string) => void;
  clearBuildLoading: (projectId: string) => void;
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
    buildStatuses: {},
    buildStartTimes: {},

    startProject: async (projectId: string) => {
      set((state) => {
        state.isLoading[projectId] = true;
        state.buildStartTimes[projectId] = Date.now();
        delete state.buildStatuses[projectId];
      });

      try {
        const processInfo = await tauriCommands.startProject(projectId);
        set((state) => {
          state.processes[projectId] = processInfo;
          // isLoading stays true — cleared by "launched" event or safety timeout
        });

        // Safety timeout: 5 min force-clear loading
        setTimeout(() => {
          if (useProcessStore.getState().isLoading[projectId]) {
            useProcessStore.getState().clearBuildLoading(projectId);
          }
        }, 5 * 60 * 1000);

        // Refresh project list to reflect port sync (e.g., Tauri devUrl port)
        useProjectStore.getState().fetchProjects();

        return processInfo;
      } catch (error) {
        set((state) => {
          state.isLoading[projectId] = false;
          delete state.buildStartTimes[projectId];
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
          delete state.buildStatuses[projectId];
          delete state.buildStartTimes[projectId];
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
        state.buildStartTimes[projectId] = Date.now();
        delete state.buildStatuses[projectId];
      });

      try {
        const processInfo = await tauriCommands.restartProject(projectId);
        set((state) => {
          state.processes[projectId] = processInfo;
          // isLoading stays true — cleared by "launched" event or safety timeout
        });

        // Safety timeout: 5 min force-clear loading
        setTimeout(() => {
          if (useProcessStore.getState().isLoading[projectId]) {
            useProcessStore.getState().clearBuildLoading(projectId);
          }
        }, 5 * 60 * 1000);

        return processInfo;
      } catch (error) {
        set((state) => {
          state.isLoading[projectId] = false;
          delete state.buildStartTimes[projectId];
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

    setBuildStatus: (projectId: string, status: string) => {
      set((state) => {
        state.buildStatuses[projectId] = status;
      });
    },

    clearBuildLoading: (projectId: string) => {
      set((state) => {
        state.isLoading[projectId] = false;
        delete state.buildStartTimes[projectId];
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
