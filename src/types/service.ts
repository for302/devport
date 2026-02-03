export type ServiceStatus = 'running' | 'stopped' | 'error' | 'unhealthy' | 'notinstalled';
export type ServiceType = 'webserver' | 'database' | 'runtime' | 'tool';

export interface ConfigFileInfo {
  name: string;
  path: string;
  description: string;
}

export interface ServiceInfo {
  id: string;
  name: string;
  serviceType: ServiceType;
  port: number;
  status: ServiceStatus;
  pid: number | null;
  autoStart: boolean;
  autoRestart: boolean;
  lastStarted: string | null;
  lastStopped: string | null;
  errorMessage: string | null;
  installed: boolean;
  configFiles: ConfigFileInfo[];
}

export interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
  source: string;
}

export interface LogReadResult {
  entries: LogEntry[];
  totalSize: number;
  path: string;
}

export const SERVICE_STATUS_COLORS: Record<ServiceStatus, string> = {
  running: 'text-green-500',
  stopped: 'text-slate-500',
  error: 'text-red-500',
  unhealthy: 'text-yellow-500',
  notinstalled: 'text-slate-400',
};

export const SERVICE_STATUS_BG_COLORS: Record<ServiceStatus, string> = {
  running: 'bg-green-500',
  stopped: 'bg-slate-500',
  error: 'bg-red-500',
  unhealthy: 'bg-yellow-500',
  notinstalled: 'bg-slate-400',
};

export const SERVICE_STATUS_LABELS: Record<ServiceStatus, string> = {
  running: 'Running',
  stopped: 'Stopped',
  error: 'Error',
  unhealthy: 'Unhealthy',
  notinstalled: 'Not Installed',
};

export const SERVICE_TYPE_ICONS: Record<ServiceType, string> = {
  webserver: '🌐',
  database: '🗄️',
  runtime: '⚙️',
  tool: '🔧',
};

// All services supported by ClickDevPort
export interface SupportedService {
  id: string;
  name: string;
  serviceType: ServiceType;
  defaultPort: number;
  description: string;
}

export const SUPPORTED_SERVICES: SupportedService[] = [
  { id: 'apache', name: 'Apache', serviceType: 'webserver', defaultPort: 80, description: 'Apache HTTP Server' },
  { id: 'nginx', name: 'Nginx', serviceType: 'webserver', defaultPort: 80, description: 'High-performance web server' },
  { id: 'mariadb', name: 'MySQL/MariaDB', serviceType: 'database', defaultPort: 3306, description: 'Relational database' },
  { id: 'postgresql', name: 'PostgreSQL', serviceType: 'database', defaultPort: 5432, description: 'Advanced SQL database' },
  { id: 'redis', name: 'Redis', serviceType: 'database', defaultPort: 6379, description: 'In-memory data store' },
  { id: 'mongodb', name: 'MongoDB', serviceType: 'database', defaultPort: 27017, description: 'NoSQL document database' },
];
