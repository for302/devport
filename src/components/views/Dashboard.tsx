import { useState, useEffect, useCallback } from "react";
import { useProjectStore, useProcessStore, useUiStore } from "@/stores";
import { ProjectListItem } from "../projects/ProjectListItem";
import { ServiceActivityLog } from "../services/ServiceActivityLog";
import { ApachePortsManager } from "../apache/ApachePortsManager";
import { Activity, Server, Zap, RefreshCw, Plus } from "lucide-react";

const AUTO_REFRESH_INTERVAL = 5 * 60 * 1000; // 5 minutes

export function Dashboard() {
  const projects = useProjectStore((state) => state.projects);
  const fetchProjects = useProjectStore((state) => state.fetchProjects);
  const processes = useProcessStore((state) => state.processes);
  const openModal = useUiStore((state) => state.openModal);
  const [isLogExpanded, setIsLogExpanded] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);

  const handleRefresh = useCallback(async () => {
    setIsRefreshing(true);
    await fetchProjects();
    setIsRefreshing(false);
  }, [fetchProjects]);

  // Auto-refresh every 5 minutes
  useEffect(() => {
    const interval = setInterval(() => {
      fetchProjects();
    }, AUTO_REFRESH_INTERVAL);

    return () => clearInterval(interval);
  }, [fetchProjects]);

  const runningCount = Object.keys(processes).length;
  const stoppedCount = projects.length - runningCount;

  return (
    <div className="flex flex-col h-full">
      {/* Scrollable content area */}
      <div className="flex-1 overflow-auto p-6">
      {/* Stats */}
      <div className="grid grid-cols-3 gap-4 mb-6">
        <div className="bg-slate-800 rounded-lg p-4 border border-slate-700">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-blue-500/20 rounded-lg">
              <Server size={24} className="text-blue-400" />
            </div>
            <div>
              <p className="text-sm text-slate-400">Total Projects</p>
              <p className="text-2xl font-bold text-white">{projects.length}</p>
            </div>
          </div>
        </div>
        <div className="bg-slate-800 rounded-lg p-4 border border-slate-700">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-green-500/20 rounded-lg">
              <Activity size={24} className="text-green-400" />
            </div>
            <div>
              <p className="text-sm text-slate-400">Running</p>
              <p className="text-2xl font-bold text-green-400">{runningCount}</p>
            </div>
          </div>
        </div>
        <div className="bg-slate-800 rounded-lg p-4 border border-slate-700">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-slate-500/20 rounded-lg">
              <Zap size={24} className="text-slate-400" />
            </div>
            <div>
              <p className="text-sm text-slate-400">Stopped</p>
              <p className="text-2xl font-bold text-slate-400">{stoppedCount}</p>
            </div>
          </div>
        </div>
      </div>

      {/* Projects List */}
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold text-white">Projects</h2>
        <div className="flex items-center gap-2">
          <button
            onClick={handleRefresh}
            disabled={isRefreshing}
            className="p-1.5 text-slate-400 hover:text-blue-400 hover:bg-slate-700 rounded transition-colors disabled:opacity-50"
            title="Refresh projects"
          >
            <RefreshCw size={16} className={isRefreshing ? "animate-spin" : ""} />
          </button>
          <button
            onClick={() => openModal("addProject")}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors"
          >
            <Plus size={16} />
            Add Project
          </button>
        </div>
      </div>
      {projects.length === 0 ? (
        <div className="text-center py-12 bg-slate-800 rounded-lg border border-slate-700">
          <Server size={48} className="mx-auto text-slate-600 mb-4" />
          <h3 className="text-lg font-medium text-slate-400 mb-2">No projects yet</h3>
          <p className="text-sm text-slate-500 mb-4">
            Add your first project to get started
          </p>
          <button
            onClick={() => openModal("addProject")}
            className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors"
          >
            <Plus size={18} />
            Add Project
          </button>
        </div>
      ) : (
        <div className="bg-slate-800 rounded-lg border border-slate-700 overflow-hidden">
          {/* Header */}
          <div className="flex items-center gap-4 px-4 py-2 bg-slate-900 border-b border-slate-700 text-xs text-slate-400 uppercase tracking-wider">
            <div className="w-2 flex-shrink-0" />
            <div className="w-16 flex-shrink-0">Type</div>
            <div className="w-32 min-w-[8rem] flex-shrink-0">Name</div>
            <div className="hidden lg:flex flex-1 min-w-0">Path</div>
            <div className="w-16 text-center flex-shrink-0">Port</div>
            <div className="w-32 text-center flex-shrink-0">Quick Actions</div>
            <div className="w-px h-4 flex-shrink-0" />
            <div className="w-16 text-center flex-shrink-0">Control</div>
            <div className="w-px h-4 flex-shrink-0" />
            <div className="w-16 text-center flex-shrink-0">Edit</div>
          </div>
          {/* List Items */}
          <div>
            {projects.map((project) => (
              <ProjectListItem key={project.id} project={project} />
            ))}
          </div>
        </div>
      )}

      {/* Apache VirtualHosts Section */}
      <ApachePortsManager compact />
      </div>

      {/* Fixed Activity Log panel at bottom */}
      <div className="flex-shrink-0 p-4 pt-0">
        <ServiceActivityLog
          isExpanded={isLogExpanded}
          onToggleExpand={() => setIsLogExpanded(!isLogExpanded)}
        />
      </div>
    </div>
  );
}
