import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import { persist } from "zustand/middleware";
import type {
  AppSettings,
  ThemeMode,
  LanguageCode,
  LogFileSizeOption,
  AppInfo,
} from "@/types/settings";
import { DEFAULT_SETTINGS } from "@/types/settings";

interface SettingsState {
  settings: AppSettings;
  appInfo: AppInfo;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
  hasUnsavedChanges: boolean;
}

interface SettingsActions {
  // General Settings
  setTheme: (theme: ThemeMode) => void;
  setLanguage: (language: LanguageCode) => void;
  setCloseToTray: (closeToTray: boolean) => void;
  setShowNotifications: (showNotifications: boolean) => void;

  // Auto Start Settings
  setStartWithWindows: (startWithWindows: boolean) => void;
  toggleAutoStartService: (serviceId: string) => void;
  setStartMinimized: (startMinimized: boolean) => void;

  // Path Settings
  setAddToSystemPath: (addToSystemPath: boolean) => void;
  toggleRuntimePath: (pathName: string) => void;

  // Log Settings
  setMaxLogFileSize: (size: LogFileSizeOption) => void;
  setMaxFilesToKeep: (count: number) => void;
  setRetentionDays: (days: number) => void;
  setAutoCleanup: (autoCleanup: boolean) => void;

  // Database Settings
  setMariadbRootPassword: (password: string) => void;
  setAutoBackup: (autoBackup: boolean) => void;
  setBackupSchedule: (schedule: "daily" | "weekly" | "manual") => void;

  // Actions
  saveSettings: () => Promise<void>;
  loadSettings: () => Promise<void>;
  resetToDefaults: () => void;
  cleanupOldLogs: () => Promise<void>;
  changeMariadbPassword: (newPassword: string) => Promise<void>;
  clearError: () => void;
}

type SettingsStore = SettingsState & SettingsActions;

