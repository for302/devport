import {
  LayoutDashboard,
  Network,
  ScrollText,
  Plus,
  ChevronLeft,
  ChevronRight,
  Server,
  Settings,
  Package,
} from "lucide-react";
import { useUiStore, useProjectStore, useProcessStore, useServiceStore } from "@/stores";

interface NavItemProps {
  icon: React.ReactNode;
  label: string;
  active?: boolean;
  collapsed?: boolean;
  onClick?: () => void;
  badge?: number;
}

function NavItem({ icon, label, active, collapsed, onClick, badge }: NavItemProps) {
  return (
    <button
      onClick={onClick}
      className={`
        flex items-center gap-3 w-full px-3 py-2 rounded-lg transition-colors
        ${active
          ? "bg-blue-600 text-white"
          : "text-slate-400 hover:bg-slate-800 hover:text-slate-200"
        }
        ${collapsed ? "justify-center" : ""}
      `}
    >
      {icon}
      {!collapsed && (
        <>
          <span className="flex-1 text-left">{label}</span>
          {badge !== undefined && badge > 0 && (
            <span className="bg-green-500 text-white text-xs px-2 py-0.5 rounded-full">
              {badge}
            </span>
          )}
        </>
      )}
    </button>
  );
}

export function Sidebar() {
  const activeView = useUiStore((state) => state.activeView);
  const setActiveView = useUiStore((state) => state.setActiveView);
  const sidebarCollapsed = useUiStore((state) => state.sidebarCollapsed);
  const toggleSidebar = useUiStore((state) => state.toggleSidebar);
  const openModal = useUiStore((state) => state.openModal);
  const projects = useProjectStore((state) => state.projects);
  const processes = useProcessStore((state) => state.processes);

  const runningCount = Object.keys(processes).length;
  const services = useServiceStore((state) => state.services);
  const runningServicesCount = services.filter((s) => s.status === "running").length;

  return (
    <aside
      className={`
        flex flex-col bg-slate-950 border-r border-slate-800 transition-all duration-300
        ${sidebarCollapsed ? "w-16" : "w-64"}
      `}
    >
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-slate-800">
        {!sidebarCollapsed && (
          <img src="/logo.png" alt="ClickDevPort" className="h-8" />
        )}
        <button
          onClick={toggleSidebar}
          className="p-1 rounded hover:bg-slate-800 text-slate-400 hover:text-slate-200"
        >
          {sidebarCollapsed ? <ChevronRight size={20} /> : <ChevronLeft size={20} />}
        </button>
      </div>

      {/* Navigation */}
      <nav className="flex-1 p-2 space-y-1">
        <NavItem
          icon={<LayoutDashboard size={20} />}
          label="Dashboard"
          active={activeView === "dashboard"}
          collapsed={sidebarCollapsed}
          onClick={() => setActiveView("dashboard")}
          badge={runningCount}
        />
        <NavItem
          icon={<Server size={20} />}
          label="Services"
          active={activeView === "services"}
          collapsed={sidebarCollapsed}
          onClick={() => setActiveView("services")}
          badge={runningServicesCount}
        />
        <NavItem
          icon={<Network size={20} />}
          label="Ports"
          active={activeView === "ports"}
          collapsed={sidebarCollapsed}
          onClick={() => setActiveView("ports")}
        />
        <NavItem
          icon={<ScrollText size={20} />}
          label="Logs"
          active={activeView === "logs"}
          collapsed={sidebarCollapsed}
          onClick={() => setActiveView("logs")}
        />
        <NavItem
          icon={<Settings size={20} />}
          label="Settings"
          active={activeView === "settings"}
          collapsed={sidebarCollapsed}
          onClick={() => setActiveView("settings")}
        />
        <NavItem
          icon={<Package size={20} />}
          label="Installer"
          active={activeView === "installer"}
          collapsed={sidebarCollapsed}
          onClick={() => setActiveView("installer")}
        />
      </nav>

      {/* Projects count */}
      {!sidebarCollapsed && (
        <div className="px-4 py-2 border-t border-slate-800">
          <p className="text-xs text-slate-500 uppercase tracking-wider">Projects</p>
          <p className="text-2xl font-bold text-white">{projects.length}</p>
        </div>
      )}

      {/* Add Project Button */}
      <div className="p-2 border-t border-slate-800">
        <button
          onClick={() => openModal("addProject")}
          className={`
            flex items-center justify-center gap-2 w-full py-2 rounded-lg
            bg-blue-600 hover:bg-blue-700 text-white font-medium transition-colors
          `}
        >
          <Plus size={20} />
          {!sidebarCollapsed && <span>Add Project</span>}
        </button>
      </div>
    </aside>
  );
}
