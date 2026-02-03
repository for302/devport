import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  BundleManifest,
  CategoryGroup,
  InstallationState,
  InstallationSummary,
  InstallOptions,
  InstallProgress,
  InstalledComponent,
  InstallerDownloadProgress,
  PresetInfo,
} from "@/types";

interface InstallerState {
  // Data
  manifest: BundleManifest | null;
  presets: PresetInfo[];
  categoryGroups: CategoryGroup[];
  installedComponents: InstalledComponent[];
  installationState: InstallationState | null;
  summary: InstallationSummary | null;
  bundleFiles: string[];

  // UI state
  selectedPresetId: string | null;
  selectedComponentIds: string[];
  isLoading: boolean;
  error: string | null;

  // Progress tracking
  installProgress: InstallProgress | null;
  downloadProgress: InstallerDownloadProgress | null;

  // Event listener cleanup
  unlisteners: UnlistenFn[];
}

interface InstallerActions {
  // Data fetching
  fetchManifest: () => Promise<void>;
  fetchPresets: () => Promise<void>;
  fetchCategoryGroups: () => Promise<void>;
  fetchInstalledComponents: () => Promise<void>;
  fetchInstallationState: () => Promise<void>;
  fetchSummary: () => Promise<void>;
  fetchBundleFiles: () => Promise<void>;
  fetchAll: () => Promise<void>;

  // Selection
  selectPreset: (presetId: string) => Promise<void>;
  toggleComponent: (componentId: string) => Promise<void>;
  clearSelection: () => void;

  // Installation
  installSelected: () => Promise<InstalledComponent[]>;
  installComponent: (
    componentId: string,
    options?: InstallOptions
  ) => Promise<InstalledComponent>;
  uninstallComponent: (componentId: string) => Promise<void>;

  // Download
  downloadBundle: (componentId: string) => Promise<string>;
  hasBundle: (componentId: string) => Promise<boolean>;
  deleteBundleFile: (fileName: string) => Promise<void>;
  cleanupDownloads: () => Promise<number>;
  getBundleStorageSize: () => Promise<number>;

  // Utility
  calculateSelectionSize: (componentIds: string[]) => Promise<number>;
  getPresetComponents: (presetId: string) => Promise<string[]>;
  createDirectories: () => Promise<void>;
  isComponentInstalled: (componentId: string) => Promise<boolean>;

  // Event listeners
  setupEventListeners: () => Promise<void>;
  cleanupEventListeners: () => void;

  // State management
  setError: (error: string | null) => void;
  clearError: () => void;
}

type InstallerStore = InstallerState & InstallerActions;

const initialState: InstallerState = {
  manifest: null,
  presets: [],
  categoryGroups: [],
  installedComponents: [],
  installationState: null,
  summary: null,
  bundleFiles: [],
  selectedPresetId: null,
  selectedComponentIds: [],
  isLoading: false,
  error: null,
  installProgress: null,
  downloadProgress: null,
  unlisteners: [],
};

