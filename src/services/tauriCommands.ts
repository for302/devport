import { invoke } from "@tauri-apps/api/core";
import type {
  Project,
  CreateProjectInput,
  UpdateProjectInput,
  DetectedProjectInfo,
  PortInfo,
  ProcessInfo,
  HealthStatus,
} from "@/types";

// Project commands
export async function getProjects(): Promise<Project[]> {
  return invoke<Project[]>("get_projects");
}

export async function createProject(input: CreateProjectInput): Promise<Project> {
  return invoke<Project>("create_project", { input });
}

export async function updateProject(input: UpdateProjectInput): Promise<Project> {
  return invoke<Project>("update_project", { input });
}

export async function deleteProject(id: string): Promise<void> {
  return invoke<void>("delete_project", { id });
}

export async function detectProjectType(path: string): Promise<DetectedProjectInfo> {
  return invoke<DetectedProjectInfo>("detect_project_type", { path });
}

// Process commands
export async function startProject(projectId: string): Promise<ProcessInfo> {
  return invoke<ProcessInfo>("start_project", { projectId });
}

export async function stopProject(projectId: string): Promise<void> {
  return invoke<void>("stop_project", { projectId });
}

export async function restartProject(projectId: string): Promise<ProcessInfo> {
  return invoke<ProcessInfo>("restart_project", { projectId });
}

export async function getRunningProcesses(): Promise<ProcessInfo[]> {
  return invoke<ProcessInfo[]>("get_running_processes");
}

// Port commands
export async function scanPorts(): Promise<PortInfo[]> {
  return invoke<PortInfo[]>("scan_ports");
}

export async function checkPortAvailable(port: number): Promise<boolean> {
  return invoke<boolean>("check_port_available", { port });
}

// Health commands
export async function checkHealth(projectId: string, url: string): Promise<HealthStatus> {
  return invoke<HealthStatus>("check_health", { projectId, url });
}

// Apache port/vhost entry type
export interface ApachePortEntry {
  port: number;
  domain: string;
  url: string;
  documentRoot: string;
  isSsl: boolean;
  serverAlias: string[];
  configFile: string;
}

// Config commands
export async function getApachePorts(): Promise<ApachePortEntry[]> {
  return invoke<ApachePortEntry[]>("get_apache_ports");
}
