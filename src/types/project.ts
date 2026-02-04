export type ProjectType =
  | "tauri"     // Desktop app with Tauri (highest priority)
  | "electron"  // Desktop app with Electron
  | "nextjs"
  | "vite"
  | "react"
  | "vue"
  | "angular"
  | "svelte"
  | "python"
  | "pythontkinter"
  | "pythonpyqt"
  | "pythonwx"
  | "pythonpygame"
  | "pythonkivy"
  | "django"
  | "flask"
  | "fastapi"
  | "laravel"
  | "codeigniter"
  | "node"
  | "express"
  | "unknown";

export interface Project {
  id: string;
  name: string;
  path: string;
  port: number;
  projectType: ProjectType;
  startCommand: string;
  envVars: Record<string, string>;
  autoStart: boolean;
  healthCheckUrl: string | null;
  domain: string | null;  // Custom domain for hosts file (e.g., "my-app.test")
  githubUrl: string | null;  // GitHub repository URL
  createdAt: string;
  updatedAt: string;
}

export interface CreateProjectInput {
  name: string;
  path: string;
  port: number;
  projectType: ProjectType;
  startCommand: string;
  autoStart?: boolean;
  healthCheckUrl?: string | null;
  domain?: string | null;  // Custom domain for hosts file
  githubUrl?: string | null;  // GitHub repository URL
}

export interface UpdateProjectInput {
  id: string;
  name?: string;
  port?: number;
  startCommand?: string;
  autoStart?: boolean;
  healthCheckUrl?: string | null;
}

export interface DetectedProjectInfo {
  projectType: ProjectType;
  name: string;
  startCommand: string;
  defaultPort: number;
  venvPath: string | null;
  githubUrl: string | null;
}
