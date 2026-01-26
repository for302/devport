export type ConfigType = "apache" | "mariadb" | "php";

export interface ConfigFile {
  type: ConfigType;
  name: string;
  path: string;
  content: string;
  lastModified: string | null;
  hasBackup: boolean;
}

export interface ConfigSection {
  name: string;
  startLine: number;
  endLine: number;
  content: string;
}

export interface ConfigValidationResult {
  valid: boolean;
  message: string;
  errors: ConfigError[];
}

export interface ConfigError {
  line: number;
  message: string;
  severity: "error" | "warning";
}

export const CONFIG_TYPE_LABELS: Record<ConfigType, string> = {
  apache: "Apache",
  mariadb: "MariaDB",
  php: "PHP",
};

export const CONFIG_FILE_NAMES: Record<ConfigType, string> = {
  apache: "httpd.conf",
  mariadb: "my.ini",
  php: "php.ini",
};
