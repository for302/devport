import { useEffect } from "react";
import {
  Download,
  X,
  Loader2,
  CheckCircle2,
  AlertCircle,
  RefreshCw,
  ExternalLink,
} from "lucide-react";
import { useUiStore, useUpdaterStore } from "@/stores";
import { formatBytes, formatRelativeDate } from "@/types/updater";

export function UpdateModal() {
  const closeModal = useUiStore((state) => state.closeModal);
  const {
    status,
    checkResult,
    downloadProgress,
    downloadedPath,
    error,
    downloadUpdate,
    installUpdate,
    dismissUpdate,
    reset,
    initEventListeners,
  } = useUpdaterStore();

  // Set up event listeners for download progress
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    initEventListeners().then((unlistenFn) => {
      unlisten = unlistenFn;
    });

    return () => {
      unlisten?.();
    };
  }, [initEventListeners]);

  const handleClose = () => {
    if (status === "downloading") {
      return; // Don't allow closing while downloading
    }
    dismissUpdate();
    closeModal();
  };

  const handleDownload = async () => {
    await downloadUpdate();
  };

  const handleInstall = async () => {
    await installUpdate();
  };

  const handleRetry = () => {
    reset();
    closeModal();
  };

  const updateInfo = checkResult?.updateInfo;

  // Render different content based on status
  const renderContent = () => {
    switch (status) {
      case "available":
        return (
          <>
            {/* Header */}
            <div className="flex items-center justify-between px-6 py-4 border-b border-slate-700">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-blue-500/20 rounded-lg">
                  <Download size={24} className="text-blue-400" />
                </div>
                <h2 className="text-lg font-semibold text-white">
                  Update Available
                </h2>
              </div>
              <button
                onClick={handleClose}
                className="p-1 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors"
              >
                <X size={20} />
              </button>
            </div>

            {/* Content */}
            <div className="p-6 space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm text-slate-400">New version</p>
                  <p className="text-xl font-semibold text-white">
                    v{updateInfo?.version}
                  </p>
                </div>
                <div className="text-right">
                  <p className="text-sm text-slate-400">Current version</p>
                  <p className="text-lg text-slate-300">
                    v{checkResult?.currentVersion}
                  </p>
                </div>
              </div>

              {updateInfo?.publishedAt && (
                <p className="text-sm text-slate-400">
                  Released {formatRelativeDate(updateInfo.publishedAt)}
                </p>
              )}

              {updateInfo?.assetSize && (
                <p className="text-sm text-slate-400">
                  Download size: {formatBytes(updateInfo.assetSize)}
                </p>
              )}

              {updateInfo?.releaseNotes && (
                <div className="mt-4">
                  <p className="text-sm font-medium text-slate-300 mb-2">
                    Release Notes
                  </p>
                  <div className="bg-slate-900 rounded-lg p-4 max-h-48 overflow-y-auto">
                    <pre className="text-sm text-slate-400 whitespace-pre-wrap font-sans">
                      {updateInfo.releaseNotes}
                    </pre>
                  </div>
                </div>
              )}

              {updateInfo?.isPrerelease && (
                <div className="p-3 bg-yellow-500/20 border border-yellow-500/50 rounded-lg text-yellow-400 text-sm">
                  This is a pre-release version. It may contain bugs or
                  incomplete features.
                </div>
              )}
            </div>

            {/* Actions */}
            <div className="flex justify-end gap-3 px-6 py-4 border-t border-slate-700">
              <button
                onClick={handleClose}
                className="px-4 py-2 rounded-lg text-slate-300 hover:bg-slate-700 transition-colors"
              >
                Later
              </button>
              <button
                onClick={handleDownload}
                className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg text-white font-medium transition-colors flex items-center gap-2"
              >
                <Download size={18} />
                Update Now
              </button>
            </div>
          </>
        );

      case "downloading":
        const percentage = downloadProgress?.percentage ?? 0;
        const downloaded = downloadProgress?.downloadedBytes ?? 0;
        const total = downloadProgress?.totalBytes ?? 0;

        return (
          <>
            {/* Header */}
            <div className="flex items-center justify-between px-6 py-4 border-b border-slate-700">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-blue-500/20 rounded-lg">
                  <Loader2 size={24} className="text-blue-400 animate-spin" />
                </div>
                <h2 className="text-lg font-semibold text-white">
                  Downloading Update
                </h2>
              </div>
            </div>

            {/* Content */}
            <div className="p-6 space-y-4">
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="text-slate-400">
                    Downloading v{updateInfo?.version}...
                  </span>
                  <span className="text-slate-300">
                    {percentage.toFixed(1)}%
                  </span>
                </div>

                {/* Progress bar */}
                <div className="h-3 bg-slate-700 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-blue-500 transition-all duration-300"
                    style={{ width: `${percentage}%` }}
                  />
                </div>

                <p className="text-sm text-slate-400">
                  {formatBytes(downloaded)} / {formatBytes(total)}
                </p>
              </div>

              <p className="text-sm text-slate-400">
                Please do not close the application during download.
              </p>
            </div>
          </>
        );

      case "ready":
        return (
          <>
            {/* Header */}
            <div className="flex items-center justify-between px-6 py-4 border-b border-slate-700">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-green-500/20 rounded-lg">
                  <CheckCircle2 size={24} className="text-green-400" />
                </div>
                <h2 className="text-lg font-semibold text-white">
                  Ready to Install
                </h2>
              </div>
              <button
                onClick={handleClose}
                className="p-1 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors"
              >
                <X size={20} />
              </button>
            </div>

            {/* Content */}
            <div className="p-6 space-y-4">
              <p className="text-slate-300">
                Version <span className="font-semibold">v{updateInfo?.version}</span> has
                been downloaded successfully.
              </p>

              <p className="text-sm text-slate-400">
                The application will close and the installer will start. Follow
                the installation wizard to complete the update.
              </p>

              {downloadedPath && (
                <p className="text-xs text-slate-500 break-all">
                  File: {downloadedPath}
                </p>
              )}
            </div>

            {/* Actions */}
            <div className="flex justify-end gap-3 px-6 py-4 border-t border-slate-700">
              <button
                onClick={handleClose}
                className="px-4 py-2 rounded-lg text-slate-300 hover:bg-slate-700 transition-colors"
              >
                Install Later
              </button>
              <button
                onClick={handleInstall}
                className="px-4 py-2 bg-green-600 hover:bg-green-700 rounded-lg text-white font-medium transition-colors flex items-center gap-2"
              >
                <ExternalLink size={18} />
                Install Now
              </button>
            </div>
          </>
        );

      case "error":
        return (
          <>
            {/* Header */}
            <div className="flex items-center justify-between px-6 py-4 border-b border-slate-700">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-red-500/20 rounded-lg">
                  <AlertCircle size={24} className="text-red-400" />
                </div>
                <h2 className="text-lg font-semibold text-white">
                  Update Error
                </h2>
              </div>
              <button
                onClick={handleClose}
                className="p-1 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors"
              >
                <X size={20} />
              </button>
            </div>

            {/* Content */}
            <div className="p-6 space-y-4">
              <div className="p-3 bg-red-500/20 border border-red-500/50 rounded-lg text-red-400 text-sm">
                {error || "An unknown error occurred while checking for updates."}
              </div>

              <p className="text-sm text-slate-400">
                Please check your internet connection and try again. If the
                problem persists, you can download the update manually from
                GitHub.
              </p>
            </div>

            {/* Actions */}
            <div className="flex justify-end gap-3 px-6 py-4 border-t border-slate-700">
              <button
                onClick={handleClose}
                className="px-4 py-2 rounded-lg text-slate-300 hover:bg-slate-700 transition-colors"
              >
                Close
              </button>
              <button
                onClick={handleRetry}
                className="px-4 py-2 bg-slate-600 hover:bg-slate-500 rounded-lg text-white font-medium transition-colors flex items-center gap-2"
              >
                <RefreshCw size={18} />
                Retry
              </button>
            </div>
          </>
        );

      default:
        return null;
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-slate-800 rounded-lg w-full max-w-md mx-4">
        {renderContent()}
      </div>
    </div>
  );
}
