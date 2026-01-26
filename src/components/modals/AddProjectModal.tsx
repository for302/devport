import { useState, useEffect } from "react";
import { X, FolderOpen, Loader2, Wand2, Globe, Database, Package } from "lucide-react";
import { useUiStore, useProjectStore } from "@/stores";
import { detectProjectType } from "@/services/tauriCommands";
import type { ProjectType, CreateProjectInput } from "@/types";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";

interface AddProjectModalProps {
  isEdit?: boolean;
}

type PackageManager = "npm" | "pnpm" | "yarn";

const FRAMEWORK_OPTIONS = [
  { value: "tauri", label: "Tauri", category: "Desktop" },
  { value: "electron", label: "Electron", category: "Desktop" },
  { value: "nextjs", label: "Next.js", category: "Node.js" },
  { value: "vite", label: "Vite (React)", category: "Node.js" },
  { value: "react", label: "React", category: "Node.js" },
  { value: "vue", label: "Vue (Vite)", category: "Node.js" },
  { value: "svelte", label: "Svelte", category: "Node.js" },
  { value: "angular", label: "Angular", category: "Node.js" },
  { value: "express", label: "Express.js", category: "Node.js" },
  { value: "node", label: "Node.js", category: "Node.js" },
  { value: "laravel", label: "Laravel", category: "PHP" },
  { value: "codeigniter", label: "CodeIgniter", category: "PHP" },
  { value: "django", label: "Django", category: "Python" },
  { value: "flask", label: "Flask", category: "Python" },
  { value: "fastapi", label: "FastAPI", category: "Python" },
  { value: "python", label: "Python", category: "Python" },
  { value: "unknown", label: "Other", category: "Other" },
];

const DEFAULT_PORTS: Record<string, number> = {
  tauri: 1420,
  electron: 3000,
  nextjs: 3000,
  react: 3000,
  vite: 5173,
  vue: 5173,
  svelte: 5173,
  angular: 4200,
  express: 3000,
  node: 3000,
  laravel: 8000,
  codeigniter: 8080,
  django: 8000,
  flask: 5000,
  fastapi: 8000,
  python: 8000,
  unknown: 3000,
};

const DEFAULT_COMMANDS: Record<string, string> = {
  tauri: "npm run tauri dev",
  electron: "npm run dev",
  nextjs: "npm run dev",
  react: "npm start",
  vite: "npm run dev",
  vue: "npm run dev",
  svelte: "npm run dev",
  angular: "npm start",
  express: "npm start",
  node: "npm start",
  laravel: "php artisan serve",
  codeigniter: "php spark serve",
  django: "python manage.py runserver",
  flask: "flask run",
  fastapi: "uvicorn main:app --reload",
  python: "python main.py",
  unknown: "npm start",
};

