export type ProcessStatus =
  | "running"
  | "stopped"
  | "starting"
  | "stopping"
  | "error";

export interface ProcessInfo {
  projectId: string;
  pid: number;
  status: ProcessStatus;
  startedAt: string | null;
  port: number;
  cpuUsage: number | null;
  memoryUsage: number | null;
}

export interface ProcessLog {
  projectId: string;
  line: string;
  stream: "stdout" | "stderr";
  timestamp: string;
}

export interface HealthStatus {
  projectId: string;
  isHealthy: boolean;
  statusCode: number | null;
  responseTimeMs: number | null;
  error: string | null;
  checkedAt: string;
}