export const useInstallerStore = create<InstallerStore>()(
  immer((set, get) => ({
    ...initialState,

    // ========================================================================
    // Data Fetching
    // ========================================================================

    fetchManifest: async () => {
      try {
        const manifest = await invoke<BundleManifest>("get_bundle_manifest");
        set((state) => {
          state.manifest = manifest;
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
        });
      }
    },

    fetchPresets: async () => {
      try {
        const presets = await invoke<PresetInfo[]>("get_install_presets");
        set((state) => {
          state.presets = presets;
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
        });
      }
    },

    fetchCategoryGroups: async () => {
      try {
        const groups = await invoke<CategoryGroup[]>(
          "get_components_by_category"
        );
        set((state) => {
          state.categoryGroups = groups;
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
        });
      }
    },

    fetchInstalledComponents: async () => {
      try {
        const installed = await invoke<InstalledComponent[]>(
          "get_installed_components"
        );
        set((state) => {
          state.installedComponents = installed;
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
        });
      }
    },

    fetchInstallationState: async () => {
      try {
        const installState = await invoke<InstallationState>(
          "get_installation_state"
        );
        set((state) => {
          state.installationState = installState;
          state.selectedComponentIds = installState.selectedComponents;
          state.selectedPresetId = installState.selectedPreset;
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
        });
      }
    },

    fetchSummary: async () => {
      try {
        const summary = await invoke<InstallationSummary>(
          "get_installation_summary"
        );
        set((state) => {
          state.summary = summary;
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
        });
      }
    },

    fetchBundleFiles: async () => {
      try {
        const files = await invoke<string[]>("list_bundle_files");
        set((state) => {
          state.bundleFiles = files;
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
        });
      }
    },

    fetchAll: async () => {
      set((state) => {
        state.isLoading = true;
        state.error = null;
      });

      try {
        await Promise.all([
          get().fetchManifest(),
          get().fetchPresets(),
          get().fetchCategoryGroups(),
          get().fetchInstalledComponents(),
          get().fetchInstallationState(),
          get().fetchSummary(),
          get().fetchBundleFiles(),
        ]);
      } finally {
        set((state) => {
          state.isLoading = false;
        });
      }
    },

    // ========================================================================
    // Selection
    // ========================================================================

    selectPreset: async (presetId: string) => {
      set((state) => {
        state.isLoading = true;
      });

      try {
        const components = await invoke<string[]>("select_install_preset", {
          presetId,
        });

        // Exclude already-installed components (detected by system scan or DevPort)
        const { categoryGroups } = get();
        const installedIds = new Set<string>();
        for (const group of categoryGroups) {
          for (const comp of group.components) {
            if (comp.isInstalled) {
              installedIds.add(comp.id);
            }
          }
        }
        const filteredComponents = components.filter((id) => !installedIds.has(id));

        set((state) => {
          state.selectedPresetId = presetId;
          state.selectedComponentIds = filteredComponents;
          state.isLoading = false;
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
          state.isLoading = false;
        });
      }
    },

    toggleComponent: async (componentId: string) => {
      try {
        const isSelected = await invoke<boolean>("toggle_component_selection", {
          componentId,
        });
        set((state) => {
          if (isSelected) {
            if (!state.selectedComponentIds.includes(componentId)) {
              state.selectedComponentIds.push(componentId);
            }
          } else {
            state.selectedComponentIds = state.selectedComponentIds.filter(
              (id) => id !== componentId
            );
          }
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
        });
      }
    },

    clearSelection: () => {
      set((state) => {
        state.selectedPresetId = null;
        state.selectedComponentIds = [];
      });
    },

    // ========================================================================
    // Installation
    // ========================================================================

    installSelected: async () => {
      set((state) => {
        state.isLoading = true;
        state.error = null;
      });

      try {
        const installed = await invoke<InstalledComponent[]>(
          "install_selected_components"
        );
        set((state) => {
          state.installedComponents = [
            ...state.installedComponents.filter(
              (c) => !installed.some((i) => i.id === c.id)
            ),
            ...installed,
          ];
          state.isLoading = false;
        });

        // Refresh data
        await get().fetchCategoryGroups();
        await get().fetchSummary();

        return installed;
      } catch (error) {
        set((state) => {
          state.error = String(error);
          state.isLoading = false;
        });
        throw error;
      }
    },

    installComponent: async (componentId: string, options?: InstallOptions) => {
      set((state) => {
        state.isLoading = true;
        state.error = null;
      });

      try {
        const installed = await invoke<InstalledComponent>("install_component", {
          componentId,
          options: options || null,
        });

        set((state) => {
          state.installedComponents = [
            ...state.installedComponents.filter((c) => c.id !== componentId),
            installed,
          ];
          state.isLoading = false;
        });

        // Refresh data
        await get().fetchCategoryGroups();
        await get().fetchSummary();

        return installed;
      } catch (error) {
        set((state) => {
          state.error = String(error);
          state.isLoading = false;
        });
        throw error;
      }
    },

    uninstallComponent: async (componentId: string) => {
      set((state) => {
        state.isLoading = true;
        state.error = null;
      });

      try {
        await invoke("uninstall_component", { componentId });

        set((state) => {
          state.installedComponents = state.installedComponents.filter(
            (c) => c.id !== componentId
          );
          state.isLoading = false;
        });

        // Refresh data
        await get().fetchCategoryGroups();
        await get().fetchSummary();
      } catch (error) {
        set((state) => {
          state.error = String(error);
          state.isLoading = false;
        });
        throw error;
      }
    },

    // ========================================================================
    // Download
    // ========================================================================

    downloadBundle: async (componentId: string) => {
      try {
        const path = await invoke<string>("download_component_bundle", {
          componentId,
        });
        await get().fetchBundleFiles();
        return path;
      } catch (error) {
        set((state) => {
          state.error = String(error);
        });
        throw error;
      }
    },

    hasBundle: async (componentId: string) => {
      return invoke<boolean>("has_component_bundle", { componentId });
    },

    deleteBundleFile: async (fileName: string) => {
      try {
        await invoke("delete_bundle_file", { fileName });
        set((state) => {
          state.bundleFiles = state.bundleFiles.filter((f) => f !== fileName);
        });
      } catch (error) {
        set((state) => {
          state.error = String(error);
        });
        throw error;
      }
    },

    cleanupDownloads: async () => {
      const cleaned = await invoke<number>("cleanup_incomplete_downloads");
      return cleaned;
    },

    getBundleStorageSize: async () => {
      return invoke<number>("get_bundle_storage_size");
    },

    // ========================================================================
    // Utility
    // ========================================================================

    calculateSelectionSize: async (componentIds: string[]) => {
      return invoke<number>("calculate_selection_size", { componentIds });
    },

    getPresetComponents: async (presetId: string) => {
      return invoke<string[]>("get_preset_components", { presetId });
    },

    createDirectories: async () => {
      await invoke("create_devport_directories");
    },

    isComponentInstalled: async (componentId: string) => {
      return invoke<boolean>("is_component_installed", { componentId });
    },

    // ========================================================================
    // Event Listeners
    // ========================================================================

    setupEventListeners: async () => {
      const unlisteners: UnlistenFn[] = [];

      // Listen for install progress
      const unlistenInstall = await listen<InstallProgress>(
        "install-progress",
        (event) => {
          set((state) => {
            state.installProgress = event.payload;
          });
        }
      );
      unlisteners.push(unlistenInstall);

      // Listen for download progress
      const unlistenDownload = await listen<InstallerDownloadProgress>(
        "download-progress",
        (event) => {
          set((state) => {
            state.downloadProgress = event.payload;
          });
        }
      );
      unlisteners.push(unlistenDownload);

      set((state) => {
        state.unlisteners = unlisteners;
      });
    },

    cleanupEventListeners: () => {
      const { unlisteners } = get();
      unlisteners.forEach((unlisten) => unlisten());
      set((state) => {
        state.unlisteners = [];
      });
    },

    // ========================================================================
    // State Management
    // ========================================================================

    setError: (error: string | null) => {
      set((state) => {
        state.error = error;
      });
    },

    clearError: () => {
      set((state) => {
        state.error = null;
      });
    },
  }))
);

// Selector hooks for optimized re-renders
export const useInstallerPresets = () =>
  useInstallerStore((state) => state.presets);
export const useInstallerCategories = () =>
  useInstallerStore((state) => state.categoryGroups);
export const useInstallerSelected = () =>
  useInstallerStore((state) => ({
    presetId: state.selectedPresetId,
    componentIds: state.selectedComponentIds,
  }));
export const useInstallerProgress = () =>
  useInstallerStore((state) => ({
    install: state.installProgress,
    download: state.downloadProgress,
    isLoading: state.isLoading,
  }));
export const useInstallerSummary = () =>
  useInstallerStore((state) => state.summary);
