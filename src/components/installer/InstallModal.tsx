import { useState, useEffect } from "react";
import { X, AlertTriangle, Check, Loader2 } from "lucide-react";
import type { ComponentInfo, InstallOptions } from "@/types";
import { formatInstallerBytes } from "@/types/installer";
import { useInstallerStore } from "@/stores";

interface InstallModalProps {
  component: ComponentInfo;
  isOpen: boolean;
  onClose: () => void;
  onInstallComplete?: () => void;
}

export function InstallModal({
  component,
  isOpen,
  onClose,
  onInstallComplete,
}: InstallModalProps) {
  const [options, setOptions] = useState<InstallOptions>({
    componentId: component.id,
    version: component.version,
    autoConfigure: true,
    addToPath: true,
    linkToApache: component.category === "Runtime" && component.id === "php",
  });
  const [isInstalling, setIsInstalling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const { installComponent, installProgress } = useInstallerStore();

  useEffect(() => {
    if (isOpen) {
      setOptions({
        componentId: component.id,
        version: component.version,
        autoConfigure: true,
        addToPath: true,
        linkToApache: component.category === "Runtime" && component.id === "php",
      });
      setIsInstalling(false);
      setError(null);
      setSuccess(false);
    }
  }, [isOpen, component]);

  const handleInstall = async () => {
    setIsInstalling(true);
    setError(null);

    try {
      await installComponent(component.id, options);
      setSuccess(true);
      setTimeout(() => {
        onClose();
        onInstallComplete?.();
      }, 1500);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsInstalling(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="relative bg-zinc-900 border border-zinc-700 rounded-lg shadow-xl w-full max-w-md mx-4">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-700">
          <h2 className="text-lg font-semibold text-zinc-100">
            {component.name} 설치
          </h2>
          <button
            onClick={onClose}
            disabled={isInstalling}
            className="p-1 hover:bg-zinc-800 rounded transition-colors disabled:opacity-50"
          >
            <X className="w-5 h-5 text-zinc-400" />
          </button>
        </div>

        {/* Content */}
        <div className="px-6 py-4 space-y-4">
          {/* Version */}
          <div>
            <label className="block text-sm font-medium text-zinc-300 mb-2">
              버전
            </label>
            <div className="px-3 py-2 bg-zinc-800 border border-zinc-700 rounded text-zinc-200">
              {component.version}
            </div>
          </div>

          {/* Options */}
          <div className="space-y-3">
            <label className="block text-sm font-medium text-zinc-300">
              설치 옵션
            </label>

            <label className="flex items-center gap-3 cursor-pointer">
              <input
                type="checkbox"
                checked={options.autoConfigure}
                onChange={(e) =>
                  setOptions((prev) => ({
                    ...prev,
                    autoConfigure: e.target.checked,
                  }))
                }
                disabled={isInstalling}
                className="w-4 h-4 rounded border-zinc-600 bg-zinc-800 text-blue-500 focus:ring-blue-500"
              />
              <span className="text-sm text-zinc-300">자동 설정 적용</span>
            </label>

            <label className="flex items-center gap-3 cursor-pointer">
              <input
                type="checkbox"
                checked={options.addToPath}
                onChange={(e) =>
                  setOptions((prev) => ({
                    ...prev,
                    addToPath: e.target.checked,
                  }))
                }
                disabled={isInstalling}
                className="w-4 h-4 rounded border-zinc-600 bg-zinc-800 text-blue-500 focus:ring-blue-500"
              />
              <span className="text-sm text-zinc-300">PATH 환경변수 추가</span>
            </label>

            {component.id === "php" && (
              <label className="flex items-center gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  checked={options.linkToApache}
                  onChange={(e) =>
                    setOptions((prev) => ({
                      ...prev,
                      linkToApache: e.target.checked,
                    }))
                  }
                  disabled={isInstalling}
                  className="w-4 h-4 rounded border-zinc-600 bg-zinc-800 text-blue-500 focus:ring-blue-500"
                />
                <span className="text-sm text-zinc-300">
                  Apache 모듈 연동 (Apache 설치 시)
                </span>
              </label>
            )}
          </div>

          {/* Install path */}
          <div>
            <label className="block text-sm font-medium text-zinc-300 mb-2">
              설치 경로
            </label>
            <div className="px-3 py-2 bg-zinc-800 border border-zinc-700 rounded text-zinc-400 text-sm font-mono">
              C:\DevPort\runtime\{component.id}
            </div>
          </div>

          {/* Size info */}
          <div className="flex items-center justify-between text-sm">
            <span className="text-zinc-400">예상 용량</span>
            <span className="text-zinc-300">{formatInstallerBytes(component.sizeBytes)}</span>
          </div>

          {/* Dependencies warning */}
          {component.dependencies.length > 0 && (
            <div className="flex items-start gap-2 p-3 bg-amber-500/10 border border-amber-500/20 rounded">
              <AlertTriangle className="w-5 h-5 text-amber-400 flex-shrink-0 mt-0.5" />
              <div className="text-sm">
                <p className="text-amber-300 font-medium">의존성 필요</p>
                <p className="text-amber-400/80">
                  이 구성요소는 다음 항목이 필요합니다:{" "}
                  {component.dependencies.join(", ")}
                </p>
              </div>
            </div>
          )}

          {/* Progress */}
          {isInstalling && installProgress && (
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-zinc-400">{installProgress.message}</span>
                <span className="text-zinc-500">{installProgress.progressPercent}%</span>
              </div>
              <div className="h-1.5 bg-zinc-700 rounded-full overflow-hidden">
                <div
                  className="h-full bg-blue-500 transition-all duration-300"
                  style={{ width: `${installProgress.progressPercent}%` }}
                />
              </div>
            </div>
          )}

          {/* Error */}
          {error && (
            <div className="p-3 bg-red-500/10 border border-red-500/20 rounded text-sm text-red-400">
              {error}
            </div>
          )}

          {/* Success */}
          {success && (
            <div className="flex items-center gap-2 p-3 bg-green-500/10 border border-green-500/20 rounded text-sm text-green-400">
              <Check className="w-5 h-5" />
              설치가 완료되었습니다!
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 px-6 py-4 border-t border-zinc-700">
          <button
            onClick={onClose}
            disabled={isInstalling}
            className="px-4 py-2 text-sm text-zinc-300 hover:bg-zinc-800 rounded transition-colors disabled:opacity-50"
          >
            취소
          </button>
          <button
            onClick={handleInstall}
            disabled={isInstalling || success}
            className="px-4 py-2 text-sm bg-blue-500 hover:bg-blue-600 text-white rounded transition-colors disabled:opacity-50 flex items-center gap-2"
          >
            {isInstalling ? (
              <>
                <Loader2 className="w-4 h-4 animate-spin" />
                설치 중...
              </>
            ) : success ? (
              <>
                <Check className="w-4 h-4" />
                완료
              </>
            ) : (
              "설치 시작"
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
