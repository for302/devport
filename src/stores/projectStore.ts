import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import { listen } from "@tauri-apps/api/event";
import type { Project, CreateProjectInput, UpdateProjectInput, ProjectType } from "@/types";
import * as tauriCommands from "@/services/tauriCommands";
import { useUiStore } from "./uiStore";

// Payload for project type changed event
interface ProjectTypeChangedPayload {
  projectId: string;
  projectName: string;
  oldType: string;
  newType: string;
  newCommand: string;
}

interface ProjectState {
  projects: Project[];
  selectedProjectId: string | null;
  isLoading: boolean;
  error: string | null;
}

interface ProjectActions {
  fetchProjects: () => Promise<void>;
  createProject: (input: CreateProjectInput) => Promise<Project>;
  updateProject: (input: UpdateProjectInput) => Promise<Project>;
  deleteProject: (id: string) => Promise<void>;
  selectProject: (id: string | null) => void;
  getProjectById: (id: string) => Project | undefined;
  handleProjectTypeChanged: (payload: ProjectTypeChangedPayload) => void;
  initEventListeners: () => Promise<() => void>;
}

type ProjectStore = ProjectState & ProjectActions;

export const useProjectStore = create<ProjectStore>()(
  immer((set, get) => ({
    projects: [],
    selectedProjectId: null,
    isLoading: false,
    error: null,

    fetchProjects: async () => {
      set((state) => {
        state.isLoading = true;
        state.error = null;
      });

      try {
        const projects = await tauriCommands.getProjects();
        set((state) => {
          state.projects = projects;
          state.isLoading = false;
        });
      } catch (error) {
        set((state) => {
          state.error = error instanceof Error ? error.message : "Failed to fetch projects";
          state.isLoading = false;
        });
      }
    },

    createProject: async (input: CreateProjectInput) => {
      set((state) => {
        state.isLoading = true;
        state.error = null;
      });

      try {
        const project = await tauriCommands.createProject(input);
        set((state) => {
          state.projects.push(project);
          state.isLoading = false;
        });
        return project;
      } catch (error) {
        set((state) => {
          state.error = error instanceof Error ? error.message : "Failed to create project";
          state.isLoading = false;
        });
        throw error;
      }
    },

    updateProject: async (input: UpdateProjectInput) => {
      set((state) => {
        state.isLoading = true;
        state.error = null;
      });

      try {
        const project = await tauriCommands.updateProject(input);
        set((state) => {
          const index = state.projects.findIndex((p) => p.id === project.id);
          if (index !== -1) {
            state.projects[index] = project;
          }
          state.isLoading = false;
        });
        return project;
      } catch (error) {
        set((state) => {
          state.error = error instanceof Error ? error.message : "Failed to update project";
          state.isLoading = false;
        });
        throw error;
      }
    },

    deleteProject: async (id: string) => {
      set((state) => {
        state.isLoading = true;
        state.error = null;
      });

      try {
        await tauriCommands.deleteProject(id);
        set((state) => {
          state.projects = state.projects.filter((p) => p.id !== id);
          if (state.selectedProjectId === id) {
            state.selectedProjectId = null;
          }
          state.isLoading = false;
        });
      } catch (error) {
        set((state) => {
          state.error = error instanceof Error ? error.message : "Failed to delete project";
          state.isLoading = false;
        });
        throw error;
      }
    },

    selectProject: (id: string | null) => {
      set((state) => {
        state.selectedProjectId = id;
      });
    },

    getProjectById: (id: string) => {
      return get().projects.find((p) => p.id === id);
    },

    handleProjectTypeChanged: (payload: ProjectTypeChangedPayload) => {
      set((state) => {
        const index = state.projects.findIndex((p) => p.id === payload.projectId);
        if (index !== -1) {
          state.projects[index].projectType = payload.newType as ProjectType;
          state.projects[index].startCommand = payload.newCommand;
        }
      });

      // Show notification
      useUiStore.getState().addNotification({
        type: "info",
        title: "Project Type Changed",
        message: `${payload.projectName}: ${payload.oldType.toUpperCase()} -> ${payload.newType.toUpperCase()}`,
        duration: 5000,
      });
    },

    initEventListeners: async () => {
      const unlisten = await listen<ProjectTypeChangedPayload>(
        "project-type-changed",
        (event) => {
          get().handleProjectTypeChanged(event.payload);
        }
      );
      return unlisten;
    },
  }))
);
