import { usePortStore, useProjectStore } from "@/stores";
import { PortRow } from "./PortRow";
import { Network } from "lucide-react";

export function PortTable() {
  const ports = usePortStore((state) => state.ports);
  const projects = useProjectStore((state) => state.projects);

  // Create a map of port -> project
  const portToProject = new Map(
    projects.map((p) => [p.port, p])
  );

  // Enrich port info with project data
  const enrichedPorts = ports.map((port) => ({
    ...port,
    project: portToProject.get(port.port),
  }));

  if (ports.length === 0) {
    return (
      <div className="text-center py-12 bg-slate-800 rounded-lg border border-slate-700">
        <Network size={48} className="mx-auto text-slate-600 mb-4" />
        <h3 className="text-lg font-medium text-slate-400 mb-2">No ports detected</h3>
        <p className="text-sm text-slate-500">
          Start a project to see active ports
        </p>
      </div>
    );
  }

  return (
    <div className="bg-slate-800 rounded-lg border border-slate-700 overflow-hidden">
      <table className="w-full">
        <thead>
          <tr className="bg-slate-900/50 text-left">
            <th className="px-4 py-3 text-sm font-medium text-slate-400">Port</th>
            <th className="px-4 py-3 text-sm font-medium text-slate-400">Service</th>
            <th className="px-4 py-3 text-sm font-medium text-slate-400">Protocol</th>
            <th className="px-4 py-3 text-sm font-medium text-slate-400">State</th>
            <th className="px-4 py-3 text-sm font-medium text-slate-400">Process</th>
            <th className="px-4 py-3 text-sm font-medium text-slate-400">PID</th>
            <th className="px-4 py-3 text-sm font-medium text-slate-400">Project</th>
            <th className="px-4 py-3 text-sm font-medium text-slate-400">Actions</th>
          </tr>
        </thead>
        <tbody>
          {enrichedPorts.map((port) => (
            <PortRow key={`${port.port}-${port.protocol}`} port={port} />
          ))}
        </tbody>
      </table>
    </div>
  );
}
