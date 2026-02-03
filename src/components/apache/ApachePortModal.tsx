import { useState, useEffect } from "react";
import {
  X,
  Loader2,
  FolderOpen,
  CheckCircle,
  AlertCircle,
  Plus,
} from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { useApacheConfigStore } from "@/stores";
import type { ApachePortEntry, ApacheVHostRequest } from "@/types";

interface ApachePortModalProps {
  isOpen: boolean;
  onClose: () => void;
  editEntry?: ApachePortEntry | null;
}

export function ApachePortModal({ isOpen, onClose, editEntry }: ApachePortModalProps) {
  const { createVHost, updateVHost, checkDocumentRoot, createDocumentRoot, apacheBasePath } =
    useApacheConfigStore();

  const [port, setPort] = useState(editEntry?.port?.toString() || "80");
  const [domain, setDomain] = useState(editEntry?.domain || "localhost");
  const [documentRoot, setDocumentRoot] = useState(editEntry?.documentRoot || "");
  const [serverAlias, setServerAlias] = useState(editEntry?.serverAlias?.join(" ") || "");
  const [isSsl, setIsSsl] = useState(editEntry?.isSsl || false);

  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isCheckingRoot, setIsCheckingRoot] = useState(false);
  const [rootExists, setRootExists] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Reset form when modal opens/closes or editEntry changes
  useEffect(() => {
    if (isOpen) {
      if (editEntry) {
        setPort(editEntry.port.toString());
        setDomain(editEntry.domain);
        setDocumentRoot(editEntry.documentRoot);
        setServerAlias(editEntry.serverAlias.join(" "));
        setIsSsl(editEntry.isSsl);
      } else {
        setPort("80");
        setDomain("localhost");
        setDocumentRoot(apacheBasePath ? `${apacheBasePath}\\htdocs` : "");
        setServerAlias("");
        setIsSsl(false);
      }
      setError(null);
      setRootExists(null);
    }
  }, [isOpen, editEntry, apacheBasePath]);

  // Check document root when it changes
  useEffect(() => {
    if (documentRoot) {
      setIsCheckingRoot(true);
      const timer = setTimeout(async () => {
        const exists = await checkDocumentRoot(documentRoot);
        setRootExists(exists);
        setIsCheckingRoot(false);
      }, 300);
      return () => clearTimeout(timer);
    } else {
      setRootExists(null);
    }
  }, [documentRoot, checkDocumentRoot]);

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Document Root Folder",
        defaultPath: apacheBasePath || undefined,
      });

      if (selected && typeof selected === "string") {
        setDocumentRoot(selected);
      }
    } catch (err) {
      console.error("Failed to select folder:", err);
    }
  };

  const handleCreateFolder = async () => {
    if (!documentRoot) return;
    try {
      await createDocumentRoot(documentRoot);
      setRootExists(true);
    } catch (err) {
      setError(`Failed to create folder: ${err}`);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setIsSubmitting(true);

    const portNum = parseInt(port, 10);
    if (isNaN(portNum) || portNum < 1 || portNum > 65535) {
      setError("Port must be between 1 and 65535");
      setIsSubmitting(false);
      return;
    }

    if (!domain.trim()) {
      setError("Domain is required");
      setIsSubmitting(false);
      return;
    }

    if (!documentRoot.trim()) {
      setError("Document Root is required");
      setIsSubmitting(false);
      return;
    }

    const request: ApacheVHostRequest = {
      port: portNum,
      domain: domain.trim(),
      documentRoot: documentRoot.trim(),
      serverAlias: serverAlias
        .split(/\s+/)
        .filter((s) => s.trim() !== ""),
      isSsl,
    };

    try {
      if (editEntry) {
        await updateVHost(editEntry.id, request);
      } else {
        await createVHost(request);
      }
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsSubmitting(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-slate-800 rounded-lg w-full max-w-lg mx-4">
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-700">
          <h2 className="text-lg font-semibold text-white">
            {editEntry ? "Edit VirtualHost" : "Add VirtualHost"}
          </h2>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors"
          >
            <X size={20} />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          {error && (
            <div className="p-3 bg-red-500/20 border border-red-500/50 rounded-lg text-red-400 text-sm">
              {error}
            </div>
          )}

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-1">
                Port
              </label>
              <input
                type="number"
                value={port}
                onChange={(e) => setPort(e.target.value)}
                min={1}
                max={65535}
                className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white
                  placeholder-slate-500 focus:outline-none focus:border-blue-500"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-300 mb-1">
                Domain (ServerName)
              </label>
              <input
                type="text"
                value={domain}
                onChange={(e) => setDomain(e.target.value)}
                placeholder="localhost"
                className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white
                  placeholder-slate-500 focus:outline-none focus:border-blue-500"
                required
              />
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-slate-300 mb-1">
              Document Root
            </label>
            <div className="flex gap-2">
              <div className="flex-1 relative">
                <input
                  type="text"
                  value={documentRoot}
                  onChange={(e) => setDocumentRoot(e.target.value)}
                  placeholder="C:\xampp\htdocs\mysite"
                  className={`w-full px-3 py-2 bg-slate-900 border rounded-lg text-white
                    placeholder-slate-500 focus:outline-none ${
                      rootExists === false
                        ? "border-yellow-500"
                        : rootExists === true
                        ? "border-green-500"
                        : "border-slate-700 focus:border-blue-500"
                    }`}
                  required
                />
                {isCheckingRoot ? (
                  <Loader2
                    size={16}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 animate-spin"
                  />
                ) : rootExists !== null ? (
                  rootExists ? (
                    <CheckCircle
                      size={16}
                      className="absolute right-3 top-1/2 -translate-y-1/2 text-green-400"
                    />
                  ) : (
                    <AlertCircle
                      size={16}
                      className="absolute right-3 top-1/2 -translate-y-1/2 text-yellow-400"
                    />
                  )
                ) : null}
              </div>
              <button
                type="button"
                onClick={handleSelectFolder}
                className="px-3 py-2 bg-slate-700 hover:bg-slate-600 rounded-lg text-slate-300 transition-colors"
                title="Browse for folder"
              >
                <FolderOpen size={20} />
              </button>
            </div>
            {rootExists === false && (
              <div className="mt-2 flex items-center gap-2">
                <span className="text-sm text-yellow-400">Folder does not exist.</span>
                <button
                  type="button"
                  onClick={handleCreateFolder}
                  className="flex items-center gap-1 px-2 py-1 text-xs bg-yellow-600/20 hover:bg-yellow-600/30
                    text-yellow-400 rounded transition-colors"
                >
                  <Plus size={14} />
                  Create folder
                </button>
              </div>
            )}
          </div>

          <div>
            <label className="block text-sm font-medium text-slate-300 mb-1">
              Server Aliases (space-separated)
            </label>
            <input
              type="text"
              value={serverAlias}
              onChange={(e) => setServerAlias(e.target.value)}
              placeholder="www.mysite.local alias.mysite.local"
              className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white
                placeholder-slate-500 focus:outline-none focus:border-blue-500"
            />
            <p className="mt-1 text-xs text-slate-500">
              Optional: Additional domains that should point to this VirtualHost
            </p>
          </div>

          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              id="isSsl"
              checked={isSsl}
              onChange={(e) => setIsSsl(e.target.checked)}
              className="w-4 h-4 rounded border-slate-700 bg-slate-900 text-blue-600
                focus:ring-blue-500 focus:ring-offset-slate-800"
            />
            <label htmlFor="isSsl" className="text-sm text-slate-300">
              Enable SSL (HTTPS)
            </label>
          </div>

          <div className="flex justify-end gap-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 rounded-lg text-slate-300 hover:bg-slate-700 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSubmitting}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg text-white font-medium
                transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
            >
              {isSubmitting && <Loader2 size={16} className="animate-spin" />}
              {editEntry ? "Save Changes" : "Add VirtualHost"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
