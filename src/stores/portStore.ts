import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type { PortInfo } from "@/types";
import * as tauriCommands from "@/services/tauriCommands";

interface PortState {
  ports: PortInfo[];
  isScanning: boolean;
  lastScanned: string | null;
  error: string | null;
}

interface PortActions {
  scanPorts: () => Promise<void>;
  checkPortAvailable: (port: number) => Promise<boolean>;
  getPortByNumber: (port: number) => PortInfo | undefined;
  getPortsByProject: (projectId: string) => PortInfo[];
}

type PortStore = PortState & PortActions;

export const usePortStore = create<PortStore>()(
  immer((set, get) => ({
    ports: [],
    isScanning: false,
    lastScanned: null,
    error: null,

    scanPorts: async () => {
      set((state) => {
        state.isScanning = true;
        state.error = null;
      });

      try {
        const ports = await tauriCommands.scanPorts();
        set((state) => {
          state.ports = ports;
          state.isScanning = false;
          state.lastScanned = new Date().toISOString();
        });
      } catch (error) {
        set((state) => {
          state.error = error instanceof Error ? error.message : "Failed to scan ports";
          state.isScanning = false;
        });
      }
    },

    checkPortAvailable: async (port: number) => {
      try {
        return await tauriCommands.checkPortAvailable(port);
      } catch {
        return false;
      }
    },

    getPortByNumber: (port: number) => {
      return get().ports.find((p) => p.port === port);
    },

    getPortsByProject: (projectId: string) => {
      return get().ports.filter((p) => p.projectId === projectId);
    },
  }))
);
