import { create } from "zustand";
import { immer } from "zustand/middleware/immer";

type ModalType = "addProject" | "editProject" | "portConflict" | "confirmDelete" | "envEditor" | "serviceSettings" | "configEditor" | null;
type ViewType = "dashboard" | "services" | "ports" | "logs" | "settings";

export type NotificationType = "info" | "success" | "warning" | "error";

export interface Notification {
  id: string;
  type: NotificationType;
  title: string;
  message?: string;
  duration?: number;
}

interface UiState {
  activeModal: ModalType;
  modalData: Record<string, unknown>;
  activeView: ViewType;
  sidebarCollapsed: boolean;
  selectedLogProjectId: string | null;
  notifications: Notification[];
}

interface UiActions {
  openModal: (modal: ModalType, data?: Record<string, unknown>) => void;
  closeModal: () => void;
  setActiveView: (view: ViewType) => void;
  toggleSidebar: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  setSelectedLogProjectId: (id: string | null) => void;
  addNotification: (notification: Omit<Notification, "id">) => void;
  removeNotification: (id: string) => void;
}

type UiStore = UiState & UiActions;

export const useUiStore = create<UiStore>()(
  immer((set) => ({
    activeModal: null,
    modalData: {},
    activeView: "dashboard",
    sidebarCollapsed: false,
    selectedLogProjectId: null,
    notifications: [],

    openModal: (modal: ModalType, data: Record<string, unknown> = {}) => {
      set((state) => {
        state.activeModal = modal;
        state.modalData = data;
      });
    },

    closeModal: () => {
      set((state) => {
        state.activeModal = null;
        state.modalData = {};
      });
    },

    setActiveView: (view: ViewType) => {
      set((state) => {
        state.activeView = view;
      });
    },

    toggleSidebar: () => {
      set((state) => {
        state.sidebarCollapsed = !state.sidebarCollapsed;
      });
    },

    setSidebarCollapsed: (collapsed: boolean) => {
      set((state) => {
        state.sidebarCollapsed = collapsed;
      });
    },

    setSelectedLogProjectId: (id: string | null) => {
      set((state) => {
        state.selectedLogProjectId = id;
      });
    },

    addNotification: (notification: Omit<Notification, "id">) => {
      const id = `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
      set((state) => {
        state.notifications.push({ ...notification, id });
      });
      // Auto-remove after duration (default 5 seconds)
      const duration = notification.duration ?? 5000;
      if (duration > 0) {
        setTimeout(() => {
          set((state) => {
            state.notifications = state.notifications.filter((n) => n.id !== id);
          });
        }, duration);
      }
    },

    removeNotification: (id: string) => {
      set((state) => {
        state.notifications = state.notifications.filter((n) => n.id !== id);
      });
    },
  }))
);
