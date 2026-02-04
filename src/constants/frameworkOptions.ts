import type { ProjectType } from "@/types";

/**
 * Framework option for the project creation form
 */
export interface FrameworkOption {
  value: ProjectType;
  label: string;
  category: string;
}

/**
 * Available framework options for project creation
 * Organized by category for easy display
 */
export const FRAMEWORK_OPTIONS: FrameworkOption[] = [
  // Desktop
  { value: "tauri", label: "Tauri", category: "Desktop" },
  { value: "electron", label: "Electron", category: "Desktop" },

  // Node.js
  { value: "nextjs", label: "Next.js", category: "Node.js" },
  { value: "vite", label: "Vite (React)", category: "Node.js" },
  { value: "react", label: "React", category: "Node.js" },
  { value: "vue", label: "Vue (Vite)", category: "Node.js" },
  { value: "svelte", label: "Svelte", category: "Node.js" },
  { value: "angular", label: "Angular", category: "Node.js" },
  { value: "express", label: "Express.js", category: "Node.js" },
  { value: "node", label: "Node.js", category: "Node.js" },

  // PHP
  { value: "laravel", label: "Laravel", category: "PHP" },
  { value: "codeigniter", label: "CodeIgniter", category: "PHP" },

  // Python Web
  { value: "django", label: "Django", category: "Python Web" },
  { value: "flask", label: "Flask", category: "Python Web" },
  { value: "fastapi", label: "FastAPI", category: "Python Web" },

  // Python
  { value: "python", label: "Python", category: "Python" },

  // Python Desktop
  { value: "pythontkinter", label: "Tkinter", category: "Python Desktop" },
  { value: "pythonpyqt", label: "PyQt", category: "Python Desktop" },
  { value: "pythonwx", label: "wxPython", category: "Python Desktop" },
  { value: "pythonpygame", label: "Pygame", category: "Python Desktop" },
  { value: "pythonkivy", label: "Kivy", category: "Python Desktop" },

  // Other
  { value: "unknown", label: "Other", category: "Other" },
];

/**
 * Default ports for each framework
 * 0 means no port (desktop apps)
 */
export const DEFAULT_PORTS: Record<string, number> = {
  // Desktop (no port needed)
  tauri: 1420,
  electron: 3000,

  // Node.js
  nextjs: 3000,
  react: 3000,
  vite: 5173,
  vue: 5173,
  svelte: 5173,
  angular: 4200,
  express: 3000,
  node: 3000,

  // PHP
  laravel: 8000,
  codeigniter: 8080,

  // Python
  django: 8000,
  flask: 5000,
  fastapi: 8000,
  python: 8000,

  // Python Desktop (no port needed)
  pythontkinter: 0,
  pythonpyqt: 0,
  pythonwx: 0,
  pythonpygame: 0,
  pythonkivy: 0,

  // Fallback
  unknown: 3000,
};

/**
 * Default start commands for each framework
 */
export const DEFAULT_COMMANDS: Record<string, string> = {
  // Desktop
  tauri: "npm run tauri dev",
  electron: "npm run dev",

  // Node.js
  nextjs: "npm run dev",
  react: "npm start",
  vite: "npm run dev",
  vue: "npm run dev",
  svelte: "npm run dev",
  angular: "npm start",
  express: "npm start",
  node: "npm start",

  // PHP
  laravel: "php artisan serve",
  codeigniter: "php spark serve",

  // Python
  django: "python manage.py runserver",
  flask: "flask run",
  fastapi: "uvicorn main:app --reload",
  python: "python main.py",

  // Python Desktop
  pythontkinter: "python main.py",
  pythonpyqt: "python main.py",
  pythonwx: "python main.py",
  pythonpygame: "python main.py",
  pythonkivy: "python main.py",

  // Fallback
  unknown: "npm start",
};

/**
 * Project types that don't require a port (desktop apps)
 */
export const NO_PORT_TYPES: ProjectType[] = [
  "pythontkinter",
  "pythonpyqt",
  "pythonwx",
  "pythonpygame",
  "pythonkivy",
];

/**
 * Get the default port for a framework
 */
export function getDefaultPort(projectType: string): number {
  return DEFAULT_PORTS[projectType] ?? DEFAULT_PORTS.unknown;
}

/**
 * Get the default start command for a framework
 */
export function getDefaultCommand(projectType: string): string {
  return DEFAULT_COMMANDS[projectType] ?? DEFAULT_COMMANDS.unknown;
}

/**
 * Check if a project type requires a port
 */
export function requiresPort(projectType: ProjectType): boolean {
  return !NO_PORT_TYPES.includes(projectType);
}

/**
 * Get framework options grouped by category
 */
export function getFrameworksByCategory(): Map<string, FrameworkOption[]> {
  const grouped = new Map<string, FrameworkOption[]>();

  for (const option of FRAMEWORK_OPTIONS) {
    const existing = grouped.get(option.category) || [];
    existing.push(option);
    grouped.set(option.category, existing);
  }

  return grouped;
}
