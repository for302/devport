import { useState, useEffect } from "react";
import { X, Save, FileText, AlertCircle, Loader2 } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import type { ConfigFileInfo } from "@/types/service";

interface ServiceConfigModalProps {
  serviceId: string;
  serviceName: string;
  configFiles: ConfigFileInfo[];
  isOpen: boolean;
  onClose: () => void;
}

export function ServiceConfigModal({
  serviceId,
  serviceName,
  configFiles,
  isOpen,
  onClose,
}: ServiceConfigModalProps) {
  const [selectedFile, setSelectedFile] = useState<ConfigFileInfo | null>(null);
  const [content, setContent] = useState("");
  const [originalContent, setOriginalContent] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasChanges, setHasChanges] = useState(false);

  useEffect(() => {
    if (configFiles.length > 0 && !selectedFile) {
      setSelectedFile(configFiles[0]);
    }
  }, [configFiles, selectedFile]);

  useEffect(() => {
    if (selectedFile) {
      loadConfigFile(selectedFile);
    }
  }, [selectedFile]);

  useEffect(() => {
    setHasChanges(content !== originalContent);
  }, [content, originalContent]);

  const loadConfigFile = async (file: ConfigFileInfo) => {
    setIsLoading(true);
    setError(null);
    try {
      const fileContent = await invoke<string>("get_service_config", {
        id: serviceId,
        configName: file.name,
      });
      setContent(fileContent);
      setOriginalContent(fileContent);
    } catch (err) {
      setError(`Failed to load config: ${err}`);
      setContent("");
      setOriginalContent("");
    } finally {
      setIsLoading(false);
    }
  };

  const handleSave = async () => {
    if (!selectedFile) return;

    setIsSaving(true);
    setError(null);
    try {
      await invoke("save_service_config", {
        id: serviceId,
        configName: selectedFile.name,
        content,
      });
      setOriginalContent(content);
      setHasChanges(false);
    } catch (err) {
      setError(`Failed to save config: ${err}`);
    } finally {
      setIsSaving(false);
    }
  };

  const handleClose = () => {
    if (hasChanges) {
      if (!confirm("You have unsaved changes. Are you sure you want to close?")) {
        return;
      }
    }
    onClose();
    setSelectedFile(null);
    setContent("");
    setOriginalContent("");
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70">
      <div className="bg-slate-900 border border-slate-700 rounded-lg shadow-xl w-[900px] max-w-[95vw] h-[700px] max-h-[90vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-slate-700">
          <h2 className="text-lg font-semibold text-white">
            {serviceName} Configuration
          </h2>
          <button
            onClick={handleClose}
            className="p-1 text-slate-400 hover:text-white transition-colors"
          >
            <X size={20} />
          </button>
        </div>

        <div className="flex flex-1 overflow-hidden">
          {/* Sidebar - Config Files List */}
          <div className="w-56 border-r border-slate-700 overflow-y-auto">
            <div className="p-2 text-xs text-slate-500 uppercase font-semibold">
              Config Files
            </div>
            {configFiles.map((file) => (
              <button
                key={file.name}
                onClick={() => setSelectedFile(file)}
                className={`w-full text-left px-3 py-2 flex items-start gap-2 hover:bg-slate-800 transition-colors ${
                  selectedFile?.name === file.name
                    ? "bg-slate-800 border-l-2 border-blue-500"
                    : ""
                }`}
              >
                <FileText size={16} className="text-slate-400 mt-0.5 flex-shrink-0" />
                <div className="min-w-0">
                  <div className="text-sm text-white truncate">{file.name}</div>
                  <div className="text-xs text-slate-500 truncate">
                    {file.description}
                  </div>
                </div>
              </button>
            ))}
          </div>

          {/* Main Content - Editor */}
          <div className="flex-1 flex flex-col overflow-hidden">
            {/* File Info Bar */}
            {selectedFile && (
              <div className="px-4 py-2 bg-slate-800/50 border-b border-slate-700 flex items-center justify-between">
                <div className="flex items-center gap-2 text-sm text-slate-400">
                  <FileText size={14} />
                  <span className="font-mono text-xs">{selectedFile.path}</span>
                </div>
                {hasChanges && (
                  <span className="text-xs text-yellow-500">Unsaved changes</span>
                )}
              </div>
            )}

            {/* Error Message */}
            {error && (
              <div className="mx-4 mt-2 p-2 bg-red-900/20 border border-red-800 rounded flex items-center gap-2 text-sm text-red-400">
                <AlertCircle size={16} />
                {error}
              </div>
            )}

            {/* Editor Area */}
            <div className="flex-1 overflow-hidden p-4">
              {isLoading ? (
                <div className="h-full flex items-center justify-center">
                  <Loader2 size={24} className="animate-spin text-slate-400" />
                </div>
              ) : (
                <textarea
                  value={content}
                  onChange={(e) => setContent(e.target.value)}
                  className="w-full h-full bg-slate-950 border border-slate-700 rounded p-3 font-mono text-sm text-slate-300 resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
                  spellCheck={false}
                  placeholder="Select a config file to edit..."
                />
              )}
            </div>

            {/* Footer */}
            <div className="px-4 py-3 bg-slate-800/30 border-t border-slate-700 flex items-center justify-between">
              <div className="text-xs text-slate-500">
                Changes require service restart to take effect
              </div>
              <div className="flex gap-2">
                <button
                  onClick={handleClose}
                  className="px-4 py-2 text-sm text-slate-300 hover:text-white transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={handleSave}
                  disabled={!hasChanges || isSaving}
                  className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed text-white text-sm rounded transition-colors"
                >
                  {isSaving ? (
                    <Loader2 size={14} className="animate-spin" />
                  ) : (
                    <Save size={14} />
                  )}
                  Save
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
