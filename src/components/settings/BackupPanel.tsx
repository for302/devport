import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Archive,
  Download,
  Upload,
  Trash2,
  FolderOpen,
  Loader2,
  CheckCircle,
  AlertTriangle,
  RefreshCw,
  FileArchive,
} from "lucide-react";

interface BackupInfo {
  filePath: string;
  fileName: string;
  createdAt: string;
  sizeBytes: number;
}

interface BackupResult {
  success: boolean;
  backupPath: string | null;
  filesBackedUp: string[];
  error: string | null;
}

interface RestoreResult {
  success: boolean;
  filesRestored: string[];
  error: string | null;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function BackupPanel() {
  const [configPath, setConfigPath] = useState<string>("");
  const [backups, setBackups] = useState<BackupInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isCreatingBackup, setIsCreatingBackup] = useState(false);
  const [isRestoring, setIsRestoring] = useState(false);
  const [backupSuccess, setBackupSuccess] = useState<string | null>(null);
  const [restoreSuccess, setRestoreSuccess] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadConfigPath();
    loadBackups();
  }, []);

  const loadConfigPath = async () => {
    try {
      const path = await invoke<string>("get_config_path");
      setConfigPath(path);
    } catch (err) {
      console.error("Failed to get config path:", err);
    }
  };

  const loadBackups = async () => {
    setIsLoading(true);
    try {
      const list = await invoke<BackupInfo[]>("list_backups");
      setBackups(list);
    } catch (err) {
      console.error("Failed to load backups:", err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreateBackup = async () => {
    setIsCreatingBackup(true);
    setError(null);
    setBackupSuccess(null);

    try {
      const result = await invoke<BackupResult>("create_backup", { customPath: null });
      if (result.success && result.backupPath) {
        setBackupSuccess(result.backupPath);
        loadBackups();
        setTimeout(() => setBackupSuccess(null), 5000);
      } else {
        setError(result.error || "Backup failed");
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setIsCreatingBackup(false);
    }
  };

  const handleRestoreBackup = async (backupPath: string) => {
    if (!confirm("설정을 복원하시겠습니까? 현재 설정이 덮어쓰기됩니다.")) {
      return;
    }

    setIsRestoring(true);
    setError(null);

    try {
      const result = await invoke<RestoreResult>("restore_backup", { backupPath });
      if (result.success) {
        setRestoreSuccess(true);
        setTimeout(() => setRestoreSuccess(false), 3000);
      } else {
        setError(result.error || "Restore failed");
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setIsRestoring(false);
    }
  };

  const handleRestoreFromFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "Backup Files", extensions: ["zip"] }],
      });

      if (selected) {
        await handleRestoreBackup(selected as string);
      }
    } catch (err) {
      setError(String(err));
    }
  };

  const handleDeleteBackup = async (backupPath: string) => {
    if (!confirm("이 백업 파일을 삭제하시겠습니까?")) {
      return;
    }

    try {
      await invoke("delete_backup", { backupPath });
      loadBackups();
    } catch (err) {
      setError(String(err));
    }
  };

  const handleOpenBackupFolder = async () => {
    try {
      await invoke("open_backup_folder");
    } catch (err) {
      setError(String(err));
    }
  };

  const handleOpenConfigFolder = async () => {
    try {
      await invoke("open_config_folder");
    } catch (err) {
      setError(String(err));
    }
  };

  return (
    <section className="bg-slate-900 border border-slate-800 rounded-lg p-6">
      <div className="flex items-center gap-2 mb-4">
        <Archive className="text-indigo-400" size={20} />
        <h2 className="text-lg font-semibold text-white">설정 백업 및 복원</h2>
      </div>

      <div className="space-y-4">
        {/* Config Path Info */}
        <div className="p-4 bg-slate-800/50 rounded-lg">
          <p className="text-sm text-slate-400 mb-2">설정 파일 저장 위치:</p>
          <div className="flex items-center gap-2">
            <code className="flex-1 text-sm text-slate-300 bg-slate-700/50 px-3 py-2 rounded">
              {configPath || "Loading..."}
            </code>
            <button
              onClick={handleOpenConfigFolder}
              className="px-3 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors"
              title="폴더 열기"
            >
              <FolderOpen size={18} />
            </button>
          </div>
        </div>

        {/* Error Message */}
        {error && (
          <div className="p-3 bg-red-900/20 border border-red-800 rounded-lg flex items-start gap-2">
            <AlertTriangle className="text-red-400 shrink-0 mt-0.5" size={18} />
            <p className="text-sm text-red-400">{error}</p>
          </div>
        )}

        {/* Success Messages */}
        {backupSuccess && (
          <div className="p-3 bg-green-900/20 border border-green-800 rounded-lg flex items-start gap-2">
            <CheckCircle className="text-green-400 shrink-0 mt-0.5" size={18} />
            <div className="text-sm text-green-400">
              <p>백업이 생성되었습니다:</p>
              <code className="text-xs">{backupSuccess}</code>
            </div>
          </div>
        )}

        {restoreSuccess && (
          <div className="p-3 bg-green-900/20 border border-green-800 rounded-lg flex items-center gap-2">
            <CheckCircle className="text-green-400" size={18} />
            <p className="text-sm text-green-400">설정이 복원되었습니다. 앱을 재시작해주세요.</p>
          </div>
        )}

        {/* Action Buttons */}
        <div className="flex flex-wrap gap-3">
          <button
            onClick={handleCreateBackup}
            disabled={isCreatingBackup}
            className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors disabled:opacity-50"
          >
            {isCreatingBackup ? (
              <Loader2 size={18} className="animate-spin" />
            ) : (
              <Download size={18} />
            )}
            백업 생성
          </button>

          <button
            onClick={handleRestoreFromFile}
            disabled={isRestoring}
            className="flex items-center gap-2 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors disabled:opacity-50"
          >
            {isRestoring ? (
              <Loader2 size={18} className="animate-spin" />
            ) : (
              <Upload size={18} />
            )}
            파일에서 복원
          </button>

          <button
            onClick={handleOpenBackupFolder}
            className="flex items-center gap-2 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors"
          >
            <FolderOpen size={18} />
            백업 폴더 열기
          </button>
        </div>

        {/* Backups List */}
        <div className="pt-4 border-t border-slate-800">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-medium text-slate-300">저장된 백업</h3>
            <button
              onClick={loadBackups}
              disabled={isLoading}
              className="text-slate-400 hover:text-white transition-colors"
              title="새로고침"
            >
              <RefreshCw size={16} className={isLoading ? "animate-spin" : ""} />
            </button>
          </div>

          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 size={24} className="animate-spin text-slate-400" />
            </div>
          ) : backups.length === 0 ? (
            <div className="text-center py-8 text-slate-500">
              <FileArchive size={40} className="mx-auto mb-2 opacity-50" />
              <p>저장된 백업이 없습니다</p>
            </div>
          ) : (
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {backups.map((backup) => (
                <div
                  key={backup.filePath}
                  className="flex items-center justify-between p-3 bg-slate-800/50 rounded-lg hover:bg-slate-800 transition-colors"
                >
                  <div className="flex items-center gap-3">
                    <FileArchive className="text-slate-400" size={20} />
                    <div>
                      <p className="text-sm text-white">{backup.fileName}</p>
                      <p className="text-xs text-slate-500">
                        {backup.createdAt} • {formatBytes(backup.sizeBytes)}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => handleRestoreBackup(backup.filePath)}
                      disabled={isRestoring}
                      className="p-2 text-blue-400 hover:bg-blue-500/20 rounded transition-colors"
                      title="복원"
                    >
                      <Upload size={16} />
                    </button>
                    <button
                      onClick={() => handleDeleteBackup(backup.filePath)}
                      className="p-2 text-red-400 hover:bg-red-500/20 rounded transition-colors"
                      title="삭제"
                    >
                      <Trash2 size={16} />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Info */}
        <div className="p-3 bg-slate-800/30 rounded-lg">
          <p className="text-xs text-slate-500">
            백업에는 프로젝트 목록, 설치된 컴포넌트 정보, 자동 시작 설정이 포함됩니다.
            <br />
            백업 파일은 <code className="text-slate-400">내 문서/ClickDevPort Backups</code> 폴더에 저장됩니다.
          </p>
        </div>
      </div>
    </section>
  );
}
