import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type { ProcessLog } from "@/types";

const MAX_LOGS_PER_PROJECT = 1000;

interface LogState {
  logs: Record<string, ProcessLog[]>;
}

interface LogActions {
  addLog: (log: ProcessLog) => void;
  clearLogs: (projectId: string) => void;
  clearAllLogs: () => void;
  getLogs: (projectId: string) => ProcessLog[];
}

type LogStore = LogState & LogActions;

export const useLogStore = create<LogStore>()(
  immer((set, get) => ({
    logs: {},

    addLog: (log: ProcessLog) => {
      set((state) => {
        if (!state.logs[log.projectId]) {
          state.logs[log.projectId] = [];
        }

        const projectLogs = state.logs[log.projectId];
        projectLogs.push({
          ...log,
          timestamp: log.timestamp || new Date().toISOString(),
        });

        // Keep only the last MAX_LOGS_PER_PROJECT logs
        if (projectLogs.length > MAX_LOGS_PER_PROJECT) {
          state.logs[log.projectId] = projectLogs.slice(-MAX_LOGS_PER_PROJECT);
        }
      });
    },

    clearLogs: (projectId: string) => {
      set((state) => {
        delete state.logs[projectId];
      });
    },

    clearAllLogs: () => {
      set((state) => {
        state.logs = {};
      });
    },

    getLogs: (projectId: string) => {
      return get().logs[projectId] || [];
    },
  }))
);
