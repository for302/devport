import { useEffect } from "react";
import { Sidebar } from "./Sidebar";
import { ToastContainer } from "./ToastContainer";
import { Dashboard } from "../views/Dashboard";
import { ServicesView } from "../views/ServicesView";
import { PortsView } from "../views/PortsView";
import { LogsView } from "../views/LogsView";
import { SettingsView } from "../views/SettingsView";
import { InstallerView } from "../views/InstallerView";
import { AddProjectModal } from "../modals/AddProjectModal";
import { PortConflictModal } from "../modals/PortConflictModal";
import { ConfirmDeleteModal } from "../modals/ConfirmDeleteModal";
import { EnvEditorModal } from "../modals/EnvEditorModal";
import { ConfigEditorModal } from "../modals/ConfigEditorModal";
import { UpdateModal } from "../modals/UpdateModal";
import { useUiStore, useProjectStore, usePortStore, useServiceStore } from "@/stores";
import { useTauriEvents } from "@/hooks";

export function AppShell() {
  const activeView = useUiStore((state) => state.activeView);
  const activeModal = useUiStore((state) => state.activeModal);
  const fetchProjects = useProjectStore((state) => state.fetchProjects);
  const scanPorts = usePortStore((state) => state.scanPorts);
  const fetchServices = useServiceStore((state) => state.fetchServices);

  // Initialize Tauri event listeners
  useTauriEvents();

  // Fetch initial data
  useEffect(() => {
    fetchProjects();
    fetchServices();

    // Delay initial port scan to avoid blocking startup
    const initialScan = setTimeout(() => {
      scanPorts();
    }, 2000);

    // Set up port scanning interval (every 30 seconds)
    const interval = setInterval(() => {
      scanPorts();
    }, 30000);

    return () => {
      clearTimeout(initialScan);
      clearInterval(interval);
    };
  }, [fetchProjects, scanPorts, fetchServices]);

  // Initialize project event listeners separately (only once)
  useEffect(() => {
    let unlistenProjectEvents: (() => void) | undefined;

    useProjectStore.getState().initEventListeners().then((unlisten) => {
      unlistenProjectEvents = unlisten;
    });

    return () => {
      unlistenProjectEvents?.();
    };
  }, []); // Run only on mount

  const renderView = () => {
    switch (activeView) {
      case "dashboard":
        return <Dashboard />;
      case "services":
        return <ServicesView />;
      case "ports":
        return <PortsView />;
      case "logs":
        return <LogsView />;
      case "settings":
        return <SettingsView />;
      case "installer":
        return <InstallerView />;
      default:
        return <Dashboard />;
    }
  };

  return (
    <div className="flex h-screen bg-slate-900 text-slate-200">
      <Sidebar />
      <main className="flex-1 overflow-auto">
        {renderView()}
      </main>

      {/* Modals */}
      {activeModal === "addProject" && <AddProjectModal />}
      {activeModal === "editProject" && <AddProjectModal isEdit />}
      {activeModal === "portConflict" && <PortConflictModal />}
      {activeModal === "confirmDelete" && <ConfirmDeleteModal />}
      {activeModal === "envEditor" && <EnvEditorModal />}
      {activeModal === "configEditor" && <ConfigEditorModal />}
      {activeModal === "update" && <UpdateModal />}

      {/* Toast Notifications */}
      <ToastContainer />
    </div>
  );
}
