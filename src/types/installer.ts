// Component category enum
export type ComponentCategory =
  | "WebServer"
  | "Database"
  | "Runtime"
  | "PackageManager"
  | "DevTool";

// Post-install action types
export type PostInstallAction =
  | "setPath"
  | "configureIni"
  | "linkToApache"
  | "initDatabase"
  | "setupService"
  | "verifyInstall";

// Bundle component definition
export interface BundleComponent {
  id: string;
  name: string;
  category: ComponentCategory;
  version: string;
  fileName: string | null;
  downloadUrl: string | null;
  sizeBytes: number;
  sha256: string | null;
  installPath: string;
  executablePath: string | null;
  postInstall: PostInstallAction[];
  dependencies: string[];
  description: string;
  icon: string | null;
}

// Installation preset
export interface InstallPreset {
  id: string;
  name: string;
  description: string;
  icon: string;
  components: string[];
  optionalComponents: string[];
}

// Bundle manifest (full config)
export interface BundleManifest {
  version: string;
  components: Record<string, BundleComponent>;
  presets: Record<string, InstallPreset>;
}

// Install phase enum
export type InstallPhase =
  | "pending"
  | "downloading"
  | "extracting"
  | "configuring"
  | "verifying"
  | "completed"
  | "failed";

// Install progress for a single component
export interface InstallProgress {
  componentId: string;
  componentName: string;
  phase: InstallPhase;
  progressPercent: number;
  message: string;
  error: string | null;
}

// Overall installation state
export interface InstallationState {
  isInstalling: boolean;
  selectedPreset: string | null;
  selectedComponents: string[];
  progress: InstallProgress[];
  currentComponent: string | null;
  overallProgress: number;
  totalSizeBytes: number;
  completedCount: number;
  totalCount: number;
  error: string | null;
}

// Installed component record
export interface InstalledComponent {
  id: string;
  name: string;
  version: string;
  installPath: string;
  installedAt: string;
  sizeBytes: number;
}

// Installation options for a single component
export interface InstallOptions {
  componentId: string;
  version: string | null;
  autoConfigure: boolean;
  addToPath: boolean;
  linkToApache: boolean;
}

// Component info for frontend display (from commands)
export interface ComponentInfo {
  id: string;
  name: string;
  category: string;
  version: string;
  sizeBytes: number;
  sizeMb: number;
  description: string;
  icon: string | null;
  isInstalled: boolean;
  installedVersion: string | null;
  installedSource: string | null;
  hasBundle: boolean;
  dependencies: string[];
}

// Preset info for frontend display
export interface PresetInfo {
  id: string;
  name: string;
  description: string;
  icon: string;
  components: string[];
  optionalComponents: string[];
  totalSizeBytes: number;
  totalSizeMb: number;
}

// Category group for organized display
export interface CategoryGroup {
  category: string;
  displayName: string;
  components: ComponentInfo[];
}

// Download status enum
export type DownloadStatus =
  | "pending"
  | "downloading"
  | "verifying"
  | "completed"
  | "failed"
  | "cancelled";

// Download progress event for installer
export interface InstallerDownloadProgress {
  componentId: string;
  componentName: string;
  downloadedBytes: number;
  totalBytes: number;
  progressPercent: number;
  speedBytesPerSec: number;
  etaSeconds: number;
  status: DownloadStatus;
  error: string | null;
}

// Installation summary
export interface InstallationSummary {
  totalComponents: number;
  installedCount: number;
  installedSizeBytes: number;
  installedSizeMb: number;
}

// Category display metadata
export const CATEGORY_DISPLAY_NAMES: Record<ComponentCategory, string> = {
  WebServer: "Web Servers",
  Database: "Databases",
  Runtime: "Runtimes",
  PackageManager: "Package Managers",
  DevTool: "Dev Tools",
};

// Phase display names
export const INSTALL_PHASE_NAMES: Record<InstallPhase, string> = {
  pending: "대기 중",
  downloading: "다운로드 중",
  extracting: "압축 해제 중",
  configuring: "구성 중",
  verifying: "검증 중",
  completed: "완료",
  failed: "실패",
};

// Download status display names
export const DOWNLOAD_STATUS_NAMES: Record<DownloadStatus, string> = {
  pending: "대기 중",
  downloading: "다운로드 중",
  verifying: "검증 중",
  completed: "완료",
  failed: "실패",
  cancelled: "취소됨",
};

// Icon mapping for categories (Lucide icon names)
export const CATEGORY_ICONS: Record<ComponentCategory, string> = {
  WebServer: "globe",
  Database: "database",
  Runtime: "cpu",
  PackageManager: "package",
  DevTool: "wrench",
};

// Format bytes to human readable (installer specific)
export function formatInstallerBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

// Format speed to human readable
export function formatSpeed(bytesPerSec: number): string {
  return `${formatInstallerBytes(bytesPerSec)}/s`;
}

// Format ETA to human readable
export function formatEta(seconds: number): string {
  if (seconds < 60) return `${seconds}초`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}분`;
  return `${Math.floor(seconds / 3600)}시간 ${Math.floor((seconds % 3600) / 60)}분`;
}
