import { useState, useEffect } from "react";
import {
  ChevronRight,
  Check,
  Loader2,
  AlertCircle,
  Package,
  RefreshCw,
  Search,
  HardDrive,
} from "lucide-react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { PresetSelector } from "../installer/PresetSelector";
import { ComponentList } from "../installer/ComponentList";
import { InstallProgress, type InstallLog } from "../installer/InstallProgress";
import { useInstallerStore } from "@/stores";
import { formatInstallerBytes } from "@/types/installer";
import { getServiceColor, getServiceDisplayName } from "@/utils/serviceColors";

type InstallerStep = "overview" | "preset" | "components" | "install" | "complete";

export function InstallerView() {
  const [currentStep, setCurrentStep] = useState<InstallerStep>("overview");
  const [isScanning, setIsScanning] = useState(true);
  const [_isInstalling, setIsInstalling] = useState(false);
  const [installError, setInstallError] = useState<string | null>(null);
  const [installLogs, setInstallLogs] = useState<InstallLog[]>([]);

  const {
    presets,
    categoryGroups,
    selectedPresetId,
    selectedComponentIds,
    installProgress,
    downloadProgress,
    installationState,
    isLoading,
    error,
    fetchAll,
    selectPreset,
    toggleComponent,
    installSelected,
    calculateSelectionSize,
    setupEventListeners,
    cleanupEventListeners,
  } = useInstallerStore();

  const [totalSize, setTotalSize] = useState(0);

  useEffect(() => {
    const init = async () => {
      setIsScanning(true);
      await fetchAll();
      setIsScanning(false);
    };
    init();
    setupEventListeners();

    // Listen for install logs
    let unlistenLog: UnlistenFn | null = null;
    listen<InstallLog>("install-log", (event) => {
      setInstallLogs((prev) => [...prev, event.payload]);
    }).then((unlisten) => {
      unlistenLog = unlisten;
    });

    return () => {
      cleanupEventListeners();
      if (unlistenLog) {
        unlistenLog();
      }
    };
  }, []);

  useEffect(() => {
    if (selectedComponentIds.length > 0) {
      calculateSelectionSize(selectedComponentIds).then(setTotalSize);
    } else {
      setTotalSize(0);
    }
  }, [selectedComponentIds]);

  const handlePresetSelect = async (presetId: string) => {
    await selectPreset(presetId);
  };

  const handleInstall = async () => {
    setCurrentStep("install");
    setIsInstalling(true);
    setInstallError(null);
    setInstallLogs([]);

    try {
      await installSelected();
      setCurrentStep("complete");
    } catch (err) {
      setInstallError(String(err));
    } finally {
      setIsInstalling(false);
    }
  };

  const handleReset = async () => {
    setCurrentStep("overview");
    setInstallError(null);
    setInstallLogs([]);
    setIsScanning(true);
    await fetchAll();
    setIsScanning(false);
  };

  // Collect all installed components from categoryGroups (includes system-detected)
  const allInstalledComponents = categoryGroups.flatMap((group) =>
    group.components.filter((comp) => comp.isInstalled)
  );
  const installedCount = allInstalledComponents.length;
  const hasInstalledComponents = installedCount > 0;

  // Show max badges, rest as "+N"
  const MAX_VISIBLE_BADGES = 4;
  const visibleBadges = allInstalledComponents.slice(0, MAX_VISIBLE_BADGES);
  const remainingCount = Math.max(0, installedCount - MAX_VISIBLE_BADGES);

  if (isScanning || (isLoading && !presets.length)) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] gap-4">
        <div className="relative">
          <Search className="w-12 h-12 text-blue-400" />
          <Loader2 className="w-6 h-6 animate-spin text-blue-400 absolute -bottom-1 -right-1" />
        </div>
        <div className="text-center">
          <p className="text-zinc-200 font-medium">설치된 구성요소 스캔 중...</p>
          <p className="text-zinc-500 text-sm mt-1">
            시스템에서 Apache, PHP, MariaDB 등을 검색하고 있습니다
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-4xl mx-auto">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-3">
            <Package className="w-6 h-6 text-blue-400" />
            <div>
              <h1 className="text-2xl font-bold text-white">개발 환경 설치</h1>
              <p className="text-zinc-400 text-sm">
                Apache, PHP, MariaDB, Node.js 등 개발에 필요한 도구를 설치합니다.
              </p>
            </div>
          </div>
          <button
            onClick={handleReset}
            className="flex items-center gap-2 px-3 py-2 text-sm text-zinc-400 hover:text-white hover:bg-zinc-800 rounded-lg transition-colors"
          >
            <RefreshCw className="w-4 h-4" />
            새로고침
          </button>
        </div>

        {error && (
          <div className="mb-4 p-3 bg-red-500/10 border border-red-500/20 rounded flex items-center gap-2 text-red-400">
            <AlertCircle className="w-5 h-5 flex-shrink-0" />
            {error}
          </div>
        )}

        {/* Installed Components Summary */}
        {hasInstalledComponents && currentStep === "overview" && (
          <div className="mb-6 space-y-4">
            {/* Colorful Badges Row */}
            <div className="p-4 bg-zinc-800/50 border border-zinc-700 rounded-lg">
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                  <HardDrive className="w-5 h-5 text-blue-400" />
                  <h3 className="font-semibold text-zinc-200">
                    감지된 개발 환경
                  </h3>
                </div>
                <span className="text-sm text-zinc-500">
                  {installedCount}개 설치됨
                </span>
              </div>
              <div className="flex flex-wrap items-center gap-2">
                {visibleBadges.map((comp) => {
                  const colors = getServiceColor(comp.id);
                  return (
                    <span
                      key={comp.id}
                      className={`px-3 py-1.5 ${colors.bg} ${colors.text} border ${colors.border} text-sm font-medium rounded-full`}
                    >
                      {getServiceDisplayName(comp.id).toLowerCase()}
                    </span>
                  );
                })}
                {remainingCount > 0 && (
                  <span className="px-3 py-1.5 bg-zinc-700 text-zinc-400 text-sm font-medium rounded-full">
                    +{remainingCount}
                  </span>
                )}
              </div>
            </div>

            {/* Detailed Installed List */}
            <div className="bg-zinc-800/50 border border-zinc-700 rounded-lg overflow-hidden">
              <div className="px-4 py-3 border-b border-zinc-700 flex items-center justify-between">
                <h4 className="text-sm font-medium text-zinc-300">설치된 구성요소 목록</h4>
                <span className="text-xs text-zinc-500">{installedCount}개</span>
              </div>
              <div className="divide-y divide-zinc-700/50 max-h-[200px] overflow-y-auto">
                {allInstalledComponents.map((comp) => {
                  const colors = getServiceColor(comp.id);
                  return (
                    <div
                      key={comp.id}
                      className="px-4 py-2.5 flex items-center justify-between hover:bg-zinc-700/30 transition-colors"
                    >
                      <div className="flex items-center gap-3">
                        <span
                          className={`w-2 h-2 rounded-full ${colors.bg.replace('/20', '')} ${colors.text.replace('text-', 'bg-')}`}
                        />
                        <div>
                          <span className="text-zinc-200 text-sm">
                            {comp.name}
                          </span>
                          {comp.installedVersion && (
                            <span className="text-zinc-500 text-xs ml-2">
                              v{comp.installedVersion}
                            </span>
                          )}
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        {comp.installedSource && (
                          <span className="text-xs text-zinc-500 bg-zinc-700/50 px-2 py-0.5 rounded">
                            {comp.installedSource === "devport" ? "DevPort" : "System"}
                          </span>
                        )}
                        <Check className="w-4 h-4 text-green-500" />
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        )}

        {/* Overview / Start Installation */}
        {currentStep === "overview" && (
          <div className="space-y-6">
            <div className="p-6 bg-zinc-800/50 rounded-lg border border-zinc-700">
              <h2 className="text-lg font-semibold text-white mb-4">
                추가 구성요소 설치
              </h2>
              <p className="text-zinc-400 mb-6">
                아래 버튼을 클릭하여 새로운 개발 도구를 설치하거나 기존 구성요소를 업데이트할 수 있습니다.
              </p>
              <button
                onClick={() => setCurrentStep("preset")}
                className="flex items-center gap-2 px-6 py-3 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors"
              >
                <Package className="w-5 h-5" />
                설치 시작
                <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}

        {/* Preset Selection */}
        {currentStep === "preset" && (
          <div className="space-y-6">
            <PresetSelector
              presets={presets}
              selectedId={selectedPresetId}
              onSelect={handlePresetSelect}
            />
            <div className="flex justify-between">
              <button
                onClick={() => setCurrentStep("overview")}
                className="px-4 py-2 text-zinc-400 hover:text-white hover:bg-zinc-800 rounded-lg transition-colors"
              >
                취소
              </button>
              <button
                onClick={() => setCurrentStep("components")}
                disabled={!selectedPresetId}
                className="flex items-center gap-2 px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                다음
                <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}

        {/* Component Selection */}
        {currentStep === "components" && (
          <div className="space-y-6">
            <ComponentList
              categoryGroups={categoryGroups}
              selectedIds={selectedComponentIds}
              onToggle={toggleComponent}
            />
            <div className="flex items-center justify-between">
              <div className="text-sm text-zinc-400">
                {selectedComponentIds.length > 0 && (
                  <>
                    {selectedComponentIds.length}개 선택됨 • 총{" "}
                    {formatInstallerBytes(totalSize)}
                  </>
                )}
              </div>
              <div className="flex gap-3">
                <button
                  onClick={() => setCurrentStep("preset")}
                  className="px-4 py-2 text-zinc-400 hover:text-white hover:bg-zinc-800 rounded-lg transition-colors"
                >
                  이전
                </button>
                <button
                  onClick={handleInstall}
                  disabled={selectedComponentIds.length === 0}
                  className="flex items-center gap-2 px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  설치 시작
                  <ChevronRight className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Installation Progress */}
        {currentStep === "install" && (
          <div className="space-y-6">
            <h2 className="text-lg font-semibold text-zinc-100">설치 진행 중...</h2>
            <InstallProgress
              installProgress={installProgress}
              downloadProgress={downloadProgress}
              overallProgress={installationState?.overallProgress || 0}
              totalCount={installationState?.totalCount || selectedComponentIds.length}
              completedCount={installationState?.completedCount || 0}
              logs={installLogs}
            />
            {installError && (
              <div className="p-3 bg-red-500/10 border border-red-500/20 rounded text-red-400">
                {installError}
              </div>
            )}
          </div>
        )}

        {/* Completion */}
        {currentStep === "complete" && (
          <div className="space-y-6">
            <div className="flex items-center gap-4">
              <div className="w-12 h-12 rounded-full bg-green-500/20 flex items-center justify-center">
                <Check className="w-6 h-6 text-green-400" />
              </div>
              <div>
                <h2 className="text-xl font-semibold text-zinc-100">
                  설치가 완료되었습니다!
                </h2>
                <p className="text-zinc-400 text-sm">
                  {selectedComponentIds.length}개의 구성요소가 설치되었습니다.
                </p>
              </div>
            </div>

            <div className="border border-zinc-700 rounded-lg overflow-hidden">
              <div className="px-4 py-3 bg-zinc-800/50 border-b border-zinc-700">
                <h3 className="font-medium text-zinc-200">설치된 구성요소</h3>
              </div>
              <div className="divide-y divide-zinc-700/50">
                {selectedComponentIds.map((id) => {
                  const group = categoryGroups.find((g) =>
                    g.components.some((c) => c.id === id)
                  );
                  const component = group?.components.find((c) => c.id === id);

                  return (
                    <div
                      key={id}
                      className="px-4 py-3 flex items-center justify-between"
                    >
                      <div className="flex items-center gap-3">
                        <div className="w-5 h-5 rounded-full bg-green-500/20 flex items-center justify-center">
                          <Check className="w-3 h-3 text-green-400" />
                        </div>
                        <div>
                          <span className="text-zinc-200">
                            {component?.name || id}
                          </span>
                          <span className="text-xs text-zinc-500 ml-2">
                            {component?.version}
                          </span>
                        </div>
                      </div>
                      <span className="text-xs text-green-400">설치됨</span>
                    </div>
                  );
                })}
              </div>
            </div>

            <button
              onClick={handleReset}
              className="px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors"
            >
              완료
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
