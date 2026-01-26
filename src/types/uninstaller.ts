/**
 * Uninstall mode determines what gets removed
 */
export type UninstallMode = 'basic' | 'fullData' | 'systemRevert';

/**
 * Type of item being uninstalled
 */
export type UninstallItemType =
  | 'executable'
  | 'directory'
  | 'file'
  | 'taskScheduler'
  | 'hostsEntry'
  | 'firewallRule'
  | 'shortcut'
  | 'registryKey';

/**
 * A single item in the uninstall preview
 */
export interface UninstallPreviewItem {
  itemType: UninstallItemType;
  path: string | null;
  name: string;
  sizeBytes: number | null;
  exists: boolean;
}

/**
 * Preview of what will be deleted
 */
export interface UninstallPreview {
  mode: UninstallMode;
  items: UninstallPreviewItem[];
  totalSizeBytes: number;
  requiresAdmin: boolean;
  warnings: string[];
}

/**
 * An item that was successfully removed
 */
export interface RemovedItem {
  itemType: UninstallItemType;
  path: string | null;
  name: string;
}

/**
 * An item that failed to be removed
 */
export interface FailedItem {
  itemType: UninstallItemType;
  path: string | null;
  name: string;
  reason: string;
}

/**
 * Result of an uninstall operation
 */
export interface UninstallResult {
  success: boolean;
  mode: UninstallMode;
  removedItems: RemovedItem[];
  failedItems: FailedItem[];
  servicesStopped: boolean;
  projectsStopped: boolean;
  requiresReboot: boolean;
  errorMessage: string | null;
}

/**
 * Information about running processes
 */
export interface RunningProcessesInfo {
  apacheRunning: boolean;
  mariadbRunning: boolean;
  anyRunning: boolean;
}

/**
 * Labels for uninstall modes
 */
export const UNINSTALL_MODE_LABELS: Record<UninstallMode, string> = {
  basic: 'Basic (App + Runtime only)',
  fullData: 'Full Data (Include projects, backups, logs)',
  systemRevert: 'System Revert (Full cleanup + system changes)',
};

/**
 * Descriptions for uninstall modes
 */
export const UNINSTALL_MODE_DESCRIPTIONS: Record<UninstallMode, string> = {
  basic:
    'Removes DevPort app and runtime bundles (Apache, MariaDB, PHP, Node.js, Git). Keeps your projects, backups, and logs.',
  fullData:
    'Removes everything including projects, database backups, and log files. All project data will be permanently deleted.',
  systemRevert:
    'Complete removal including Task Scheduler entries, hosts file modifications, firewall rules, and shortcuts. Requires administrator privileges.',
};

/**
 * Icons for item types (using Lucide icon names)
 */
export const ITEM_TYPE_ICONS: Record<UninstallItemType, string> = {
  executable: 'FileCode',
  directory: 'Folder',
  file: 'File',
  taskScheduler: 'Calendar',
  hostsEntry: 'Globe',
  firewallRule: 'Shield',
  shortcut: 'Link',
  registryKey: 'Key',
};

/**
 * Format bytes to human readable string (for uninstaller display)
 */
export function formatUninstallBytes(bytes: number): string {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}
