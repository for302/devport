export type ThemeMode = "dark" | "light" | "system";
export type LanguageCode = "en" | "ko" | "ja" | "zh";

export interface GeneralSettings {
  theme: ThemeMode;
  language: LanguageCode;
  closeToTray: boolean;
  showNotifications: boolean;
}

export interface AutoStartSettings {
  startWithWindows: boolean;
  autoStartServices: string[];
  startMinimized: boolean;
}

export interface RuntimePath {
  name: string;
  path: string;
  enabled: boolean;
}

export interface PathSettings {
  addToSystemPath: boolean;
  runtimePaths: RuntimePath[];
  devportInstallPath: string;
  projectsDirectory: string;
  logsDirectory: string;
}

export type LogFileSizeOption = 10 | 50 | 100 | 250 | 500;

export interface LogSettings {
  maxLogFileSize: LogFileSizeOption;
  maxFilesToKeep: number;
  retentionDays: number;
  autoCleanup: boolean;
}

export interface DatabaseSettings {
  mariadbRootPassword: string;
  defaultBackupLocation: string;
  autoBackup: boolean;
  backupSchedule: "daily" | "weekly" | "manual";
}

export interface AppInfo {
  version: string;
  tauriVersion: string;
  buildType: "development" | "production";
  buildDate: string;
}

export interface AppSettings {
  general: GeneralSettings;
  autoStart: AutoStartSettings;
  paths: PathSettings;
  logs: LogSettings;
  database: DatabaseSettings;
}

export const DEFAULT_SETTINGS: AppSettings = {
  general: {
    theme: "dark",
    language: "en",
    closeToTray: true,
    showNotifications: true,
  },
  autoStart: {
    startWithWindows: false,
    autoStartServices: [],
    startMinimized: true,
  },
  paths: {
    addToSystemPath: false,
    runtimePaths: [
      { name: "Node.js", path: "C:\\DevPort\\runtime\\nodejs", enabled: true },
      { name: "PHP", path: "C:\\DevPort\\runtime\\php", enabled: true },
      { name: "Git", path: "C:\\DevPort\\runtime\\git", enabled: true },
      { name: "Composer", path: "C:\\DevPort\\runtime\\composer", enabled: true },
    ],
    devportInstallPath: "C:\\DevPort",
    projectsDirectory: "C:\\DevPort\\projects",
    logsDirectory: "C:\\DevPort\\logs",
  },
  logs: {
    maxLogFileSize: 50,
    maxFilesToKeep: 5,
    retentionDays: 30,
    autoCleanup: true,
  },
  database: {
    mariadbRootPassword: "",
    defaultBackupLocation: "C:\\DevPort\\backups",
    autoBackup: false,
    backupSchedule: "manual",
  },
};

export const THEME_LABELS: Record<ThemeMode, string> = {
  dark: "Dark",
  light: "Light",
  system: "System",
};

export const LANGUAGE_LABELS: Record<LanguageCode, string> = {
  en: "English",
  ko: "Korean",
  ja: "Japanese",
  zh: "Chinese",
};

export const LOG_SIZE_OPTIONS: LogFileSizeOption[] = [10, 50, 100, 250, 500];
