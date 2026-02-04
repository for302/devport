import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Project } from "@/types";

/**
 * Return type for the useProjectActions hook
 */
export interface UseProjectActionsReturn {
  /** Open project in VS Code */
  openInVscode: () => Promise<void>;
  /** Open terminal at project path */
  openInTerminal: () => Promise<void>;
  /** Open project URL in browser */
  openInBrowser: () => Promise<void>;
  /** Open project folder in file explorer */
  openInExplorer: () => Promise<void>;
  /** Open GitHub repository in browser */
  openGitHub: () => Promise<void>;
}

/**
 * Hook for project quick actions (open in VS Code, terminal, browser, etc.)
 *
 * Extracts duplicated action handlers from ProjectCard and ProjectListItem
 * to reduce code duplication and ensure consistent behavior.
 *
 * @example
 * ```tsx
 * const { openInVscode, openInBrowser } = useProjectActions(project);
 *
 * return (
 *   <>
 *     <button onClick={openInVscode}>Open in VS Code</button>
 *     <button onClick={openInBrowser}>Open in Browser</button>
 *   </>
 * );
 * ```
 */
export function useProjectActions(project: Project): UseProjectActionsReturn {
  const openInVscode = useCallback(async () => {
    try {
      await invoke("open_in_vscode", { path: project.path });
    } catch (error) {
      console.error("Failed to open VS Code:", error);
    }
  }, [project.path]);

  const openInTerminal = useCallback(async () => {
    try {
      await invoke("open_in_terminal", { path: project.path });
    } catch (error) {
      console.error("Failed to open terminal:", error);
    }
  }, [project.path]);

  const openInBrowser = useCallback(async () => {
    try {
      await invoke("open_in_browser", {
        url: `http://localhost:${project.port}`,
      });
    } catch (error) {
      console.error("Failed to open browser:", error);
    }
  }, [project.port]);

  const openInExplorer = useCallback(async () => {
    try {
      await invoke("open_file_explorer", { path: project.path });
    } catch (error) {
      console.error("Failed to open file explorer:", error);
    }
  }, [project.path]);

  const openGitHub = useCallback(async () => {
    if (!project.githubUrl) return;
    try {
      await invoke("open_in_browser", { url: project.githubUrl });
    } catch (error) {
      console.error("Failed to open GitHub:", error);
    }
  }, [project.githubUrl]);

  return {
    openInVscode,
    openInTerminal,
    openInBrowser,
    openInExplorer,
    openGitHub,
  };
}

// Note: Project lifecycle hooks (start/stop/restart) are handled directly
// in the components using useProcessStore and useActivityLogStore
// to avoid circular dependency issues with require().
