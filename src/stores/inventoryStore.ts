import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import { invoke } from "@tauri-apps/api/core";
import type { InventoryResult, InventoryItem } from "@/types";

interface InventoryState {
  inventory: InventoryResult | null;
  isScanning: boolean;
  error: string | null;
  showInstalledOnly: boolean;
  searchQuery: string;
}

interface InventoryActions {
  scanInventory: () => Promise<void>;
  refreshItem: (id: string) => Promise<void>;
  setShowInstalledOnly: (value: boolean) => void;
  setSearchQuery: (query: string) => void;
  clearError: () => void;
}

type InventoryStore = InventoryState & InventoryActions;

export const useInventoryStore = create<InventoryStore>()(
  immer((set, get) => ({
    inventory: null,
    isScanning: false,
    error: null,
    showInstalledOnly: false,
    searchQuery: "",

    scanInventory: async () => {
      set((state) => {
        state.isScanning = true;
        state.error = null;
      });

      try {
        const result = await invoke<InventoryResult>("scan_inventory");
        set((state) => {
          state.inventory = result;
          state.isScanning = false;
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
          state.isScanning = false;
        });
      }
    },

    refreshItem: async (id: string) => {
      try {
        const item = await invoke<InventoryItem>("refresh_inventory_item", {
          id,
        });
        const inventory = get().inventory;
        if (!inventory) return;

        set((state) => {
          if (!state.inventory) return;

          // Find and update the item in the appropriate category
          const categories = [
            "runtimes",
            "webServers",
            "databases",
            "buildTools",
            "frameworks",
            "packageManagers",
            "devTools",
          ] as const;

          for (const cat of categories) {
            const index = state.inventory[cat].findIndex((i) => i.id === id);
            if (index !== -1) {
              state.inventory[cat][index] = item;
              break;
            }
          }
        });
      } catch (error) {
        console.error("Failed to refresh item:", error);
      }
    },

    setShowInstalledOnly: (value: boolean) => {
      set((state) => {
        state.showInstalledOnly = value;
      });
    },

    setSearchQuery: (query: string) => {
      set((state) => {
        state.searchQuery = query;
      });
    },

    clearError: () => {
      set((state) => {
        state.error = null;
      });
    },
  }))
);
