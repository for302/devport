import { useState, useEffect, useCallback } from "react";
import {
  X,
  Save,
  RotateCcw,
  CheckCircle,
  AlertCircle,
  Loader2,
  FileText,
  Server,
  Database,
  Code,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import type {
  ConfigType,
  ConfigFile,
  ConfigValidationResult,
} from "@/types/config";
import { CONFIG_TYPE_LABELS, CONFIG_FILE_NAMES } from "@/types/config";
import { useUiStore } from "@/stores";

type TabConfig = {
  type: ConfigType;
  icon: typeof Server;
  label: string;
};

const TABS: TabConfig[] = [
  { type: "apache", icon: Server, label: "Apache" },
  { type: "mariadb", icon: Database, label: "MariaDB" },
  { type: "php", icon: Code, label: "PHP" },
];

export function ConfigEditorModal() {
  const closeModal = useUiStore((state) => state.closeModal);
  const modalData = useUiStore((state) => state.modalData);

  const initialTab = (modalData.configType as ConfigType) || "apache";

  const [activeTab, setActiveTab] = useState<ConfigType>(initialTab);
  const [configFiles, setConfigFiles] = useState<Record<ConfigType, ConfigFile | null>>({
    apache: null,
    mariadb: null,
    php: null,
  });
  const [editedContent, setEditedContent] = useState<Record<ConfigType, string>>({
    apache: "",
    mariadb: "",
    php: "",
  });
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isValidating, setIsValidating] = useState(false);
  const [isRestoring, setIsRestoring] = useState(false);
  const [hasChanges, setHasChanges] = useState<Record<ConfigType, boolean>>({
    apache: false,
    mariadb: false,
    php: false,
  });
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [validationResult, setValidationResult] = useState<ConfigValidationResult | null>(null);

  const loadConfig = useCallback(async (type: ConfigType) => {
    setIsLoading(true);
    setError(null);
    setValidationResult(null);

    try {
      let config: ConfigFile;
      switch (type) {
        case "apache":
          config = await invoke<ConfigFile>("get_apache_config");
          break;
        case "mariadb":
          config = await invoke<ConfigFile>("get_mariadb_config");
          break;
        case "php":
          config = await invoke<ConfigFile>("get_php_config");
          break;
      }

      setConfigFiles((prev) => ({ ...prev, [type]: config }));
      setEditedContent((prev) => ({ ...prev, [type]: config.content }));
      setHasChanges((prev) => ({ ...prev, [type]: false }));
    } catch (err) {
      setError(`Failed to load ${CONFIG_TYPE_LABELS[type]} config: ${err}`);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadConfig(activeTab);
  }, [activeTab, loadConfig]);

  const handleContentChange = (content: string) => {
    setEditedContent((prev) => ({ ...prev, [activeTab]: content }));
    const originalContent = configFiles[activeTab]?.content || "";
    setHasChanges((prev) => ({ ...prev, [activeTab]: content !== originalContent }));
    setValidationResult(null);
    setSuccessMessage(null);
  };

  const handleSave = async () => {
    setIsSaving(true);
    setError(null);
    setSuccessMessage(null);

    try {
      const content = editedContent[activeTab];
      switch (activeTab) {
        case "apache":
          await invoke("save_apache_config", { content });
          break;
        case "mariadb":
          await invoke("save_mariadb_config", { content });
          break;
        case "php":
          await invoke("save_php_config", { content });
          break;
      }

      setHasChanges((prev) => ({ ...prev, [activeTab]: false }));
      setSuccessMessage(`${CONFIG_TYPE_LABELS[activeTab]} configuration saved successfully`);

      // Reload config to get updated metadata
      await loadConfig(activeTab);
    } catch (err) {
      setError(`Failed to save: ${err}`);
    } finally {
      setIsSaving(false);
    }
  };

  const handleValidate = async () => {
    if (activeTab !== "apache") return;

    setIsValidating(true);
    setError(null);
    setSuccessMessage(null);

    try {
      const result = await invoke<ConfigValidationResult>("validate_apache_config", {
        content: editedContent.apache,
      });
      setValidationResult(result);
      if (result.valid) {
        setSuccessMessage("Configuration syntax is valid");
      }
    } catch (err) {
      setError(`Validation failed: ${err}`);
    } finally {
      setIsValidating(false);
    }
  };

  const handleRestore = async () => {
    const confirmed = window.confirm(
      `Are you sure you want to restore the ${CONFIG_TYPE_LABELS[activeTab]} configuration from backup? This will overwrite the current configuration.`
    );
    if (!confirmed) return;

    setIsRestoring(true);
    setError(null);
    setSuccessMessage(null);

    try {
      await invoke("restore_config_backup", { configType: activeTab });
      setSuccessMessage(`${CONFIG_TYPE_LABELS[activeTab]} configuration restored from backup`);
      await loadConfig(activeTab);
    } catch (err) {
      setError(`Failed to restore: ${err}`);
    } finally {
      setIsRestoring(false);
    }
  };

  const handleTabChange = (tab: ConfigType) => {
    if (hasChanges[activeTab]) {
      const confirmed = window.confirm(
        "You have unsaved changes. Are you sure you want to switch tabs?"
      );
      if (!confirmed) return;
    }
    setActiveTab(tab);
    setValidationResult(null);
    setError(null);
    setSuccessMessage(null);
  };

  const handleClose = () => {
    const anyChanges = Object.values(hasChanges).some(Boolean);
    if (anyChanges) {
      const confirmed = window.confirm(
        "You have unsaved changes. Are you sure you want to close?"
      );
      if (!confirmed) return;
    }
    closeModal();
  };

  const currentConfig = configFiles[activeTab];
  const currentHasChanges = hasChanges[activeTab];
  const currentContent = editedContent[activeTab];

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-slate-900 border border-slate-700 rounded-lg w-[95vw] max-w-5xl h-[90vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-slate-700">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-blue-500/20 rounded-lg">
              <FileText size={24} className="text-blue-400" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-white">
                Configuration Editor
              </h2>
              <p className="text-sm text-slate-400">
                Edit service configuration files
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            {currentHasChanges && (
              <span className="text-sm text-yellow-500 mr-2">Unsaved changes</span>
            )}
            <button
              onClick={handleClose}
              className="p-2 text-slate-400 hover:text-white hover:bg-slate-800 rounded transition-colors"
            >
              <X size={20} />
            </button>
          </div>
        </div>

        {/* Tabs */}
        <div className="flex items-center gap-1 px-4 pt-3 border-b border-slate-700 bg-slate-800/50">
          {TABS.map((tab) => {
            const Icon = tab.icon;
            const isActive = activeTab === tab.type;
            const tabHasChanges = hasChanges[tab.type];

            return (
              <button
                key={tab.type}
                onClick={() => handleTabChange(tab.type)}
                className={`flex items-center gap-2 px-4 py-2.5 rounded-t-lg text-sm font-medium transition-colors
                  ${
                    isActive
                      ? "bg-slate-900 text-white border-t border-l border-r border-slate-700 -mb-px"
                      : "text-slate-400 hover:text-white hover:bg-slate-700/50"
                  }`}
              >
                <Icon size={16} />
                {tab.label}
                {tabHasChanges && (
                  <span className="w-2 h-2 bg-yellow-500 rounded-full" />
                )}
              </button>
            );
          })}
        </div>

        {/* File path bar */}
        {currentConfig && (
          <div className="flex items-center justify-between px-4 py-2 bg-slate-800/30 border-b border-slate-700">
            <div className="flex items-center gap-2 text-sm">
              <span className="text-slate-500">File:</span>
              <code className="px-2 py-0.5 bg-slate-700 rounded text-slate-300 font-mono text-xs">
                {currentConfig.path}
              </code>
            </div>
            {currentConfig.lastModified && (
              <span className="text-xs text-slate-500">
                Last modified: {new Date(currentConfig.lastModified).toLocaleString()}
              </span>
            )}
          </div>
        )}

        {/* Messages */}
        {error && (
          <div className="mx-4 mt-3 p-3 bg-red-500/20 border border-red-500/50 rounded-lg flex items-start gap-2">
            <AlertCircle size={18} className="text-red-400 flex-shrink-0 mt-0.5" />
            <span className="text-red-400 text-sm">{error}</span>
          </div>
        )}

        {successMessage && (
          <div className="mx-4 mt-3 p-3 bg-green-500/20 border border-green-500/50 rounded-lg flex items-start gap-2">
            <CheckCircle size={18} className="text-green-400 flex-shrink-0 mt-0.5" />
            <span className="text-green-400 text-sm">{successMessage}</span>
          </div>
        )}

        {validationResult && !validationResult.valid && (
          <div className="mx-4 mt-3 p-3 bg-red-500/20 border border-red-500/50 rounded-lg">
            <div className="flex items-center gap-2 mb-2">
              <AlertCircle size={18} className="text-red-400" />
              <span className="text-red-400 text-sm font-medium">
                Configuration validation failed
              </span>
            </div>
            <div className="space-y-1">
              {validationResult.errors.map((err, idx) => (
                <div key={idx} className="text-sm text-slate-300 font-mono">
                  <span className="text-red-400">Line {err.line}:</span> {err.message}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Editor */}
        <div className="flex-1 p-4 overflow-hidden">
          {isLoading ? (
            <div className="flex items-center justify-center h-full">
              <Loader2 size={32} className="text-blue-400 animate-spin" />
            </div>
          ) : (
            <textarea
              value={currentContent}
              onChange={(e) => handleContentChange(e.target.value)}
              className="w-full h-full px-4 py-3 bg-slate-950 border border-slate-700 rounded-lg
                text-slate-200 font-mono text-sm leading-relaxed resize-none
                focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500
                scrollbar-thin scrollbar-thumb-slate-600 scrollbar-track-slate-800"
              placeholder={`Enter ${CONFIG_TYPE_LABELS[activeTab]} configuration...`}
              spellCheck={false}
            />
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-4 py-3 border-t border-slate-700 bg-slate-800/50">
          <div className="flex items-center gap-2">
            <button
              onClick={handleRestore}
              disabled={isRestoring || !currentConfig?.hasBackup}
              className="flex items-center gap-2 px-3 py-2 text-sm text-slate-300
                hover:bg-slate-700 rounded-lg transition-colors
                disabled:opacity-50 disabled:cursor-not-allowed"
              title={currentConfig?.hasBackup ? "Restore from backup" : "No backup available"}
            >
              {isRestoring ? (
                <Loader2 size={16} className="animate-spin" />
              ) : (
                <RotateCcw size={16} />
              )}
              Restore Backup
            </button>

            {activeTab === "apache" && (
              <button
                onClick={handleValidate}
                disabled={isValidating}
                className="flex items-center gap-2 px-3 py-2 text-sm text-slate-300
                  hover:bg-slate-700 rounded-lg transition-colors
                  disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isValidating ? (
                  <Loader2 size={16} className="animate-spin" />
                ) : (
                  <CheckCircle size={16} />
                )}
                Validate Config
              </button>
            )}
          </div>

          <div className="flex items-center gap-2">
            <span className="text-xs text-slate-500 mr-2">
              {CONFIG_FILE_NAMES[activeTab]}
            </span>
            <button
              onClick={handleClose}
              className="px-4 py-2 text-sm text-slate-300 hover:bg-slate-700 rounded-lg transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              disabled={isSaving || !currentHasChanges}
              className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700
                text-white text-sm font-medium rounded-lg transition-colors
                disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isSaving ? (
                <Loader2 size={16} className="animate-spin" />
              ) : (
                <Save size={16} />
              )}
              Save Changes
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
