/**
 * Color mappings for project types
 * Used in ProjectCard, ProjectListItem, and other components displaying project badges
 */
export const PROJECT_TYPE_COLORS: Record<string, string> = {
  // Desktop frameworks
  tauri: "bg-yellow-500 text-black",
  electron: "bg-blue-400 text-white",

  // Node.js frameworks
  nextjs: "bg-black text-white",
  vite: "bg-purple-500 text-white",
  react: "bg-cyan-500 text-white",
  vue: "bg-emerald-500 text-white",
  angular: "bg-red-500 text-white",
  svelte: "bg-orange-500 text-white",
  node: "bg-green-500 text-white",
  express: "bg-gray-700 text-white",

  // PHP frameworks
  laravel: "bg-red-600 text-white",
  codeigniter: "bg-orange-600 text-white",

  // Python web frameworks
  django: "bg-green-700 text-white",
  flask: "bg-gray-600 text-white",
  fastapi: "bg-teal-500 text-white",

  // Python general and desktop
  python: "bg-yellow-600 text-black",
  pythontkinter: "bg-yellow-600 text-black",
  pythonpyqt: "bg-yellow-600 text-black",
  pythonwx: "bg-yellow-600 text-black",
  pythonpygame: "bg-yellow-600 text-black",
  pythonkivy: "bg-yellow-600 text-black",

  // Fallback
  unknown: "bg-slate-600 text-white",
};

/**
 * Get the color class for a project type
 * @param projectType - The project type string
 * @returns Tailwind CSS classes for the badge
 */
export function getProjectTypeColor(projectType: string): string {
  return PROJECT_TYPE_COLORS[projectType.toLowerCase()] || PROJECT_TYPE_COLORS.unknown;
}
