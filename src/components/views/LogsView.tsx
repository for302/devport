import { useProjectStore, useUiStore, useServiceStore } from "@/stores";
import { LogViewer } from "../logs/LogViewer";
import { ScrollText, Server } from "lucide-react";

export function LogsView() {
  const projects = useProjectStore((state) => state.projects);
  const services = useServiceStore((state) => state.services);
  const selectedLogProjectId = useUiStore((state) => state.selectedLogProjectId);
  const setSelectedLogProjectId = useUiStore((state) => state.setSelectedLogProjectId);

  return (
    <div className="flex h-full">
      {/* Project/Service List */}
      <div className="w-64 border-r border-slate-800 bg-slate-900 flex flex-col">
        {/* Services Section */}
        <div className="border-b border-slate-800">
          <div className="p-4 border-b border-slate-700">
            <h2 className="text-sm font-semibold text-slate-400 uppercase tracking-wider">Services</h2>
          </div>
          <div className="p-2">
            {services.length === 0 ? (
              <p className="text-sm text-slate-500 p-2">No services</p>
            ) : (
              services.map((service) => (
                <button
                  key={service.id}
                  onClick={() => setSelectedLogProjectId(`service:${service.id}`)}
                  className={`
                    w-full text-left px-3 py-2 rounded-lg transition-colors flex items-center gap-2
                    ${selectedLogProjectId === `service:${service.id}`
                      ? "bg-blue-600 text-white"
                      : "text-slate-400 hover:bg-slate-800 hover:text-slate-200"
                    }
                  `}
                >
                  <Server size={16} />
                  <div>
                    <p className="font-medium truncate">{service.name}</p>
                    <p className="text-xs opacity-60">Port {service.port}</p>
                  </div>
                </button>
              ))
            )}
          </div>
        </div>

        {/* Projects Section */}
        <div className="flex-1 overflow-auto">
          <div className="p-4 border-b border-slate-700">
            <h2 className="text-sm font-semibold text-slate-400 uppercase tracking-wider">Projects</h2>
          </div>
          <div className="p-2">
            {projects.length === 0 ? (
              <p className="text-sm text-slate-500 p-2">No projects</p>
            ) : (
              projects.map((project) => (
                <button
                  key={project.id}
                  onClick={() => setSelectedLogProjectId(project.id)}
                  className={`
                    w-full text-left px-3 py-2 rounded-lg transition-colors
                    ${selectedLogProjectId === project.id
                      ? "bg-blue-600 text-white"
                      : "text-slate-400 hover:bg-slate-800 hover:text-slate-200"
                    }
                  `}
                >
                  <p className="font-medium truncate">{project.name}</p>
                  <p className="text-xs opacity-60">Port {project.port}</p>
                </button>
              ))
            )}
          </div>
        </div>
      </div>

      {/* Log Viewer */}
      <div className="flex-1 flex flex-col">
        {selectedLogProjectId ? (
          <LogViewer projectId={selectedLogProjectId} />
        ) : (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <ScrollText size={48} className="mx-auto text-slate-600 mb-4" />
              <h3 className="text-lg font-medium text-slate-400 mb-2">
                Select a project or service
              </h3>
              <p className="text-sm text-slate-500">
                Choose from the list to view logs
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