export const useSettingsStore = create<SettingsStore>()(
  persist(
    immer((set, _get) => ({
      settings: DEFAULT_SETTINGS,
      appInfo: {
        version: "0.1.0",
        tauriVersion: "2.x",
        buildType: "development",
        buildDate: new Date().toISOString().split("T")[0],
      },
      isLoading: false,
      isSaving: false,
      error: null,
      hasUnsavedChanges: false,

      // General Settings
      setTheme: (theme: ThemeMode) => {
        set((state) => {
          state.settings.general.theme = theme;
          state.hasUnsavedChanges = true;
        });
      },

      setLanguage: (language: LanguageCode) => {
        set((state) => {
          state.settings.general.language = language;
          state.hasUnsavedChanges = true;
        });
      },

      setCloseToTray: (closeToTray: boolean) => {
        set((state) => {
          state.settings.general.closeToTray = closeToTray;
          state.hasUnsavedChanges = true;
        });
      },

      setShowNotifications: (showNotifications: boolean) => {
        set((state) => {
          state.settings.general.showNotifications = showNotifications;
          state.hasUnsavedChanges = true;
        });
      },

      // Auto Start Settings
      setStartWithWindows: (startWithWindows: boolean) => {
        set((state) => {
          state.settings.autoStart.startWithWindows = startWithWindows;
          state.hasUnsavedChanges = true;
        });
      },

      toggleAutoStartService: (serviceId: string) => {
        set((state) => {
          const services = state.settings.autoStart.autoStartServices;
          const index = services.indexOf(serviceId);
          if (index === -1) {
            services.push(serviceId);
          } else {
            services.splice(index, 1);
          }
          state.hasUnsavedChanges = true;
        });
      },

      setStartMinimized: (startMinimized: boolean) => {
        set((state) => {
          state.settings.autoStart.startMinimized = startMinimized;
          state.hasUnsavedChanges = true;
        });
      },

      // Path Settings
      setAddToSystemPath: (addToSystemPath: boolean) => {
        set((state) => {
          state.settings.paths.addToSystemPath = addToSystemPath;
          state.hasUnsavedChanges = true;
        });
      },

      toggleRuntimePath: (pathName: string) => {
        set((state) => {
          const path = state.settings.paths.runtimePaths.find(
            (p) => p.name === pathName
          );
          if (path) {
            path.enabled = !path.enabled;
          }
          state.hasUnsavedChanges = true;
        });
      },

      // Log Settings
      setMaxLogFileSize: (size: LogFileSizeOption) => {
        set((state) => {
          state.settings.logs.maxLogFileSize = size;
          state.hasUnsavedChanges = true;
        });
      },

      setMaxFilesToKeep: (count: number) => {
        set((state) => {
          state.settings.logs.maxFilesToKeep = Math.max(1, Math.min(100, count));
          state.hasUnsavedChanges = true;
        });
      },

      setRetentionDays: (days: number) => {
        set((state) => {
          state.settings.logs.retentionDays = Math.max(1, Math.min(365, days));
          state.hasUnsavedChanges = true;
        });
      },

      setAutoCleanup: (autoCleanup: boolean) => {
        set((state) => {
          state.settings.logs.autoCleanup = autoCleanup;
          state.hasUnsavedChanges = true;
        });
      },

      // Database Settings
      setMariadbRootPassword: (password: string) => {
        set((state) => {
          state.settings.database.mariadbRootPassword = password;
          state.hasUnsavedChanges = true;
        });
      },

      setAutoBackup: (autoBackup: boolean) => {
        set((state) => {
          state.settings.database.autoBackup = autoBackup;
          state.hasUnsavedChanges = true;
        });
      },

      setBackupSchedule: (schedule: "daily" | "weekly" | "manual") => {
        set((state) => {
          state.settings.database.backupSchedule = schedule;
          state.hasUnsavedChanges = true;
        });
      },

      // Actions
      saveSettings: async () => {
        set((state) => {
          state.isSaving = true;
          state.error = null;
        });

        try {
          // TODO: Call Tauri backend to persist settings
          // await invoke("save_settings", { settings: get().settings });

          // Simulate save delay
          await new Promise((resolve) => setTimeout(resolve, 300));

          set((state) => {
            state.isSaving = false;
            state.hasUnsavedChanges = false;
          });
        } catch (error) {
          set((state) => {
            state.error = String(error);
            state.isSaving = false;
          });
        }
      },

      loadSettings: async () => {
        set((state) => {
          state.isLoading = true;
          state.error = null;
        });

        try {
          // TODO: Call Tauri backend to load settings
          // const settings = await invoke<AppSettings>("load_settings");
          // set((state) => {
          //   state.settings = settings;
          // });

          set((state) => {
            state.isLoading = false;
          });
        } catch (error) {
          set((state) => {
            state.error = String(error);
            state.isLoading = false;
          });
        }
      },

      resetToDefaults: () => {
        set((state) => {
          state.settings = DEFAULT_SETTINGS;
          state.hasUnsavedChanges = true;
        });
      },

      cleanupOldLogs: async () => {
        set((state) => {
          state.isLoading = true;
          state.error = null;
        });

        try {
          // TODO: Call Tauri backend to cleanup logs
          // await invoke("cleanup_logs", {
          //   maxSize: get().settings.logs.maxLogFileSize,
          //   maxFiles: get().settings.logs.maxFilesToKeep,
          //   retentionDays: get().settings.logs.retentionDays,
          // });

          // Simulate cleanup delay
          await new Promise((resolve) => setTimeout(resolve, 500));

          set((state) => {
            state.isLoading = false;
          });
        } catch (error) {
          set((state) => {
            state.error = String(error);
            state.isLoading = false;
          });
        }
      },

      changeMariadbPassword: async (newPassword: string) => {
        set((state) => {
          state.isLoading = true;
          state.error = null;
        });

        try {
          // TODO: Call Tauri backend to change MariaDB password
          // await invoke("change_mariadb_password", { newPassword });

          set((state) => {
            state.settings.database.mariadbRootPassword = newPassword;
            state.isLoading = false;
          });
        } catch (error) {
          set((state) => {
            state.error = String(error);
            state.isLoading = false;
          });
        }
      },

      clearError: () => {
        set((state) => {
          state.error = null;
        });
      },
    })),
    {
      name: "devport-settings",
      partialize: (state) => ({ settings: state.settings }),
    }
  )
);
