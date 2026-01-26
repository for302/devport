import { useState, useEffect } from "react";
import {
  X,
  Plus,
  Trash2,
  Eye,
  EyeOff,
  Save,
  Copy,
  FileText,
  ChevronDown,
  ChevronUp,
  TestTube,
  Server,
  Rocket,
  Settings,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import type { EnvVariable, EnvProfile, ProfileInfo } from "@/types";
import { useUiStore } from "@/stores";
import { ProfileManager } from "@/components/env";

const PROFILE_ICONS: Record<string, React.ElementType> = {
  development: TestTube,
  staging: Server,
  production: Rocket,
};

const PROFILE_COLORS: Record<string, string> = {
  development: "text-blue-400",
  staging: "text-yellow-400",
  production: "text-red-400",
};

export function EnvEditorModal() {
  const closeModal = useUiStore((state) => state.closeModal);
  const modalData = useUiStore((state) => state.modalData);

  const projectPath = modalData.projectPath as string;
  const projectName = modalData.projectName as string;

  const [profiles, setProfiles] = useState<EnvProfile[]>([]);
  const [profileInfos, setProfileInfos] = useState<ProfileInfo[]>([]);
  const [activeProfile, setActiveProfile] = useState<string>(".env");
  const [variables, setVariables] = useState<EnvVariable[]>([]);
  const [visibleSecrets, setVisibleSecrets] = useState<Set<string>>(new Set());
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);
  const [newVarKey, setNewVarKey] = useState("");
  const [newVarValue, setNewVarValue] = useState("");
  const [showProfileManager, setShowProfileManager] = useState(false);

  useEffect(() => {
    loadProfiles();
    loadActiveProfile();
  }, [projectPath]);

  useEffect(() => {
    loadVariables(activeProfile);
  }, [activeProfile, projectPath]);

  const loadActiveProfile = async () => {
    try {
      const active = await invoke<string>("get_active_profile", { projectPath });
      setActiveProfile(active);
    } catch (error) {
      console.error("Failed to load active profile:", error);
    }
  };

  const loadProfiles = async () => {
    try {
      const envFiles = await invoke<string[]>("get_env_files", { projectPath });
      const profileList: EnvProfile[] = envFiles.map((file) => ({
        name: file === ".env" ? "Default" : file.replace(".env.", ""),
        fileName: file,
        variables: [],
      }));
      setProfiles(profileList);

      // Also load detailed profile info
      const infos = await invoke<ProfileInfo[]>("list_profiles", { projectPath });
      setProfileInfos(infos);

      if (profileList.length > 0 && !envFiles.includes(activeProfile)) {
        setActiveProfile(profileList[0].fileName);
      }
    } catch (error) {
      console.error("Failed to load profiles:", error);
    }
  };

  const loadVariables = async (fileName: string) => {
    setIsLoading(true);
    try {
      const vars = await invoke<EnvVariable[]>("read_env_file", {
        projectPath,
        fileName,
      });
      setVariables(vars);
      setHasChanges(false);
    } catch (error) {
      console.error("Failed to load variables:", error);
      setVariables([]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await invoke("write_env_file", {
        projectPath,
        fileName: activeProfile,
        variables,
      });
      setHasChanges(false);
    } catch (error) {
      console.error("Failed to save:", error);
    } finally {
      setIsSaving(false);
    }
  };

  const handleAddVariable = async () => {
    if (!newVarKey.trim()) return;

    const newVar: EnvVariable = {
      key: newVarKey.trim().toUpperCase().replace(/\s+/g, "_"),
      value: newVarValue,
      isSecret: false,
      comment: null,
    };

    setVariables([...variables, newVar]);
    setNewVarKey("");
    setNewVarValue("");
    setHasChanges(true);
  };

  const handleUpdateVariable = (key: string, value: string) => {
    setVariables(
      variables.map((v) => (v.key === key ? { ...v, value } : v))
    );
    setHasChanges(true);
  };

  const handleDeleteVariable = (key: string) => {
    setVariables(variables.filter((v) => v.key !== key));
    setHasChanges(true);
  };

  const toggleSecretVisibility = (key: string) => {
    const newSet = new Set(visibleSecrets);
    if (newSet.has(key)) {
      newSet.delete(key);
    } else {
      newSet.add(key);
    }
    setVisibleSecrets(newSet);
  };

  const handleCreateProfile = async () => {
    const name = prompt("Enter profile name (e.g., staging, production):");
    if (!name) return;

    const fileName = `.env.${name.toLowerCase().replace(/\s+/g, "-")}`;
    try {
      await invoke("create_env_file", { projectPath, fileName });
      await loadProfiles();
      setActiveProfile(fileName);
    } catch (error) {
      alert(`Failed to create profile: ${error}`);
    }
  };

  const handleCopyProfile = async () => {
    const name = prompt("Enter new profile name:");
    if (!name) return;

    const destination = `.env.${name.toLowerCase().replace(/\s+/g, "-")}`;
    try {
      await invoke("copy_env_file", {
        projectPath,
        source: activeProfile,
        destination,
      });
      await loadProfiles();
      setActiveProfile(destination);
    } catch (error) {
      alert(`Failed to copy profile: ${error}`);
    }
  };

  const handleProfileChange = async (profileFileName: string) => {
    if (hasChanges) {
      const confirm = window.confirm(
        "You have unsaved changes. Switch profile anyway?"
      );
      if (!confirm) return;
    }
    setActiveProfile(profileFileName);
  };

  const getActiveProfileInfo = () => {
    return profileInfos.find((p) => p.fileName === activeProfile);
  };

  const getProfileTypeString = (info: ProfileInfo | undefined): string => {
    if (!info) return "development";
    if (typeof info.profileType === "string") return info.profileType;
    return "custom";
  };

  const activeProfileInfo = getActiveProfileInfo();
  const activeProfileType = getProfileTypeString(activeProfileInfo);
  const ProfileIcon = PROFILE_ICONS[activeProfileType] || Settings;
  const profileColor = PROFILE_COLORS[activeProfileType] || "text-slate-400";

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-slate-900 border border-slate-700 rounded-lg w-[90vw] max-w-4xl max-h-[85vh] flex flex-col">
        <div className="flex items-center justify-between p-4 border-b border-slate-700">
          <div>
            <h2 className="text-lg font-semibold text-white">
              Environment Variables
            </h2>
            <p className="text-sm text-slate-400">{projectName}</p>
          </div>
          <div className="flex items-center gap-2">
            {hasChanges && (
              <span className="text-sm text-yellow-500">Unsaved changes</span>
            )}
            <button
              onClick={handleSave}
              disabled={isSaving || !hasChanges}
              className="flex items-center gap-1.5 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white text-sm rounded"
            >
              <Save size={16} />
              Save
            </button>
            <button
              onClick={closeModal}
              className="p-2 text-slate-400 hover:text-white hover:bg-slate-800 rounded"
            >
              <X size={18} />
            </button>
          </div>
        </div>

        {/* Profile Selector Bar */}
        <div className="flex items-center gap-2 p-4 border-b border-slate-700 bg-slate-800/50">
          <div className="flex items-center gap-2 flex-1">
            <span className="text-sm text-slate-400">Profile:</span>
            <div className="flex items-center gap-2">
              <ProfileIcon size={18} className={profileColor} />
              <select
                value={activeProfile}
                onChange={(e) => handleProfileChange(e.target.value)}
                className="px-3 py-1.5 bg-slate-800 border border-slate-600 rounded text-white text-sm min-w-[200px]"
              >
                {profiles.map((profile) => (
                  <option key={profile.fileName} value={profile.fileName}>
                    {profile.name} ({profile.fileName})
                  </option>
                ))}
              </select>
            </div>
            {activeProfileInfo?.isActive && (
              <span className="text-xs bg-green-500/20 text-green-400 px-2 py-0.5 rounded">
                Active
              </span>
            )}
          </div>
          <div className="flex items-center gap-1">
            <button
              onClick={handleCreateProfile}
              className="p-1.5 text-slate-400 hover:text-white hover:bg-slate-700 rounded"
              title="Create new profile"
            >
              <Plus size={18} />
            </button>
            <button
              onClick={handleCopyProfile}
              className="p-1.5 text-slate-400 hover:text-white hover:bg-slate-700 rounded"
              title="Copy current profile"
            >
              <Copy size={18} />
            </button>
            <button
              onClick={() => setShowProfileManager(!showProfileManager)}
              className={`p-1.5 rounded ${
                showProfileManager
                  ? "text-blue-400 bg-blue-500/20"
                  : "text-slate-400 hover:text-white hover:bg-slate-700"
              }`}
              title="Profile management"
            >
              {showProfileManager ? (
                <ChevronUp size={18} />
              ) : (
                <ChevronDown size={18} />
              )}
            </button>
          </div>
        </div>

        {/* Profile Manager Panel (collapsible) */}
        {showProfileManager && (
          <div className="p-4 border-b border-slate-700 bg-slate-800/30">
            <ProfileManager
              projectPath={projectPath}
              activeProfile={activeProfile}
              onProfileChange={handleProfileChange}
              onRefresh={loadProfiles}
            />
          </div>
        )}

        <div className="flex-1 overflow-auto p-4">
          {isLoading ? (
            <div className="text-center py-8 text-slate-400">Loading...</div>
          ) : (
            <div className="space-y-2">
              {variables.map((variable) => (
                <div
                  key={variable.key}
                  className="flex items-center gap-2 p-2 bg-slate-800/50 rounded group"
                >
                  <input
                    type="text"
                    value={variable.key}
                    readOnly
                    className="w-48 px-2 py-1.5 bg-slate-700 border border-slate-600 rounded text-slate-300 text-sm font-mono"
                  />
                  <span className="text-slate-500">=</span>
                  <div className="flex-1 relative">
                    <input
                      type={
                        variable.isSecret && !visibleSecrets.has(variable.key)
                          ? "password"
                          : "text"
                      }
                      value={variable.value}
                      onChange={(e) =>
                        handleUpdateVariable(variable.key, e.target.value)
                      }
                      className="w-full px-2 py-1.5 pr-10 bg-slate-700 border border-slate-600 rounded text-white text-sm font-mono"
                    />
                    {variable.isSecret && (
                      <button
                        onClick={() => toggleSecretVisibility(variable.key)}
                        className="absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 hover:text-white"
                      >
                        {visibleSecrets.has(variable.key) ? (
                          <EyeOff size={16} />
                        ) : (
                          <Eye size={16} />
                        )}
                      </button>
                    )}
                  </div>
                  <button
                    onClick={() => handleDeleteVariable(variable.key)}
                    className="p-1.5 text-slate-500 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity"
                  >
                    <Trash2 size={16} />
                  </button>
                </div>
              ))}

              <div className="flex items-center gap-2 p-2 mt-4 border-t border-slate-700 pt-4">
                <input
                  type="text"
                  value={newVarKey}
                  onChange={(e) => setNewVarKey(e.target.value)}
                  placeholder="KEY"
                  className="w-48 px-2 py-1.5 bg-slate-800 border border-slate-600 rounded text-white text-sm font-mono placeholder:text-slate-500"
                />
                <span className="text-slate-500">=</span>
                <input
                  type="text"
                  value={newVarValue}
                  onChange={(e) => setNewVarValue(e.target.value)}
                  placeholder="value"
                  className="flex-1 px-2 py-1.5 bg-slate-800 border border-slate-600 rounded text-white text-sm font-mono placeholder:text-slate-500"
                  onKeyDown={(e) => e.key === "Enter" && handleAddVariable()}
                />
                <button
                  onClick={handleAddVariable}
                  disabled={!newVarKey.trim()}
                  className="flex items-center gap-1.5 px-3 py-1.5 bg-green-600 hover:bg-green-700 disabled:opacity-50 text-white text-sm rounded"
                >
                  <Plus size={16} />
                  Add
                </button>
              </div>
            </div>
          )}
        </div>

        <div className="p-4 border-t border-slate-700 bg-slate-800/50 flex items-center justify-between">
          <span className="text-xs text-slate-500">
            {variables.length} variables
          </span>
          <div className="flex items-center gap-4 text-xs text-slate-500">
            <span className="flex items-center gap-1">
              <FileText size={14} />
              {activeProfile}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
