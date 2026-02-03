import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  RefreshCw,
  Download,
  CheckCircle,
  AlertTriangle,
  ExternalLink,
  Loader2,
  Sparkles,
  Package,
} from "lucide-react";
import {
  UpdateCheckResult,
  UpdateInfo,
  UpdateStatus,
  formatBytes,
  formatRelativeDate,
} from "@/types/updater";

export function UpdateChecker() {
  const [status, setStatus] = useState<UpdateStatus>("idle");
  const [currentVersion, setCurrentVersion] = useState<string>("");
  const [updateResult, setUpdateResult] = useState<UpdateCheckResult | null>(
    null
  );
  const [downloadPath, setDownloadPath] = useState<string>("");
  const [error, setError] = useState<string | null>(null);

  const checkForUpdates = useCallback(async () => {
    setStatus("checking");
    setError(null);

    try {
      const version = await invoke<string>("get_current_version");
      setCurrentVersion(version);

      const result = await invoke<UpdateCheckResult>("check_for_updates");
      setUpdateResult(result);

      if (result.error) {
        setStatus("error");
        setError(result.error);
      } else if (result.updateAvailable) {
        setStatus("available");
      } else {
        setStatus("up-to-date");
      }
    } catch (err) {
      setStatus("error");
      setError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  const downloadUpdate = useCallback(async () => {
    if (!updateResult?.updateInfo) return;

    setStatus("downloading");
    setError(null);

    try {
      const path = await invoke<string>("download_update", {
        updateInfo: updateResult.updateInfo,
      });
      setDownloadPath(path);
      setStatus("ready");
    } catch (err) {
      setStatus("error");
      setError(err instanceof Error ? err.message : String(err));
    }
  }, [updateResult]);

  const installUpdate = useCallback(async () => {
    if (!downloadPath) return;

    try {
      await invoke("install_update", { filePath: downloadPath });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, [downloadPath]);

  const openReleasesPage = useCallback(async () => {
    try {
      const url = await invoke<string>("get_releases_url");
      window.open(url, "_blank");
    } catch (err) {
      console.error("Failed to get releases URL:", err);
    }
  }, []);

  return (
    <div className="space-y-4">
      {/* Current Version */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Package className="text-blue-400" size={20} />
          <div>
            <p className="text-white font-medium">Current Version</p>
            <p className="text-sm text-slate-400">
              {currentVersion || "Loading..."}
            </p>
          </div>
        </div>
        <button
          onClick={checkForUpdates}
          disabled={status === "checking" || status === "downloading"}
          className="flex items-center gap-2 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {status === "checking" ? (
            <Loader2 size={18} className="animate-spin" />
          ) : (
            <RefreshCw size={18} />
          )}
          Check for Updates
        </button>
      </div>

      {/* Status Messages */}
      {status === "up-to-date" && (
        <div className="p-4 bg-green-900/20 border border-green-800 rounded-lg flex items-center gap-3">
          <CheckCircle className="text-green-400" size={20} />
          <div>
            <p className="text-green-400 font-medium">You're up to date!</p>
            <p className="text-sm text-green-400/70">
              ClickDevPort {currentVersion} is the latest version.
            </p>
          </div>
        </div>
      )}

      {status === "error" && error && (
        <div className="p-4 bg-red-900/20 border border-red-800 rounded-lg flex items-start gap-3">
          <AlertTriangle className="text-red-400 shrink-0 mt-0.5" size={20} />
          <div>
            <p className="text-red-400 font-medium">Update check failed</p>
            <p className="text-sm text-red-400/70">{error}</p>
            <button
              onClick={openReleasesPage}
              className="mt-2 flex items-center gap-1 text-sm text-blue-400 hover:text-blue-300"
            >
              <ExternalLink size={14} />
              Check releases manually
            </button>
          </div>
        </div>
      )}

      {/* Update Available */}
      {status === "available" && updateResult?.updateInfo && (
        <UpdateAvailableCard
          updateInfo={updateResult.updateInfo}
          currentVersion={currentVersion}
          onDownload={downloadUpdate}
          onOpenReleases={openReleasesPage}
        />
      )}

      {/* Downloading */}
      {status === "downloading" && (
        <div className="p-4 bg-blue-900/20 border border-blue-800 rounded-lg flex items-center gap-3">
          <Loader2 className="text-blue-400 animate-spin" size={20} />
          <div>
            <p className="text-blue-400 font-medium">Downloading update...</p>
            <p className="text-sm text-blue-400/70">
              Please wait while the update is being downloaded.
            </p>
          </div>
        </div>
      )}

      {/* Ready to Install */}
      {status === "ready" && (
        <div className="p-4 bg-green-900/20 border border-green-800 rounded-lg">
          <div className="flex items-start gap-3">
            <CheckCircle className="text-green-400 shrink-0 mt-0.5" size={20} />
            <div className="flex-1">
              <p className="text-green-400 font-medium">Download complete!</p>
              <p className="text-sm text-green-400/70 mb-3">
                The update has been downloaded to: {downloadPath}
              </p>
              <div className="flex gap-2">
                <button
                  onClick={installUpdate}
                  className="flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors"
                >
                  <Sparkles size={18} />
                  Install Update
                </button>
                <button
                  onClick={() => setStatus("idle")}
                  className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors"
                >
                  Later
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Last Checked */}
      {updateResult?.checkedAt && (
        <p className="text-xs text-slate-500">
          Last checked: {formatRelativeDate(updateResult.checkedAt)}
        </p>
      )}
    </div>
  );
}

interface UpdateAvailableCardProps {
  updateInfo: UpdateInfo;
  currentVersion: string;
  onDownload: () => void;
  onOpenReleases: () => void;
}

function UpdateAvailableCard({
  updateInfo,
  currentVersion,
  onDownload,
  onOpenReleases,
}: UpdateAvailableCardProps) {
  const [showReleaseNotes, setShowReleaseNotes] = useState(false);

  return (
    <div className="p-4 bg-amber-900/20 border border-amber-800 rounded-lg">
      <div className="flex items-start gap-3">
        <Sparkles className="text-amber-400 shrink-0 mt-0.5" size={20} />
        <div className="flex-1">
          <div className="flex items-center justify-between mb-2">
            <p className="text-amber-400 font-medium">Update Available!</p>
            {updateInfo.isPrerelease && (
              <span className="px-2 py-0.5 text-xs bg-amber-800 text-amber-200 rounded">
                Pre-release
              </span>
            )}
          </div>

          <div className="flex items-center gap-4 text-sm mb-3">
            <span className="text-slate-400">
              {currentVersion}
              <span className="mx-2 text-slate-600">-&gt;</span>
              <span className="text-white font-medium">
                {updateInfo.version}
              </span>
            </span>
            {updateInfo.assetSize && (
              <span className="text-slate-500">
                {formatBytes(updateInfo.assetSize)}
              </span>
            )}
            <span className="text-slate-500">
              {formatRelativeDate(updateInfo.publishedAt)}
            </span>
          </div>

          {/* Release Notes Toggle */}
          {updateInfo.releaseNotes && (
            <div className="mb-3">
              <button
                onClick={() => setShowReleaseNotes(!showReleaseNotes)}
                className="text-sm text-blue-400 hover:text-blue-300"
              >
                {showReleaseNotes ? "Hide" : "Show"} release notes
              </button>
              {showReleaseNotes && (
                <div className="mt-2 p-3 bg-slate-800/50 rounded-lg text-sm text-slate-300 max-h-48 overflow-y-auto whitespace-pre-wrap">
                  {updateInfo.releaseNotes}
                </div>
              )}
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-2">
            <button
              onClick={onDownload}
              className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
            >
              <Download size={18} />
              Download Update
            </button>
            <button
              onClick={onOpenReleases}
              className="flex items-center gap-2 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors"
            >
              <ExternalLink size={18} />
              View on GitHub
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
