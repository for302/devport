import { useState, useEffect } from "react";
import {
  Save,
  FolderOpen,
  Sun,
  Moon,
  Monitor,
  Power,
  Database,
  FileText,
  Info,
  AlertTriangle,
  Trash2,
  Key,
  ExternalLink,
  RefreshCw,
  CheckCircle,
  Loader2,
  Globe,
  Bell,
  HardDrive,
  Network,
  Download,
  Sparkles,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useSettingsStore } from "@/stores/settingsStore";
import {
  THEME_LABELS,
  LANGUAGE_LABELS,
  LOG_SIZE_OPTIONS,
  type ThemeMode,
  type LanguageCode,
  type LogFileSizeOption,
} from "@/types/settings";
import { BackupPanel } from "../settings/BackupPanel";

export function SettingsView() {
  const {
    settings,
    appInfo,
    isLoading,
    isSaving,
    hasUnsavedChanges,
    error,
    setTheme,
    setLanguage,
    setCloseToTray,
    setShowNotifications,
    setStartWithWindows,
    toggleAutoStartService,
    setStartMinimized,
    setAddToSystemPath,
    toggleRuntimePath,
    setMaxLogFileSize,
    setMaxFilesToKeep,
    setRetentionDays,
    setAutoCleanup,
    saveSettings,
    resetToDefaults,
    cleanupOldLogs,
    clearError,
  } = useSettingsStore();

  const [showPasswordModal, setShowPasswordModal] = useState(false);
  const [cleanupSuccess, setCleanupSuccess] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);

  // Update state
  const [isCheckingUpdate, setIsCheckingUpdate] = useState(false);
  const [updateCheckResult, setUpdateCheckResult] = useState<{
    updateAvailable: boolean;
    currentVersion: string;
    latestVersion: string | null;
    updateInfo: {
      version: string;
      downloadUrl: string;
      releaseNotes: string;
      publishedAt: string;
      isPrerelease: boolean;
      assetName: string | null;
      assetSize: number | null;
    } | null;
    error: string | null;
  } | null>(null);
  const [isDownloading, setIsDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState<{
    downloadedBytes: number;
    totalBytes: number;
    percentage: number;
  } | null>(null);
  const [downloadedFilePath, setDownloadedFilePath] = useState<string | null>(null);

  // Orphan hosts state
  const [orphanHosts, setOrphanHosts] = useState<Array<{ domain: string; ip: string; comment: string | null }>>([]);
  const [isLoadingOrphans, setIsLoadingOrphans] = useState(false);
  const [isCleaningOrphans, setIsCleaningOrphans] = useState(false);
  const [orphanCleanupSuccess, setOrphanCleanupSuccess] = useState(false);

  // Load orphan hosts on mount
  useEffect(() => {
    loadOrphanHosts();
  }, []);

  // Listen for download progress
  useEffect(() => {
    const unlisten = listen<{ downloadedBytes: number; totalBytes: number; percentage: number }>(
      "download-progress",
      (event) => {
        setDownloadProgress(event.payload);
      }
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleCheckForUpdates = async () => {
    setIsCheckingUpdate(true);
    setUpdateCheckResult(null);
    try {
      const result = await invoke<typeof updateCheckResult>("check_for_updates");
      setUpdateCheckResult(result);
    } catch (err) {
      console.error("Failed to check for updates:", err);
      setUpdateCheckResult({
        updateAvailable: false,
        currentVersion: appInfo.version,
        latestVersion: null,
        updateInfo: null,
        error: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setIsCheckingUpdate(false);
    }
  };

  const handleDownloadUpdate = async () => {
    if (!updateCheckResult?.updateInfo) return;

    setIsDownloading(true);
    setDownloadProgress(null);
    setDownloadedFilePath(null);
    try {
      const filePath = await invoke<string>("download_update_with_progress", {
        updateInfo: updateCheckResult.updateInfo,
      });
      setDownloadedFilePath(filePath);
    } catch (err) {
      console.error("Failed to download update:", err);
    } finally {
      setIsDownloading(false);
    }
  };

  const handleInstallUpdate = async () => {
    if (!downloadedFilePath) return;
    try {
      await invoke("install_update_and_quit", { filePath: downloadedFilePath });
    } catch (err) {
      console.error("Failed to install update:", err);
    }
  };

  const handleOpenReleasesPage = async () => {
    try {
      const url = await invoke<string>("get_releases_url");
      await invoke("open_in_browser", { url });
    } catch (err) {
      console.error("Failed to open releases page:", err);
    }
  };

  const loadOrphanHosts = async () => {
    setIsLoadingOrphans(true);
    try {
      const orphans = await invoke<Array<{ domain: string; ip: string; comment: string | null }>>("get_orphan_hosts");
      setOrphanHosts(orphans);
    } catch (err) {
      console.error("Failed to load orphan hosts:", err);
    } finally {
      setIsLoadingOrphans(false);
    }
  };

  const handleCleanupOrphanHosts = async () => {
    if (orphanHosts.length === 0) return;

    const domains = orphanHosts.map((h) => h.domain);
    setIsCleaningOrphans(true);
    try {
      await invoke("delete_orphan_hosts", { domains });
      setOrphanCleanupSuccess(true);
      setOrphanHosts([]);
      setTimeout(() => setOrphanCleanupSuccess(false), 2000);
    } catch (err) {
      console.error("Failed to cleanup orphan hosts:", err);
    } finally {
      setIsCleaningOrphans(false);
    }
  };

  const handleSave = async () => {
    await saveSettings();
    setSaveSuccess(true);
    setTimeout(() => setSaveSuccess(false), 2000);
  };

  const handleCleanupLogs = async () => {
    await cleanupOldLogs();
    setCleanupSuccess(true);
    setTimeout(() => setCleanupSuccess(false), 2000);
  };

  const handleResetToDefaults = () => {
    if (confirm("Are you sure you want to reset all settings to defaults?")) {
      resetToDefaults();
    }
  };

  const themeIcons: Record<ThemeMode, React.ReactNode> = {
    dark: <Moon size={16} />,
    light: <Sun size={16} />,
    system: <Monitor size={16} />,
  };

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-3xl mx-auto">
        {/* Header */}
        <div className="mb-6">
          <h1 className="text-2xl font-bold text-white">Settings</h1>
          <p className="text-slate-400 mt-1">
            Configure ClickDevPort preferences
          </p>
        </div>

        {/* Error Alert */}
        {error && (
          <div className="mb-6 p-4 bg-red-900/30 border border-red-700 rounded-lg flex items-start gap-3">
            <AlertTriangle className="text-red-400 shrink-0 mt-0.5" size={20} />
            <div className="flex-1">
              <p className="text-red-400">{error}</p>
            </div>
            <button
              onClick={clearError}
              className="text-red-400 hover:text-red-300"
            >
              Dismiss
            </button>
          </div>
        )}

        <div className="space-y-6">
          {/* General Settings */}
          <section className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center gap-2 mb-4">
              <Monitor className="text-blue-400" size={20} />
              <h2 className="text-lg font-semibold text-white">
                General Settings
              </h2>
            </div>

            <div className="space-y-5">
              {/* Theme Selection */}
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  App Theme
                </label>
                <div className="flex gap-2">
                  {(Object.keys(THEME_LABELS) as ThemeMode[]).map((theme) => (
                    <button
                      key={theme}
                      onClick={() => setTheme(theme)}
                      className={`flex items-center gap-2 px-4 py-2 rounded-lg border transition-colors ${
                        settings.general.theme === theme
                          ? "bg-blue-600 border-blue-500 text-white"
                          : "bg-slate-800 border-slate-700 text-slate-300 hover:border-slate-600"
                      }`}
                    >
                      {themeIcons[theme]}
                      {THEME_LABELS[theme]}
                    </button>
                  ))}
                </div>
                <p className="text-xs text-slate-500 mt-2">
                  Note: Light theme is coming in a future update
                </p>
              </div>

              {/* Language Selection */}
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  <Globe className="inline mr-2" size={16} />
                  Language
                </label>
                <select
                  value={settings.general.language}
                  onChange={(e) =>
                    setLanguage(e.target.value as LanguageCode)
                  }
                  className="w-full max-w-xs px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white focus:border-blue-500 focus:outline-none"
                >
                  {(Object.keys(LANGUAGE_LABELS) as LanguageCode[]).map(
                    (lang) => (
                      <option key={lang} value={lang}>
                        {LANGUAGE_LABELS[lang]}
                      </option>
                    )
                  )}
                </select>
                <p className="text-xs text-slate-500 mt-2">
                  Additional languages coming soon
                </p>
              </div>

              {/* Close to Tray */}
              <label className="flex items-start gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  checked={settings.general.closeToTray}
                  onChange={(e) => setCloseToTray(e.target.checked)}
                  className="mt-1 w-4 h-4 rounded border-slate-600 bg-slate-800 text-blue-600 focus:ring-blue-500 focus:ring-offset-slate-900"
                />
                <div>
                  <span className="text-white">
                    Minimize to system tray on close
                  </span>
                  <p className="text-sm text-slate-400">
                    When closing the window, minimize to tray instead of
                    quitting. Services will continue running.
                  </p>
                </div>
              </label>

              {/* Notifications */}
              <label className="flex items-start gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  checked={settings.general.showNotifications}
                  onChange={(e) => setShowNotifications(e.target.checked)}
                  className="mt-1 w-4 h-4 rounded border-slate-600 bg-slate-800 text-blue-600 focus:ring-blue-500 focus:ring-offset-slate-900"
                />
                <div>
                  <Bell className="inline mr-1 text-slate-400" size={16} />
                  <span className="text-white">Show desktop notifications</span>
                  <p className="text-sm text-slate-400">
                    Receive notifications for service status changes and errors
                  </p>
                </div>
              </label>
            </div>
          </section>

          {/* Auto-Start Settings */}
          <section className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center gap-2 mb-4">
              <Power className="text-green-400" size={20} />
              <h2 className="text-lg font-semibold text-white">
                Auto-Start Settings
              </h2>
            </div>

            <div className="space-y-4">
              <label className="flex items-start gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  checked={settings.autoStart.startWithWindows}
                  onChange={(e) => setStartWithWindows(e.target.checked)}
                  className="mt-1 w-4 h-4 rounded border-slate-600 bg-slate-800 text-blue-600 focus:ring-blue-500 focus:ring-offset-slate-900"
                />
                <div>
                  <span className="text-white">Start DevPort when Windows starts</span>
                  <p className="text-sm text-slate-400">
                    Automatically launch ClickDevPort when you log into Windows
                  </p>
                </div>
              </label>

              {settings.autoStart.startWithWindows && (
                <div className="ml-7 space-y-4">
                  <label className="flex items-start gap-3 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={settings.autoStart.startMinimized}
                      onChange={(e) => setStartMinimized(e.target.checked)}
                      className="mt-1 w-4 h-4 rounded border-slate-600 bg-slate-800 text-blue-600 focus:ring-blue-500 focus:ring-offset-slate-900"
                    />
                    <div>
                      <span className="text-slate-300">Start minimized to tray</span>
                    </div>
                  </label>

                  <div className="p-4 bg-slate-800/50 rounded-lg">
                    <p className="text-sm text-slate-300 mb-3">
                      Services to start automatically:
                    </p>
                    <div className="space-y-2">
                      <label className="flex items-center gap-3 cursor-pointer">
                        <input
                          type="checkbox"
                          checked={settings.autoStart.autoStartServices.includes(
                            "apache"
                          )}
                          onChange={() => toggleAutoStartService("apache")}
                          className="w-4 h-4 rounded border-slate-600 bg-slate-800 text-blue-600 focus:ring-blue-500 focus:ring-offset-slate-900"
                        />
                        <span className="text-slate-300">Apache</span>
                      </label>
                      <label className="flex items-center gap-3 cursor-pointer">
                        <input
                          type="checkbox"
                          checked={settings.autoStart.autoStartServices.includes(
                            "mariadb"
                          )}
                          onChange={() => toggleAutoStartService("mariadb")}
                          className="w-4 h-4 rounded border-slate-600 bg-slate-800 text-blue-600 focus:ring-blue-500 focus:ring-offset-slate-900"
                        />
                        <span className="text-slate-300">MariaDB</span>
                      </label>
                    </div>
                    <p className="text-xs text-slate-500 mt-3">
                      When enabled, selected services will start automatically with DevPort
                    </p>
                  </div>
                </div>
              )}
            </div>
          </section>

          {/* PATH Management */}
          <section className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center gap-2 mb-4">
              <HardDrive className="text-purple-400" size={20} />
              <h2 className="text-lg font-semibold text-white">
                PATH Management
              </h2>
            </div>

            <div className="space-y-4">
              <div className="p-4 bg-slate-800/50 rounded-lg">
                <p className="text-sm text-slate-300 mb-3">
                  DevPort Runtime Paths:
                </p>
                <div className="space-y-2 font-mono text-sm">
                  {settings.paths.runtimePaths.map((runtime) => (
                    <div
                      key={runtime.name}
                      className="flex items-center justify-between py-1"
                    >
                      <div className="flex items-center gap-3">
                        <span className="text-slate-400 w-24">
                          {runtime.name}:
                        </span>
                        <span className="text-slate-300">{runtime.path}</span>
                      </div>
                      {settings.paths.addToSystemPath && (
                        <label className="flex items-center gap-2 cursor-pointer">
                          <input
                            type="checkbox"
                            checked={runtime.enabled}
                            onChange={() => toggleRuntimePath(runtime.name)}
                            className="w-4 h-4 rounded border-slate-600 bg-slate-700 text-blue-600 focus:ring-blue-500"
                          />
                          <span className="text-xs text-slate-400">Include</span>
                        </label>
                      )}
                    </div>
                  ))}
                </div>
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <p className="text-white">Add DevPort runtime to system PATH</p>
                  <p className="text-sm text-slate-400">
                    Make bundled runtimes available in external terminals
                  </p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={settings.paths.addToSystemPath}
                    onChange={(e) => setAddToSystemPath(e.target.checked)}
                    className="sr-only peer"
                  />
                  <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                </label>
              </div>

              {settings.paths.addToSystemPath && (
                <div className="p-3 bg-yellow-900/20 border border-yellow-800 rounded-lg flex items-start gap-2">
                  <AlertTriangle
                    className="text-yellow-400 shrink-0 mt-0.5"
                    size={18}
                  />
                  <p className="text-sm text-yellow-400">
                    Warning: This may conflict with existing Node.js, PHP, or Git
                    installations on your system. Enable only the runtimes you need.
                  </p>
                </div>
              )}

              {/* Directory Paths */}
              <div className="pt-4 border-t border-slate-800 space-y-4">
                <div>
                  <label className="block text-sm text-slate-400 mb-2">
                    DevPort Installation Path
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={settings.paths.devportInstallPath}
                      readOnly
                      className="flex-1 px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-slate-300"
                    />
                    <button className="px-3 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors">
                      <FolderOpen size={18} />
                    </button>
                  </div>
                </div>

                <div>
                  <label className="block text-sm text-slate-400 mb-2">
                    Default Projects Directory
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={settings.paths.projectsDirectory}
                      readOnly
                      className="flex-1 px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-slate-300"
                    />
                    <button className="px-3 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors">
                      <FolderOpen size={18} />
                    </button>
                  </div>
                </div>

                <div>
                  <label className="block text-sm text-slate-400 mb-2">
                    Logs Directory
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={settings.paths.logsDirectory}
                      readOnly
                      className="flex-1 px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-slate-300"
                    />
                    <button className="px-3 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors">
                      <FolderOpen size={18} />
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </section>

          {/* Log Settings */}
          <section className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center gap-2 mb-4">
              <FileText className="text-orange-400" size={20} />
              <h2 className="text-lg font-semibold text-white">Log Settings</h2>
            </div>

            <div className="space-y-5">
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div>
                  <label className="block text-sm text-slate-400 mb-2">
                    Max log file size
                  </label>
                  <select
                    value={settings.logs.maxLogFileSize}
                    onChange={(e) =>
                      setMaxLogFileSize(
                        parseInt(e.target.value) as LogFileSizeOption
                      )
                    }
                    className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white focus:border-blue-500 focus:outline-none"
                  >
                    {LOG_SIZE_OPTIONS.map((size) => (
                      <option key={size} value={size}>
                        {size} MB
                      </option>
                    ))}
                  </select>
                </div>

                <div>
                  <label className="block text-sm text-slate-400 mb-2">
                    Max files to keep
                  </label>
                  <input
                    type="number"
                    min={1}
                    max={100}
                    value={settings.logs.maxFilesToKeep}
                    onChange={(e) =>
                      setMaxFilesToKeep(parseInt(e.target.value) || 1)
                    }
                    className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white focus:border-blue-500 focus:outline-none"
                  />
                </div>

                <div>
                  <label className="block text-sm text-slate-400 mb-2">
                    Retention days
                  </label>
                  <input
                    type="number"
                    min={1}
                    max={365}
                    value={settings.logs.retentionDays}
                    onChange={(e) =>
                      setRetentionDays(parseInt(e.target.value) || 1)
                    }
                    className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white focus:border-blue-500 focus:outline-none"
                  />
                </div>
              </div>

              <label className="flex items-start gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  checked={settings.logs.autoCleanup}
                  onChange={(e) => setAutoCleanup(e.target.checked)}
                  className="mt-1 w-4 h-4 rounded border-slate-600 bg-slate-800 text-blue-600 focus:ring-blue-500 focus:ring-offset-slate-900"
                />
                <div>
                  <span className="text-white">Enable automatic cleanup</span>
                  <p className="text-sm text-slate-400">
                    Automatically remove old log files based on settings above
                  </p>
                </div>
              </label>

              <div className="flex items-center gap-3">
                <button
                  onClick={handleCleanupLogs}
                  disabled={isLoading}
                  className="flex items-center gap-2 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors disabled:opacity-50"
                >
                  {isLoading ? (
                    <Loader2 size={18} className="animate-spin" />
                  ) : (
                    <Trash2 size={18} />
                  )}
                  Clean up old logs
                </button>
                {cleanupSuccess && (
                  <span className="flex items-center gap-1 text-green-400 text-sm">
                    <CheckCircle size={16} />
                    Cleanup completed
                  </span>
                )}
              </div>
            </div>
          </section>

          {/* Database Settings */}
          <section className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center gap-2 mb-4">
              <Database className="text-cyan-400" size={20} />
              <h2 className="text-lg font-semibold text-white">
                Database Settings
              </h2>
            </div>

            <div className="space-y-4">
              <div className="flex items-center justify-between p-4 bg-slate-800/50 rounded-lg">
                <div>
                  <p className="text-white">MariaDB Root Password</p>
                  <p className="text-sm text-slate-400">
                    Change the root password for MariaDB
                  </p>
                </div>
                <button
                  onClick={() => setShowPasswordModal(true)}
                  className="flex items-center gap-2 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors"
                >
                  <Key size={18} />
                  Change Password
                </button>
              </div>

              <div>
                <label className="block text-sm text-slate-400 mb-2">
                  Default Backup Location
                </label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={settings.database.defaultBackupLocation}
                    readOnly
                    className="flex-1 px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-slate-300"
                  />
                  <button className="px-3 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors">
                    <FolderOpen size={18} />
                  </button>
                </div>
              </div>
            </div>
          </section>

          {/* Hosts Management */}
          <section className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center gap-2 mb-4">
              <Network className="text-pink-400" size={20} />
              <h2 className="text-lg font-semibold text-white">
                Hosts File Management
              </h2>
            </div>

            <div className="space-y-4">
              <p className="text-sm text-slate-400">
                DevPort automatically manages entries in your hosts file for custom domains.
                Orphan entries are domains that no longer have associated projects.
              </p>

              {/* Orphan Hosts List */}
              <div className="p-4 bg-slate-800/50 rounded-lg">
                <div className="flex items-center justify-between mb-3">
                  <p className="text-sm text-slate-300">
                    Orphan Hosts Entries
                  </p>
                  <button
                    onClick={loadOrphanHosts}
                    disabled={isLoadingOrphans}
                    className="text-slate-400 hover:text-white transition-colors"
                    title="Refresh"
                  >
                    <RefreshCw size={16} className={isLoadingOrphans ? "animate-spin" : ""} />
                  </button>
                </div>

                {isLoadingOrphans ? (
                  <div className="flex items-center justify-center py-4">
                    <Loader2 size={20} className="animate-spin text-slate-400" />
                  </div>
                ) : orphanHosts.length === 0 ? (
                  <div className="flex items-center gap-2 py-4 text-green-400">
                    <CheckCircle size={16} />
                    <span className="text-sm">No orphan entries found</span>
                  </div>
                ) : (
                  <div className="space-y-2">
                    {orphanHosts.map((host) => (
                      <div
                        key={host.domain}
                        className="flex items-center justify-between py-2 px-3 bg-slate-700/50 rounded"
                      >
                        <div className="flex items-center gap-3">
                          <Globe size={14} className="text-slate-400" />
                          <span className="font-mono text-sm text-white">{host.domain}</span>
                          <span className="text-xs text-slate-500">→ {host.ip}</span>
                        </div>
                        {host.comment && (
                          <span className="text-xs text-slate-500">{host.comment}</span>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>

              {/* Cleanup Button */}
              {orphanHosts.length > 0 && (
                <div className="flex items-center gap-3">
                  <button
                    onClick={handleCleanupOrphanHosts}
                    disabled={isCleaningOrphans}
                    className="flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors disabled:opacity-50"
                  >
                    {isCleaningOrphans ? (
                      <Loader2 size={18} className="animate-spin" />
                    ) : (
                      <Trash2 size={18} />
                    )}
                    Clean up {orphanHosts.length} orphan {orphanHosts.length === 1 ? "entry" : "entries"}
                  </button>
                  {orphanCleanupSuccess && (
                    <span className="flex items-center gap-1 text-green-400 text-sm">
                      <CheckCircle size={16} />
                      Cleaned up successfully
                    </span>
                  )}
                </div>
              )}

              <div className="p-3 bg-yellow-900/20 border border-yellow-800 rounded-lg flex items-start gap-2">
                <AlertTriangle className="text-yellow-400 shrink-0 mt-0.5" size={18} />
                <p className="text-sm text-yellow-400">
                  Note: Cleaning orphan entries will not affect running applications,
                  but custom domains will stop working. Use localhost:port instead.
                </p>
              </div>
            </div>
          </section>

          {/* Backup & Restore */}
          <BackupPanel />

          {/* About & Updates */}
          <section className="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <div className="flex items-center gap-2 mb-4">
              <Info className="text-slate-400" size={20} />
              <h2 className="text-lg font-semibold text-white">About & Updates</h2>
            </div>

            <div className="space-y-4">
              {/* Version Info */}
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-slate-400">Version:</span>
                  <span className="ml-2 text-white font-semibold">{appInfo.version}</span>
                </div>
                <div>
                  <span className="text-slate-400">Tauri:</span>
                  <span className="ml-2 text-white">{appInfo.tauriVersion}</span>
                </div>
                <div>
                  <span className="text-slate-400">Build:</span>
                  <span className="ml-2 text-white capitalize">
                    {appInfo.buildType}
                  </span>
                </div>
                <div>
                  <span className="text-slate-400">Build Date:</span>
                  <span className="ml-2 text-white">{appInfo.buildDate}</span>
                </div>
              </div>

              {/* Update Check Section */}
              <div className="pt-4 border-t border-slate-800">
                <div className="flex items-center justify-between mb-4">
                  <div>
                    <h3 className="text-white font-medium">Software Updates</h3>
                    <p className="text-sm text-slate-400">Check for new versions of ClickDevPort</p>
                  </div>
                  <button
                    onClick={handleCheckForUpdates}
                    disabled={isCheckingUpdate}
                    className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors disabled:opacity-50"
                  >
                    {isCheckingUpdate ? (
                      <Loader2 size={18} className="animate-spin" />
                    ) : (
                      <RefreshCw size={18} />
                    )}
                    Check for Updates
                  </button>
                </div>

                {/* Update Check Result */}
                {updateCheckResult && (
                  <div className="p-4 bg-slate-800/50 rounded-lg space-y-3">
                    {updateCheckResult.error ? (
                      <div className="flex items-center gap-2 text-red-400">
                        <AlertTriangle size={18} />
                        <span>Failed to check: {updateCheckResult.error}</span>
                      </div>
                    ) : updateCheckResult.updateAvailable && updateCheckResult.updateInfo ? (
                      <>
                        <div className="flex items-center gap-2 text-green-400">
                          <Sparkles size={18} />
                          <span className="font-medium">
                            New version available: v{updateCheckResult.updateInfo.version}
                          </span>
                        </div>
                        <p className="text-sm text-slate-400">
                          Current: v{updateCheckResult.currentVersion} → New: v{updateCheckResult.updateInfo.version}
                        </p>
                        {updateCheckResult.updateInfo.releaseNotes && (
                          <div className="p-3 bg-slate-900 rounded border border-slate-700 max-h-32 overflow-y-auto">
                            <p className="text-xs text-slate-500 mb-1">Release Notes:</p>
                            <p className="text-sm text-slate-300 whitespace-pre-wrap">
                              {updateCheckResult.updateInfo.releaseNotes}
                            </p>
                          </div>
                        )}
                        {updateCheckResult.updateInfo.assetSize && (
                          <p className="text-xs text-slate-500">
                            Download size: {(updateCheckResult.updateInfo.assetSize / 1024 / 1024).toFixed(1)} MB
                          </p>
                        )}

                        {/* Download Progress */}
                        {isDownloading && downloadProgress && (
                          <div className="space-y-2">
                            <div className="flex items-center justify-between text-sm">
                              <span className="text-slate-400">Downloading...</span>
                              <span className="text-white">{downloadProgress.percentage.toFixed(0)}%</span>
                            </div>
                            <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
                              <div
                                className="h-full bg-blue-500 transition-all"
                                style={{ width: `${downloadProgress.percentage}%` }}
                              />
                            </div>
                          </div>
                        )}

                        {/* Download/Install Buttons */}
                        <div className="flex items-center gap-3 pt-2">
                          {!downloadedFilePath ? (
                            <button
                              onClick={handleDownloadUpdate}
                              disabled={isDownloading}
                              className="flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors disabled:opacity-50"
                            >
                              {isDownloading ? (
                                <Loader2 size={18} className="animate-spin" />
                              ) : (
                                <Download size={18} />
                              )}
                              Download Update
                            </button>
                          ) : (
                            <button
                              onClick={handleInstallUpdate}
                              className="flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors"
                            >
                              <Download size={18} />
                              Install & Restart
                            </button>
                          )}
                          <button
                            onClick={handleOpenReleasesPage}
                            className="flex items-center gap-2 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors"
                          >
                            <ExternalLink size={18} />
                            View on GitHub
                          </button>
                        </div>
                      </>
                    ) : (
                      <div className="flex items-center gap-2 text-green-400">
                        <CheckCircle size={18} />
                        <span>You're using the latest version (v{updateCheckResult.currentVersion})</span>
                      </div>
                    )}
                  </div>
                )}
              </div>

              {/* Links */}
              <div className="pt-4 border-t border-slate-800 flex gap-4">
                <button
                  onClick={handleOpenReleasesPage}
                  className="flex items-center gap-1 text-blue-400 hover:text-blue-300 text-sm transition-colors"
                >
                  <ExternalLink size={14} />
                  Releases
                </button>
                <a
                  href="#"
                  className="flex items-center gap-1 text-blue-400 hover:text-blue-300 text-sm transition-colors"
                >
                  <ExternalLink size={14} />
                  Documentation
                </a>
                <a
                  href="#"
                  className="flex items-center gap-1 text-blue-400 hover:text-blue-300 text-sm transition-colors"
                >
                  <ExternalLink size={14} />
                  Report an Issue
                </a>
              </div>
            </div>
          </section>

          {/* Action Buttons */}
          <div className="flex items-center justify-between pt-4">
            <button
              onClick={handleResetToDefaults}
              className="flex items-center gap-2 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors"
            >
              <RefreshCw size={18} />
              Reset to Defaults
            </button>

            <div className="flex items-center gap-3">
              {hasUnsavedChanges && (
                <span className="text-amber-400 text-sm">Unsaved changes</span>
              )}
              {saveSuccess && (
                <span className="flex items-center gap-1 text-green-400 text-sm">
                  <CheckCircle size={16} />
                  Saved
                </span>
              )}
              <button
                onClick={handleSave}
                disabled={isSaving || !hasUnsavedChanges}
                className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isSaving ? (
                  <Loader2 size={18} className="animate-spin" />
                ) : (
                  <Save size={18} />
                )}
                Save Settings
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Password Change Modal */}
      {showPasswordModal && (
        <PasswordChangeModal onClose={() => setShowPasswordModal(false)} />
      )}
    </div>
  );
}

interface PasswordChangeModalProps {
  onClose: () => void;
}

function PasswordChangeModal({ onClose }: PasswordChangeModalProps) {
  const { changeMariadbPassword, isLoading, error } = useSettingsStore();
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [validationError, setValidationError] = useState("");
  const [success, setSuccess] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setValidationError("");

    if (newPassword.length < 8) {
      setValidationError("Password must be at least 8 characters long");
      return;
    }

    if (newPassword !== confirmPassword) {
      setValidationError("Passwords do not match");
      return;
    }

    await changeMariadbPassword(newPassword);
    if (!error) {
      setSuccess(true);
      setTimeout(() => {
        onClose();
      }, 1500);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-slate-900 border border-slate-700 rounded-lg p-6 w-full max-w-md">
        <h3 className="text-lg font-semibold text-white mb-4">
          Change MariaDB Root Password
        </h3>

        {success ? (
          <div className="flex flex-col items-center py-8">
            <CheckCircle className="text-green-400 mb-3" size={48} />
            <p className="text-green-400">Password changed successfully!</p>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block text-sm text-slate-400 mb-2">
                Current Password
              </label>
              <input
                type="password"
                value={currentPassword}
                onChange={(e) => setCurrentPassword(e.target.value)}
                className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white focus:border-blue-500 focus:outline-none"
                required
              />
            </div>

            <div>
              <label className="block text-sm text-slate-400 mb-2">
                New Password
              </label>
              <input
                type="password"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white focus:border-blue-500 focus:outline-none"
                required
                minLength={8}
              />
            </div>

            <div>
              <label className="block text-sm text-slate-400 mb-2">
                Confirm New Password
              </label>
              <input
                type="password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white focus:border-blue-500 focus:outline-none"
                required
              />
            </div>

            {(validationError || error) && (
              <p className="text-red-400 text-sm">{validationError || error}</p>
            )}

            <div className="flex justify-end gap-3 pt-2">
              <button
                type="button"
                onClick={onClose}
                className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={isLoading}
                className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors disabled:opacity-50"
              >
                {isLoading ? (
                  <Loader2 size={18} className="animate-spin" />
                ) : (
                  <Key size={18} />
                )}
                Change Password
              </button>
            </div>
          </form>
        )}
      </div>
    </div>
  );
}
