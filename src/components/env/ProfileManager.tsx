import { useState, useEffect } from "react";
import {
  FolderSync,
  Plus,
  Trash2,
  Download,
  Upload,
  GitCompare,
  Check,
  Server,
  TestTube,
  Rocket,
  Settings,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { save, open } from "@tauri-apps/plugin-dialog";
import type { ProfileInfo, ProfileComparison } from "@/types";

interface ProfileManagerProps {
  projectPath: string;
  activeProfile: string;
  onProfileChange: (profileFileName: string) => void;
  onRefresh: () => void;
}

const PROFILE_ICONS: Record<string, React.ElementType> = {
  development: TestTube,
  staging: Server,
  production: Rocket,
};

const PROFILE_COLORS: Record<string, string> = {
  development: "text-blue-400 bg-blue-500/10 border-blue-500/30",
  staging: "text-yellow-400 bg-yellow-500/10 border-yellow-500/30",
  production: "text-red-400 bg-red-500/10 border-red-500/30",
};

export function ProfileManager({
  projectPath,
  activeProfile,
  onProfileChange,
  onRefresh,
}: ProfileManagerProps) {
  const [profiles, setProfiles] = useState<ProfileInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showCompareModal, setShowCompareModal] = useState(false);
  const [comparison, setComparison] = useState<ProfileComparison | null>(null);
  const [compareProfiles, setCompareProfiles] = useState<{
    a: string;
    b: string;
  }>({ a: "", b: "" });

  useEffect(() => {
    loadProfiles();
  }, [projectPath]);

  const loadProfiles = async () => {
    setIsLoading(true);
    try {
      const profileList = await invoke<ProfileInfo[]>("list_profiles", {
        projectPath,
      });
      setProfiles(profileList);
    } catch (error) {
      console.error("Failed to load profiles:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSwitchProfile = async (profileFileName: string) => {
    try {
      await invoke("switch_profile", { projectPath, profileFileName });
      onProfileChange(profileFileName);
      loadProfiles();
    } catch (error) {
      console.error("Failed to switch profile:", error);
      alert(`Failed to switch profile: ${error}`);
    }
  };

  const handleDeleteProfile = async (profileFileName: string) => {
    if (!confirm(`Are you sure you want to delete ${profileFileName}?`)) {
      return;
    }

    try {
      await invoke("delete_profile", { projectPath, profileFileName });
      loadProfiles();
      onRefresh();
    } catch (error) {
      console.error("Failed to delete profile:", error);
      alert(`Failed to delete profile: ${error}`);
    }
  };

  const handleExportProfile = async (profileFileName: string) => {
    try {
      const exportPath = await save({
        defaultPath: profileFileName.replace(".", "_") + ".env",
        filters: [{ name: "Environment File", extensions: ["env"] }],
      });

      if (exportPath) {
        await invoke("export_profile", {
          projectPath,
          profileFileName,
          exportPath,
        });
        alert(`Profile exported to ${exportPath}`);
      }
    } catch (error) {
      console.error("Failed to export profile:", error);
      alert(`Failed to export profile: ${error}`);
    }
  };

  const handleImportProfile = async () => {
    try {
      const importPath = await open({
        filters: [{ name: "Environment File", extensions: ["env", "txt"] }],
      });

      if (importPath) {
        const profileType = prompt(
          "Enter profile type (development, staging, production, or custom name):"
        );
        if (!profileType) return;

        let type = profileType.toLowerCase();
        let customName: string | null = null;

        if (!["development", "staging", "production"].includes(type)) {
          customName = profileType;
          type = "custom";
        }

        await invoke("import_profile", {
          projectPath,
          profileType: type,
          customName,
          importPath: importPath as string,
        });

        loadProfiles();
        onRefresh();
      }
    } catch (error) {
      console.error("Failed to import profile:", error);
      alert(`Failed to import profile: ${error}`);
    }
  };

  const handleCompareProfiles = async () => {
    if (!compareProfiles.a || !compareProfiles.b) {
      alert("Please select two profiles to compare");
      return;
    }

    try {
      const result = await invoke<ProfileComparison>("compare_profiles", {
        projectPath,
        profileA: compareProfiles.a,
        profileB: compareProfiles.b,
      });
      setComparison(result);
    } catch (error) {
      console.error("Failed to compare profiles:", error);
      alert(`Failed to compare profiles: ${error}`);
    }
  };

  const getProfileIcon = (profile: ProfileInfo) => {
    const profileType =
      typeof profile.profileType === "string"
        ? profile.profileType
        : "custom";
    const Icon = PROFILE_ICONS[profileType] || Settings;
    return Icon;
  };

  const getProfileColor = (profile: ProfileInfo) => {
    const profileType =
      typeof profile.profileType === "string"
        ? profile.profileType
        : "custom";
    return PROFILE_COLORS[profileType] || "text-slate-400 bg-slate-500/10 border-slate-500/30";
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-slate-300 flex items-center gap-2">
          <FolderSync size={16} />
          Environment Profiles
        </h3>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowCompareModal(true)}
            className="p-1.5 text-slate-400 hover:text-white hover:bg-slate-700 rounded"
            title="Compare profiles"
          >
            <GitCompare size={16} />
          </button>
          <button
            onClick={handleImportProfile}
            className="p-1.5 text-slate-400 hover:text-white hover:bg-slate-700 rounded"
            title="Import profile"
          >
            <Upload size={16} />
          </button>
          <button
            onClick={() => setShowCreateModal(true)}
            className="p-1.5 text-slate-400 hover:text-white hover:bg-slate-700 rounded"
            title="Create new profile"
          >
            <Plus size={16} />
          </button>
        </div>
      </div>

      {isLoading ? (
        <div className="text-center py-4 text-slate-400">Loading profiles...</div>
      ) : (
        <div className="grid grid-cols-2 gap-2">
          {profiles.map((profile) => {
            const Icon = getProfileIcon(profile);
            const colorClass = getProfileColor(profile);
            const isActive = profile.fileName === activeProfile;

            return (
              <div
                key={profile.fileName}
                className={`relative p-3 rounded border ${colorClass} ${
                  isActive ? "ring-2 ring-offset-2 ring-offset-slate-900 ring-blue-500" : ""
                }`}
              >
                <div className="flex items-start justify-between">
                  <div className="flex items-center gap-2">
                    <Icon size={18} />
                    <div>
                      <div className="font-medium text-sm">{profile.name}</div>
                      <div className="text-xs opacity-70">{profile.fileName}</div>
                    </div>
                  </div>
                  {isActive && (
                    <span className="text-xs bg-green-500/20 text-green-400 px-1.5 py-0.5 rounded">
                      Active
                    </span>
                  )}
                </div>

                <div className="mt-2 text-xs opacity-70">
                  {profile.variableCount} variables
                </div>

                <div className="mt-2 flex items-center gap-1">
                  {!isActive && (
                    <button
                      onClick={() => handleSwitchProfile(profile.fileName)}
                      className="flex-1 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded"
                      title="Switch to this profile"
                    >
                      Switch
                    </button>
                  )}
                  <button
                    onClick={() => handleExportProfile(profile.fileName)}
                    className="p-1 text-slate-400 hover:text-white hover:bg-slate-700 rounded"
                    title="Export"
                  >
                    <Download size={14} />
                  </button>
                  {profile.fileName !== ".env" && (
                    <button
                      onClick={() => handleDeleteProfile(profile.fileName)}
                      className="p-1 text-slate-400 hover:text-red-400 hover:bg-slate-700 rounded"
                      title="Delete"
                    >
                      <Trash2 size={14} />
                    </button>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Create Profile Modal */}
      {showCreateModal && (
        <CreateProfileModal
          projectPath={projectPath}
          existingProfiles={profiles}
          onClose={() => setShowCreateModal(false)}
          onCreated={() => {
            setShowCreateModal(false);
            loadProfiles();
            onRefresh();
          }}
        />
      )}

      {/* Compare Profiles Modal */}
      {showCompareModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-slate-900 border border-slate-700 rounded-lg w-[600px] max-h-[80vh] overflow-hidden flex flex-col">
            <div className="p-4 border-b border-slate-700">
              <h3 className="text-lg font-semibold text-white">Compare Profiles</h3>
            </div>

            <div className="p-4 space-y-4">
              <div className="flex gap-4">
                <div className="flex-1">
                  <label className="block text-sm text-slate-400 mb-1">Profile A</label>
                  <select
                    value={compareProfiles.a}
                    onChange={(e) =>
                      setCompareProfiles((prev) => ({ ...prev, a: e.target.value }))
                    }
                    className="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded text-white text-sm"
                  >
                    <option value="">Select profile</option>
                    {profiles.map((p) => (
                      <option key={p.fileName} value={p.fileName}>
                        {p.name}
                      </option>
                    ))}
                  </select>
                </div>
                <div className="flex-1">
                  <label className="block text-sm text-slate-400 mb-1">Profile B</label>
                  <select
                    value={compareProfiles.b}
                    onChange={(e) =>
                      setCompareProfiles((prev) => ({ ...prev, b: e.target.value }))
                    }
                    className="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded text-white text-sm"
                  >
                    <option value="">Select profile</option>
                    {profiles.map((p) => (
                      <option key={p.fileName} value={p.fileName}>
                        {p.name}
                      </option>
                    ))}
                  </select>
                </div>
              </div>

              <button
                onClick={handleCompareProfiles}
                disabled={!compareProfiles.a || !compareProfiles.b}
                className="w-full py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white rounded"
              >
                Compare
              </button>

              {comparison && (
                <div className="space-y-4 max-h-[300px] overflow-auto">
                  {comparison.onlyInA.length > 0 && (
                    <div>
                      <h4 className="text-sm font-medium text-yellow-400 mb-2">
                        Only in {comparison.profileA}:
                      </h4>
                      <div className="space-y-1">
                        {comparison.onlyInA.map((key) => (
                          <div key={key} className="text-sm text-slate-300 pl-2">
                            {key}
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  {comparison.onlyInB.length > 0 && (
                    <div>
                      <h4 className="text-sm font-medium text-blue-400 mb-2">
                        Only in {comparison.profileB}:
                      </h4>
                      <div className="space-y-1">
                        {comparison.onlyInB.map((key) => (
                          <div key={key} className="text-sm text-slate-300 pl-2">
                            {key}
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  {comparison.differentValues.length > 0 && (
                    <div>
                      <h4 className="text-sm font-medium text-orange-400 mb-2">
                        Different values:
                      </h4>
                      <div className="space-y-2">
                        {comparison.differentValues.map((diff) => (
                          <div
                            key={diff.key}
                            className="text-sm bg-slate-800 p-2 rounded"
                          >
                            <div className="font-medium text-slate-300">
                              {diff.key}
                            </div>
                            <div className="text-xs text-slate-400 mt-1">
                              A: <span className="text-yellow-400">{diff.valueA || "(empty)"}</span>
                            </div>
                            <div className="text-xs text-slate-400">
                              B: <span className="text-blue-400">{diff.valueB || "(empty)"}</span>
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  {comparison.onlyInA.length === 0 &&
                    comparison.onlyInB.length === 0 &&
                    comparison.differentValues.length === 0 && (
                      <div className="text-center text-green-400 py-4">
                        <Check size={24} className="mx-auto mb-2" />
                        Profiles are identical
                      </div>
                    )}
                </div>
              )}
            </div>

            <div className="p-4 border-t border-slate-700 flex justify-end">
              <button
                onClick={() => {
                  setShowCompareModal(false);
                  setComparison(null);
                }}
                className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

interface CreateProfileModalProps {
  projectPath: string;
  existingProfiles: ProfileInfo[];
  onClose: () => void;
  onCreated: () => void;
}

function CreateProfileModal({
  projectPath,
  existingProfiles,
  onClose,
  onCreated,
}: CreateProfileModalProps) {
  const [profileType, setProfileType] = useState<string>("development");
  const [customName, setCustomName] = useState("");
  const [copyFrom, setCopyFrom] = useState<string>("");
  const [isCreating, setIsCreating] = useState(false);

  const handleCreate = async () => {
    setIsCreating(true);
    try {
      await invoke("create_profile", {
        projectPath,
        profileType,
        customName: profileType === "custom" ? customName : null,
        copyFrom: copyFrom || null,
      });
      onCreated();
    } catch (error) {
      console.error("Failed to create profile:", error);
      alert(`Failed to create profile: ${error}`);
    } finally {
      setIsCreating(false);
    }
  };

  const presetTypes = [
    { value: "development", label: "Development", icon: TestTube, color: "text-blue-400" },
    { value: "staging", label: "Staging", icon: Server, color: "text-yellow-400" },
    { value: "production", label: "Production", icon: Rocket, color: "text-red-400" },
    { value: "custom", label: "Custom", icon: Settings, color: "text-slate-400" },
  ];

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-slate-900 border border-slate-700 rounded-lg w-[400px]">
        <div className="p-4 border-b border-slate-700">
          <h3 className="text-lg font-semibold text-white">Create New Profile</h3>
        </div>

        <div className="p-4 space-y-4">
          <div>
            <label className="block text-sm text-slate-400 mb-2">Profile Type</label>
            <div className="grid grid-cols-2 gap-2">
              {presetTypes.map((type) => {
                const Icon = type.icon;
                const isSelected = profileType === type.value;
                return (
                  <button
                    key={type.value}
                    onClick={() => setProfileType(type.value)}
                    className={`flex items-center gap-2 p-3 rounded border ${
                      isSelected
                        ? "border-blue-500 bg-blue-500/10"
                        : "border-slate-600 hover:border-slate-500"
                    }`}
                  >
                    <Icon size={18} className={type.color} />
                    <span className="text-sm text-white">{type.label}</span>
                  </button>
                );
              })}
            </div>
          </div>

          {profileType === "custom" && (
            <div>
              <label className="block text-sm text-slate-400 mb-1">
                Custom Profile Name
              </label>
              <input
                type="text"
                value={customName}
                onChange={(e) => setCustomName(e.target.value)}
                placeholder="e.g., testing, qa, local"
                className="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded text-white text-sm"
              />
            </div>
          )}

          <div>
            <label className="block text-sm text-slate-400 mb-1">
              Copy variables from (optional)
            </label>
            <select
              value={copyFrom}
              onChange={(e) => setCopyFrom(e.target.value)}
              className="w-full px-3 py-2 bg-slate-800 border border-slate-600 rounded text-white text-sm"
            >
              <option value="">Start with empty profile</option>
              {existingProfiles.map((p) => (
                <option key={p.fileName} value={p.fileName}>
                  {p.name} ({p.variableCount} vars)
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className="p-4 border-t border-slate-700 flex justify-end gap-2">
          <button
            onClick={onClose}
            className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded"
          >
            Cancel
          </button>
          <button
            onClick={handleCreate}
            disabled={isCreating || (profileType === "custom" && !customName.trim())}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white rounded"
          >
            {isCreating ? "Creating..." : "Create Profile"}
          </button>
        </div>
      </div>
    </div>
  );
}
