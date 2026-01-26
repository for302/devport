export type RuntimeType =
  | 'apache'
  | 'mariadb'
  | 'php'
  | 'nodejs'
  | 'git'
  | 'phpmyadmin'
  | 'composer';

export interface BundleStatus {
  runtimeType: RuntimeType;
  name: string;
  isInstalled: boolean;
  isValid: boolean;
  missingFiles: string[];
  basePath: string;
}

export interface RuntimeInfo {
  runtimeType: RuntimeType;
  name: string;
  version: string | null;
  isInstalled: boolean;
  basePath: string;
  executablePath: string;
}

export interface BundlePaths {
  devportBase: string;
  runtimeBase: string;
  toolsBase: string;
  apache: string;
  mariadb: string;
  php: string;
  nodejs: string;
  git: string;
  phpmyadmin: string;
  composer: string;
}

export interface BundleConfig {
  runtimeType: RuntimeType;
  basePath: string;
  executablePath: string;
  versionCommand: string;
  versionArgs: string[];
  requiredFiles: string[];
}

export const RUNTIME_DISPLAY_NAMES: Record<RuntimeType, string> = {
  apache: 'Apache HTTP Server',
  mariadb: 'MariaDB',
  php: 'PHP',
  nodejs: 'Node.js',
  git: 'Git',
  phpmyadmin: 'phpMyAdmin',
  composer: 'Composer',
};

export const RUNTIME_TYPES: RuntimeType[] = [
  'apache',
  'mariadb',
  'php',
  'nodejs',
  'git',
  'phpmyadmin',
  'composer',
];

export const CORE_RUNTIMES: RuntimeType[] = [
  'apache',
  'mariadb',
  'php',
  'nodejs',
  'git',
];

export const TOOL_RUNTIMES: RuntimeType[] = ['phpmyadmin', 'composer'];
