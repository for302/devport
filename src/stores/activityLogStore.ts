import { create } from "zustand";
import { immer } from "zustand/middleware/immer";

export type ActivityLogLevel = "info" | "success" | "warning" | "error";

export interface ActivityLogEntry {
  id: string;
  timestamp: Date;
  service: string;
  message: string;
  level: ActivityLogLevel;
}

interface ActivityLogState {
  logs: ActivityLogEntry[];
  maxLogs: number;
}

interface ActivityLogActions {
  addLog: (service: string, message: string, level?: ActivityLogLevel) => void;
  clearLogs: () => void;
}

type ActivityLogStore = ActivityLogState & ActivityLogActions;

let logIdCounter = 0;

export const useActivityLogStore = create<ActivityLogStore>()(
  immer((set) => ({
    logs: [],
    maxLogs: 100,

    addLog: (service: string, message: string, level: ActivityLogLevel = "info") => {
      set((state) => {
        const newLog: ActivityLogEntry = {
          id: `log-${++logIdCounter}`,
          timestamp: new Date(),
          service,
          message,
          level,
        };
        state.logs.push(newLog);
        // Keep only the last maxLogs entries
        if (state.logs.length > state.maxLogs) {
          state.logs = state.logs.slice(-state.maxLogs);
        }
      });
    },

    clearLogs: () => {
      set((state) => {
        state.logs = [];
      });
    },
  }))
);
