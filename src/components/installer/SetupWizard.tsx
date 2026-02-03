import { useState, useEffect } from "react";
import {
  ChevronLeft,
  ChevronRight,
  Check,
  Loader2,
  AlertCircle,
} from "lucide-react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { PresetSelector } from "./PresetSelector";
import { ComponentList } from "./ComponentList";
import { InstallProgress, type InstallLog } from "./InstallProgress";
import { useInstallerStore } from "@/stores";
import { formatInstallerBytes } from "@/types/installer";

type WizardStep = "preset" | "components" | "install" | "complete";

interface SetupWizardProps {
  onComplete?: () => void;
  onCancel?: () => void;
}

export function SetupWizard({ onComplete, onCancel }: SetupWizardProps) {
  const [currentStep, setCurrentStep] = useState<WizardStep>("preset");
  const [isInstalling, setIsInstalling] = useState(false);
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
    fetchAll();
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
    setInstallLogs([]); // Clear previous logs

    try {
      await installSelected();
      setCurrentStep("complete");
    } catch (err) {
      setInstallError(String(err));
    } finally {
      setIsInstalling(false);
    }
  };

  const canProceed = () => {
    switch (currentStep) {
      case "preset":
        return selectedPresetId !== null;
      case "components":
        return selectedComponentIds.length > 0;
      case "install":
        return !isInstalling;
      case "complete":
        return true;
      default:
        return false;
    }
  };

  const handleNext = () => {
    switch (currentStep) {
      case "preset":
        setCurrentStep("components");
        break;
      case "components":
        handleInstall();
        break;
      case "complete":
        onComplete?.();
        break;
    }
  };

  const handleBack = () => {
    switch (currentStep) {
      case "components":
        setCurrentStep("preset");
        break;
      case "install":
        if (!isInstalling) {
          setCurrentStep("components");
        }
        break;
    }
  };

  if (isLoading && !presets.length) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] gap-4">
        <Loader2 className="w-8 h-8 animate-spin text-blue-400" />
        <p className="text-zinc-400">설치 정보를 불러오는 중...</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-700">
        <div className="flex items-center gap-3">
          <img src="/icon.png" alt="ClickDevPort" className="w-6 h-6" />
          <h1 className="text-xl font-semibold text-zinc-100">
            ClickDevPort 설치
          </h1>
        </div>
        <StepIndicator currentStep={currentStep} />
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto p-6">
        {error && (
          <div className="mb-4 p-3 bg-red-500/10 border border-red-500/20 rounded flex items-center gap-2 text-red-400">
            <AlertCircle className="w-5 h-5 flex-shrink-0" />
            {error}
          </div>
        )}

        {currentStep === "preset" && (
          <PresetSelector
            presets={presets}
            selectedId={selectedPresetId}
            onSelect={handlePresetSelect}
          />
        )}

        {currentStep === "components" && (
          <ComponentList
            categoryGroups={categoryGroups}
            selectedIds={selectedComponentIds}
            onToggle={toggleComponent}
          />
        )}

        {currentStep === "install" && (
          <div className="space-y-6">
            <h2 className="text-lg font-semibold text-zinc-100">
              설치 진행 중...
            </h2>
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
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="flex items-center justify-between px-6 py-4 border-t border-zinc-700">
        <div className="text-sm text-zinc-400">
          {currentStep === "components" && selectedComponentIds.length > 0 && (
            <>
              {selectedComponentIds.length}개 선택됨 • 총{" "}
              {formatInstallerBytes(totalSize)}
            </>
          )}
        </div>

        <div className="flex items-center gap-3">
          {currentStep !== "preset" && currentStep !== "complete" && (
            <button
              onClick={handleBack}
              disabled={isInstalling}
              className="flex items-center gap-2 px-4 py-2 text-sm text-zinc-300 hover:bg-zinc-800 rounded transition-colors disabled:opacity-50"
            >
              <ChevronLeft className="w-4 h-4" />
              이전
            </button>
          )}

          {currentStep === "preset" && onCancel && (
            <button
              onClick={onCancel}
              className="px-4 py-2 text-sm text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 rounded transition-colors"
            >
              건너뛰기 (나중에 설치)
            </button>
          )}

          {currentStep !== "install" && (
            <button
              onClick={handleNext}
              disabled={!canProceed()}
              className="flex items-center gap-2 px-4 py-2 text-sm bg-blue-500 hover:bg-blue-600 text-white rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {currentStep === "complete" ? (
                "완료"
              ) : currentStep === "components" ? (
                <>
                  설치 시작
                  <ChevronRight className="w-4 h-4" />
                </>
              ) : (
                <>
                  다음
                  <ChevronRight className="w-4 h-4" />
                </>
              )}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

interface StepIndicatorProps {
  currentStep: WizardStep;
}

function StepIndicator({ currentStep }: StepIndicatorProps) {
  const steps: { key: WizardStep; label: string }[] = [
    { key: "preset", label: "프리셋 선택" },
    { key: "components", label: "구성요소" },
    { key: "install", label: "설치" },
    { key: "complete", label: "완료" },
  ];

  const currentIndex = steps.findIndex((s) => s.key === currentStep);

  return (
    <div className="flex items-center gap-2">
      {steps.map((step, index) => {
        const isActive = index === currentIndex;
        const isComplete = index < currentIndex;

        return (
          <div key={step.key} className="flex items-center">
            <div
              className={`flex items-center gap-2 px-3 py-1 rounded-full text-sm ${
                isActive
                  ? "bg-blue-500/20 text-blue-400"
                  : isComplete
                  ? "bg-green-500/20 text-green-400"
                  : "bg-zinc-800 text-zinc-500"
              }`}
            >
              {isComplete ? (
                <Check className="w-4 h-4" />
              ) : (
                <span className="w-4 h-4 flex items-center justify-center text-xs">
                  {index + 1}
                </span>
              )}
              <span className="hidden sm:inline">{step.label}</span>
            </div>
            {index < steps.length - 1 && (
              <div
                className={`w-8 h-0.5 mx-1 ${
                  isComplete ? "bg-green-500" : "bg-zinc-700"
                }`}
              />
            )}
          </div>
        );
      })}
    </div>
  );
}