export function AddProjectModal({ isEdit = false }: AddProjectModalProps) {
  const closeModal = useUiStore((state) => state.closeModal);
  const modalData = useUiStore((state) => state.modalData);
  const createProject = useProjectStore((state) => state.createProject);
  const updateProject = useProjectStore((state) => state.updateProject);
  const getProjectById = useProjectStore((state) => state.getProjectById);

  const editProjectId = modalData.projectId as string | undefined;
  const editProject = editProjectId ? getProjectById(editProjectId) : undefined;

  const [mode, setMode] = useState<"existing" | "new">("existing");
  const [name, setName] = useState(editProject?.name || "");
  const [path, setPath] = useState(editProject?.path || "");
  const [port, setPort] = useState(editProject?.port?.toString() || "3000");
  const [projectType, setProjectType] = useState<ProjectType>(
    editProject?.projectType || "unknown"
  );
  const [startCommand, setStartCommand] = useState(editProject?.startCommand || "");
  const [autoStart, setAutoStart] = useState(editProject?.autoStart || false);
  const [healthCheckUrl, setHealthCheckUrl] = useState(editProject?.healthCheckUrl || "");

  const [domain, setDomain] = useState("");
  const [domainEnabled, setDomainEnabled] = useState(false);
  const [packageManager, setPackageManager] = useState<PackageManager>("pnpm");
  const [createDatabase, setCreateDatabase] = useState(false);
  const [databaseName, setDatabaseName] = useState("");

  const [isDetecting, setIsDetecting] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [portAvailable, setPortAvailable] = useState<boolean | null>(null);
  const [domainCheck, setDomainCheck] = useState<{
    available: boolean;
    valid: boolean;
    error: string | null;
  } | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (port) {
      checkPortAvailability(parseInt(port, 10));
    }
  }, [port]);

  useEffect(() => {
    if (domain) {
      checkDomainAvailability(domain);
    } else {
      setDomainCheck(null);
    }
  }, [domain]);

  useEffect(() => {
    if (name && !domain) {
      const suggestedDomain = name
        .toLowerCase()
        .replace(/[^a-z0-9]/g, "-")
        .replace(/-+/g, "-");
      setDomain(`${suggestedDomain}.test`);
      setDatabaseName(name.toLowerCase().replace(/[^a-z0-9]/g, "_"));
    }
  }, [name]);

  const checkPortAvailability = async (portNum: number) => {
    try {
      const available = await invoke<boolean>("check_port_available", { port: portNum });
      setPortAvailable(available);
    } catch {
      setPortAvailable(null);
    }
  };

  const checkDomainAvailability = async (domainName: string) => {
    try {
      const result = await invoke<{
        available: boolean;
        valid: boolean;
        error: string | null;
      }>("check_domain_available", { domain: domainName });
      setDomainCheck(result);
    } catch {
      setDomainCheck(null);
    }
  };

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: mode === "new" ? "Select Parent Folder" : "Select Project Folder",
      });

      if (selected && typeof selected === "string") {
        setPath(selected);
        if (mode === "existing") {
          handleDetect(selected);
        }
      }
    } catch (err) {
      console.error("Failed to select folder:", err);
    }
  };

  const handleDetect = async (projectPath: string) => {
    console.log("handleDetect called with path:", projectPath);
    if (!projectPath) {
      console.log("No path provided, returning");
      return;
    }

    setIsDetecting(true);
    setError(null);

    try {
      console.log("Calling detectProjectType...");
      const detected = await detectProjectType(projectPath);
      console.log("Detection result:", detected);
      setName(detected.name);
      setProjectType(detected.projectType);
      setStartCommand(detected.startCommand);
      setPort(detected.defaultPort.toString());
      console.log("Detection complete, UI updated");
    } catch (err) {
      console.error("Detection failed:", err);
      setError(`Failed to detect project type: ${err}`);
    } finally {
      setIsDetecting(false);
    }
  };

  const handleFrameworkChange = (type: ProjectType) => {
    setProjectType(type);
    setPort(DEFAULT_PORTS[type]?.toString() || "3000");
    setStartCommand(DEFAULT_COMMANDS[type] || "npm start");
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setIsSubmitting(true);

    try {
      if (isEdit && editProjectId) {
        await updateProject({
          id: editProjectId,
          name,
          port: parseInt(port, 10),
          startCommand,
          autoStart,
          healthCheckUrl: healthCheckUrl || null,
        });
      } else {
        const input: CreateProjectInput = {
          name,
          path: mode === "new" ? `${path}\\${name}` : path,
          port: parseInt(port, 10),
          projectType,
          startCommand,
          autoStart,
          healthCheckUrl: healthCheckUrl || null,
          domain: domainEnabled && domain ? domain : null,  // Only use if enabled
        };
        await createProject(input);
      }
      closeModal();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save project");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-slate-800 rounded-lg w-full max-w-2xl mx-4 max-h-[90vh] overflow-y-auto">
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-700">
          <h2 className="text-lg font-semibold text-white">
            {isEdit ? "Edit Project" : "Add Project"}
          </h2>
          <button
            onClick={closeModal}
            className="p-1 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors"
          >
            <X size={20} />
          </button>
        </div>

        {!isEdit && (
          <div className="flex border-b border-slate-700">
            <button
              type="button"
              onClick={() => setMode("existing")}
              className={`flex-1 px-4 py-3 text-sm font-medium transition-colors ${
                mode === "existing"
                  ? "text-blue-400 border-b-2 border-blue-400"
                  : "text-slate-400 hover:text-white"
              }`}
            >
              Import Existing
            </button>
            <button
              type="button"
              onClick={() => setMode("new")}
              className={`flex-1 px-4 py-3 text-sm font-medium transition-colors ${
                mode === "new"
                  ? "text-blue-400 border-b-2 border-blue-400"
                  : "text-slate-400 hover:text-white"
              }`}
            >
              Create New
            </button>
          </div>
        )}

        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          {error && (
            <div className="p-3 bg-red-500/20 border border-red-500/50 rounded-lg text-red-400 text-sm">
              {error}
            </div>
          )}

          {!isEdit && (
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-1">
                {mode === "new" ? "Parent Folder" : "Project Path"}
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={path}
                  onChange={(e) => setPath(e.target.value)}
                  placeholder={mode === "new" ? "C:\\DevPort\\projects" : "C:\\Projects\\my-app"}
                  className="flex-1 px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white
                    placeholder-slate-500 focus:outline-none focus:border-blue-500"
                  required
                />
                <button
                  type="button"
                  onClick={handleSelectFolder}
                  className="px-3 py-2 bg-slate-700 hover:bg-slate-600 rounded-lg text-slate-300 transition-colors"
                >
                  <FolderOpen size={20} />
                </button>
                {mode === "existing" && (
                  <button
                    type="button"
                    onClick={() => handleDetect(path)}
                    disabled={!path || isDetecting}
                    className="px-3 py-2 bg-purple-600 hover:bg-purple-700 rounded-lg text-white transition-colors
                      disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {isDetecting ? <Loader2 size={20} className="animate-spin" /> : <Wand2 size={20} />}
                  </button>
                )}
              </div>
            </div>
          )}

          <div>
            <label className="block text-sm font-medium text-slate-300 mb-1">
              Project Name
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="my-awesome-project"
              className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white
                placeholder-slate-500 focus:outline-none focus:border-blue-500"
              required
            />
          </div>

          {!isEdit && mode === "new" && (
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-1">
                Framework
              </label>
              <div className="grid grid-cols-3 gap-2">
                {FRAMEWORK_OPTIONS.map((fw) => (
                  <button
                    key={fw.value}
                    type="button"
                    onClick={() => handleFrameworkChange(fw.value as ProjectType)}
                    className={`px-3 py-2 text-sm rounded-lg border transition-colors ${
                      projectType === fw.value
                        ? "bg-blue-600 border-blue-500 text-white"
                        : "bg-slate-900 border-slate-700 text-slate-300 hover:border-slate-500"
                    }`}
                  >
                    {fw.label}
                  </button>
                ))}
              </div>
            </div>
          )}

          {!isEdit && mode === "existing" && (
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-1">
                Project Type
              </label>
              <select
                value={projectType}
                onChange={(e) => handleFrameworkChange(e.target.value as ProjectType)}
                className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white
                  focus:outline-none focus:border-blue-500"
              >
                {FRAMEWORK_OPTIONS.map((fw) => (
                  <option key={fw.value} value={fw.value}>
                    {fw.label}
                  </option>
                ))}
              </select>
            </div>
          )}

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-1">
                Port
              </label>
              <div className="relative">
                <input
                  type="number"
                  value={port}
                  onChange={(e) => setPort(e.target.value)}
                  min={1}
                  max={65535}
                  className={`w-full px-3 py-2 bg-slate-900 border rounded-lg text-white
                    placeholder-slate-500 focus:outline-none ${
                      portAvailable === false
                        ? "border-yellow-500"
                        : portAvailable === true
                        ? "border-green-500"
                        : "border-slate-700 focus:border-blue-500"
                    }`}
                  required
                />
                {portAvailable !== null && (
                  <span className={`absolute right-3 top-1/2 -translate-y-1/2 text-xs ${
                    portAvailable ? "text-green-400" : "text-yellow-400"
                  }`}>
                    {portAvailable ? "Available" : "In use"}
                  </span>
                )}
              </div>
            </div>

            {!isEdit && (
              <div>
                <div className="flex items-center justify-between mb-1">
                  <label className="text-sm font-medium text-slate-300">
                    <Globe size={14} className="inline mr-1" />
                    Domain
                  </label>
                  <button
                    type="button"
                    onClick={() => setDomainEnabled(!domainEnabled)}
                    className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${
                      domainEnabled ? "bg-blue-600" : "bg-slate-600"
                    }`}
                  >
                    <span
                      className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform ${
                        domainEnabled ? "translate-x-4.5" : "translate-x-1"
                      }`}
                      style={{ transform: domainEnabled ? "translateX(18px)" : "translateX(4px)" }}
                    />
                  </button>
                </div>
                <div className="relative">
                  <input
                    type="text"
                    value={domain}
                    onChange={(e) => setDomain(e.target.value)}
                    disabled={!domainEnabled}
                    placeholder="my-app.test"
                    className={`w-full px-3 py-2 bg-slate-900 border rounded-lg text-white
                      placeholder-slate-500 focus:outline-none transition-opacity ${
                        !domainEnabled
                          ? "opacity-50 cursor-not-allowed border-slate-700"
                          : domainCheck?.valid === false || domainCheck?.available === false
                          ? "border-red-500"
                          : domainCheck?.available === true
                          ? "border-green-500"
                          : "border-slate-700 focus:border-blue-500"
                      }`}
                  />
                  {domainCheck !== null && (
                    <span className={`absolute right-3 top-1/2 -translate-y-1/2 text-xs ${
                      domainCheck.available && domainCheck.valid ? "text-green-400" : "text-red-400"
                    } ${!domainEnabled ? "opacity-50" : ""}`}>
                      {domainCheck.available && domainCheck.valid ? "Available" :
                       !domainCheck.valid ? "Invalid" : "Conflict"}
                    </span>
                  )}
                </div>
                {domainEnabled && domainCheck?.error && (
                  <p className="mt-1 text-xs text-red-400">{domainCheck.error}</p>
                )}
                <p className="mt-1 text-xs text-slate-500">
                  {domainEnabled ? "Use .test, .localhost, or .local TLD" : "Enable to use custom domain"}
                </p>
              </div>
            )}
          </div>

          <div>
            <label className="block text-sm font-medium text-slate-300 mb-1">
              Start Command
            </label>
            <input
              type="text"
              value={startCommand}
              onChange={(e) => setStartCommand(e.target.value)}
              placeholder="npm run dev"
              className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white
                placeholder-slate-500 focus:outline-none focus:border-blue-500"
              required
            />
          </div>

          {!isEdit && mode === "new" && (
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-1">
                <Package size={14} className="inline mr-1" />
                Package Manager
              </label>
              <div className="flex gap-2">
                {(["npm", "pnpm", "yarn"] as PackageManager[]).map((pm) => (
                  <button
                    key={pm}
                    type="button"
                    onClick={() => setPackageManager(pm)}
                    className={`px-4 py-2 rounded-lg border transition-colors ${
                      packageManager === pm
                        ? "bg-blue-600 border-blue-500 text-white"
                        : "bg-slate-900 border-slate-700 text-slate-300 hover:border-slate-500"
                    }`}
                  >
                    {pm}
                  </button>
                ))}
              </div>
            </div>
          )}

          <div>
            <label className="block text-sm font-medium text-slate-300 mb-1">
              Health Check URL (optional)
            </label>
            <input
              type="text"
              value={healthCheckUrl}
              onChange={(e) => setHealthCheckUrl(e.target.value)}
              placeholder="http://localhost:3000/api/health"
              className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white
                placeholder-slate-500 focus:outline-none focus:border-blue-500"
            />
          </div>

          {!isEdit && (
            <div className="p-4 bg-slate-900/50 border border-slate-700 rounded-lg space-y-3">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={createDatabase}
                  onChange={(e) => setCreateDatabase(e.target.checked)}
                  className="w-4 h-4 rounded border-slate-700 bg-slate-900 text-blue-600"
                />
                <Database size={16} className="text-slate-400" />
                <span className="text-sm text-slate-300">Create database for this project</span>
              </label>

              {createDatabase && (
                <div className="ml-6">
                  <input
                    type="text"
                    value={databaseName}
                    onChange={(e) => setDatabaseName(e.target.value)}
                    placeholder="my_app_db"
                    className="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded text-white text-sm
                      placeholder-slate-500 focus:outline-none focus:border-blue-500"
                  />
                  <p className="mt-1 text-xs text-slate-500">
                    A dedicated user will be created with full access to this database
                  </p>
                </div>
              )}
            </div>
          )}

          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              id="autoStart"
              checked={autoStart}
              onChange={(e) => setAutoStart(e.target.checked)}
              className="w-4 h-4 rounded border-slate-700 bg-slate-900 text-blue-600
                focus:ring-blue-500 focus:ring-offset-slate-800"
            />
            <label htmlFor="autoStart" className="text-sm text-slate-300">
              Auto-start on app launch
            </label>
          </div>

          <div className="flex justify-end gap-3 pt-4">
            <button
              type="button"
              onClick={closeModal}
              className="px-4 py-2 rounded-lg text-slate-300 hover:bg-slate-700 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSubmitting}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg text-white font-medium
                transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isSubmitting ? (
                <Loader2 size={20} className="animate-spin" />
              ) : isEdit ? (
                "Save Changes"
              ) : mode === "new" ? (
                "Create Project"
              ) : (
                "Add Project"
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
