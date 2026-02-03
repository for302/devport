import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import { invoke } from "@tauri-apps/api/core";
import type { ApachePortEntry, ApacheVHostRequest } from "@/types";

interface ApacheConfigState {
  ports: ApachePortEntry[];
  isLoading: boolean;
  error: string | null;
  needsRestart: boolean;
  apacheBasePath: string | null;
}

interface ApacheConfigActions {
  fetchPorts: () => Promise<void>;
  createVHost: (request: ApacheVHostRequest) => Promise<ApachePortEntry>;
  updateVHost: (id: string, request: ApacheVHostRequest) => Promise<void>;
  deleteVHost: (id: string) => Promise<void>;
  addListenPort: (port: number) => Promise<void>;
  removeListenPort: (port: number) => Promise<void>;
  checkDocumentRoot: (path: string) => Promise<boolean>;
  createDocumentRoot: (path: string) => Promise<void>;
  fetchApacheBasePath: () => Promise<void>;
  setNeedsRestart: (value: boolean) => void;
  clearError: () => void;
}

type ApacheConfigStore = ApacheConfigState & ApacheConfigActions;

export const useApacheConfigStore = create<ApacheConfigStore>()(
  immer((set, get) => ({
    ports: [],
    isLoading: false,
    error: null,
    needsRestart: false,
    apacheBasePath: null,

    fetchPorts: async () => {
      set((state) => {
        state.isLoading = true;
        state.error = null;
      });

      try {
        const ports = await invoke<ApachePortEntry[]>("get_apache_ports");
        set((state) => {
          state.ports = ports;
          state.isLoading = false;
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
          state.isLoading = false;
        });
      }
    },

    createVHost: async (request: ApacheVHostRequest) => {
      set((state) => {
        state.isLoading = true;
        state.error = null;
      });

      try {
        const entry = await invoke<ApachePortEntry>("create_apache_vhost", { request });
        set((state) => {
          state.ports.push(entry);
          state.ports.sort((a, b) => a.port - b.port);
          state.isLoading = false;
          state.needsRestart = true;
        });
        return entry;
      } catch (error) {
        set((state) => {
          state.error = String(error);
          state.isLoading = false;
        });
        throw error;
      }
    },

    updateVHost: async (id: string, request: ApacheVHostRequest) => {
      set((state) => {
        state.isLoading = true;
        state.error = null;
      });

      try {
        const entry = await invoke<ApachePortEntry>("update_apache_vhost", { id, request });
        set((state) => {
          const index = state.ports.findIndex((p) => p.id === id);
          if (index !== -1) {
            state.ports[index] = entry;
          }
          state.ports.sort((a, b) => a.port - b.port);
          state.isLoading = false;
          state.needsRestart = true;
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
          state.isLoading = false;
        });
        throw error;
      }
    },

    deleteVHost: async (id: string) => {
      set((state) => {
        state.isLoading = true;
        state.error = null;
      });

      try {
        await invoke("delete_apache_vhost", { id });
        set((state) => {
          state.ports = state.ports.filter((p) => p.id !== id);
          state.isLoading = false;
          state.needsRestart = true;
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
          state.isLoading = false;
        });
        throw error;
      }
    },

    addListenPort: async (port: number) => {
      try {
        await invoke("add_listen_port", { port });
        set((state) => {
          state.needsRestart = true;
        });
        await get().fetchPorts();
      } catch (error) {
        set((state) => {
          state.error = String(error);
        });
        throw error;
      }
    },

    removeListenPort: async (port: number) => {
      try {
        await invoke("remove_listen_port", { port });
        set((state) => {
          state.needsRestart = true;
        });
        await get().fetchPorts();
      } catch (error) {
        set((state) => {
          state.error = String(error);
        });
        throw error;
      }
    },

    checkDocumentRoot: async (path: string) => {
      try {
        return await invoke<boolean>("check_document_root", { path });
      } catch (error) {
        console.error("Failed to check document root:", error);
        return false;
      }
    },

    createDocumentRoot: async (path: string) => {
      try {
        await invoke("create_document_root", { path });
      } catch (error) {
        set((state) => {
          state.error = String(error);
        });
        throw error;
      }
    },

    fetchApacheBasePath: async () => {
      try {
        const basePath = await invoke<string>("get_apache_base_path");
        set((state) => {
          state.apacheBasePath = basePath;
        });
      } catch (error) {
        set((state) => {
          state.apacheBasePath = null;
        });
      }
    },

    setNeedsRestart: (value: boolean) => {
      set((state) => {
        state.needsRestart = value;
      });
    },

    clearError: () => {
      set((state) => {
        state.error = null;
      });
    },
  }))
);
