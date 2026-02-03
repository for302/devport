import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  UpdateCheckResult,
  DownloadProgress,
  UpdateStatus,
} from "@/types";

interface UpdaterState {
  status: UpdateStatus;
  checkResult: UpdateCheckResult | null;
  downloadProgress: DownloadProgress | null;
  downloadedPath: string | null;
  error: string | null;
  isDismissed: boolean;
}

interface UpdaterActions {
  checkForUpdates: () => Promise<void>;
  downloadUpdate: () => Promise<void>;
  installUpdate: () => Promise<void>;
  dismissUpdate: () => void;
  reset: () => void;
  initEventListeners: () => Promise<UnlistenFn>;
}

type UpdaterStore = UpdaterState & UpdaterActions;

const initialState: UpdaterState = {
  status: "idle",
  checkResult: null,
  downloadProgress: null,
  downloadedPath: null,
  error: null,
  isDismissed: false,
};

export const useUpdaterStore = create<UpdaterStore>()(
  immer((set, get) => ({
    ...initialState,

    checkForUpdates: async () => {
      set((state) => {
        state.status = "checking";
        state.error = null;
        state.isDismissed = false;
      });

      try {
        const result = await invoke<UpdateCheckResult>("check_for_updates");
        set((state) => {
          state.checkResult = result;
          if (result.error) {
            state.status = "error";
            state.error = result.error;
          } else if (result.updateAvailable) {
            state.status = "available";
          } else {
            state.status = "up-to-date";
          }
        });
      } catch (error) {
        set((state) => {
          state.status = "error";
          state.error = error instanceof Error ? error.message : String(error);
        });
      }
    },

    downloadUpdate: async () => {
      const { checkResult } = get();
      if (!checkResult?.updateInfo) {
        return;
      }

      set((state) => {
        state.status = "downloading";
        state.downloadProgress = null;
        state.error = null;
      });

      try {
        const filePath = await invoke<string>("download_update_with_progress", {
          updateInfo: checkResult.updateInfo,
        });
        set((state) => {
          state.status = "ready";
          state.downloadedPath = filePath;
        });
      } catch (error) {
        set((state) => {
          state.status = "error";
          state.error = error instanceof Error ? error.message : String(error);
        });
      }
    },

    installUpdate: async () => {
      const { downloadedPath } = get();
      if (!downloadedPath) {
        return;
      }

      try {
        await invoke("install_update_and_quit", { filePath: downloadedPath });
      } catch (error) {
        set((state) => {
          state.status = "error";
          state.error = error instanceof Error ? error.message : String(error);
        });
      }
    },

    dismissUpdate: () => {
      set((state) => {
        state.isDismissed = true;
      });
    },

    reset: () => {
      set(() => ({ ...initialState }));
    },

    initEventListeners: async () => {
      const unlisten = await listen<DownloadProgress>(
        "download-progress",
        (event) => {
          set((state) => {
            state.downloadProgress = event.payload;
          });
        }
      );
      return unlisten;
    },
  }))
);

// Selectors
export const useUpdateStatus = () => useUpdaterStore((state) => state.status);
export const useUpdateCheckResult = () =>
  useUpdaterStore((state) => state.checkResult);
export const useDownloadProgress = () =>
  useUpdaterStore((state) => state.downloadProgress);
export const useIsUpdateAvailable = () =>
  useUpdaterStore(
    (state) => state.status === "available" && !state.isDismissed
  );
export const useIsUpdateReady = () =>
  useUpdaterStore((state) => state.status === "ready");
