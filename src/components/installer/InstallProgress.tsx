import { Check, AlertCircle, Loader2, Download, Archive, Settings, Shield, Terminal } from "lucide-react";
import { useRef, useEffect } from "react";
import type { InstallProgress as InstallProgressType, InstallerDownloadProgress, InstallPhase } from "@/types";
import { formatInstallerBytes, formatSpeed, formatEta, INSTALL_PHASE_NAMES, DOWNLOAD_STATUS_NAMES } from "@/types/installer";

export interface InstallLog {
  timestamp: string;
  level: string;
  message: string;
}

interface InstallProgressProps {
  installProgress: InstallProgressType | null;
  downloadProgress: InstallerDownloadProgress | null;
  overallProgress?: number;
  totalCount?: number;
  completedCount?: number;
  logs?: InstallLog[];
}

const PHASE_ICONS: Record<InstallPhase, React.ElementType> = {
  pending: Loader2,
  downloading: Download,
  extracting: Archive,
  configuring: Settings,
  verifying: Shield,
  completed: Check,
  failed: AlertCircle,
};

const PHASE_COLORS: Record<InstallPhase, string> = {
  pending: "text-zinc-400",
  downloading: "text-blue-400",
  extracting: "text-yellow-400",
  configuring: "text-purple-400",
  verifying: "text-cyan-400",
  completed: "text-green-400",
  failed: "text-red-400",
};

export function InstallProgress({
  installProgress,
  downloadProgress,
  overallProgress = 0,
  totalCount = 0,
  completedCount = 0,
  logs = [],
}: InstallProgressProps) {
  const logContainerRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
    }
  }, [logs]);

  return (
    <div className="space-y-6">
      {/* Overall progress */}
      <div className="space-y-2">
        <div className="flex items-center justify-between text-sm">
          <span className="text-zinc-300">전체 진행률</span>
          <span className="text-zinc-400">
            {completedCount} / {totalCount} 완료
          </span>
        </div>
        <div className="h-2 bg-zinc-700 rounded-full overflow-hidden">
          <div
            className="h-full bg-blue-500 transition-all duration-300"
            style={{ width: `${overallProgress}%` }}
          />
        </div>
        <div className="text-xs text-zinc-500 text-right">
          {overallProgress}%
        </div>
      </div>

      {/* Current component progress */}
      {installProgress && (
        <CurrentComponentProgress progress={installProgress} />
      )}

      {/* Download progress */}
      {downloadProgress && downloadProgress.status === "downloading" && (
        <DownloadProgressBar progress={downloadProgress} />
      )}

      {/* Log panel */}
      {logs.length > 0 && (
        <div className="border border-zinc-700 rounded-lg overflow-hidden">
          <div className="px-3 py-2 bg-zinc-800/50 border-b border-zinc-700 flex items-center gap-2">
            <Terminal className="w-4 h-4 text-zinc-400" />
            <span className="text-sm font-medium text-zinc-300">설치 로그</span>
          </div>
          <div
            ref={logContainerRef}
            className="p-3 max-h-48 overflow-y-auto bg-zinc-900 font-mono text-xs"
          >
            {logs.map((log, index) => (
              <div key={index} className="flex gap-2 py-0.5">
                <span className="text-zinc-500 flex-shrink-0">{log.timestamp}</span>
                <span
                  className={`flex-shrink-0 ${
                    log.level === "error"
                      ? "text-red-400"
                      : log.level === "success"
                      ? "text-green-400"
                      : log.level === "warning"
                      ? "text-yellow-400"
                      : "text-zinc-400"
                  }`}
                >
                  [{log.level.toUpperCase()}]
                </span>
                <span className="text-zinc-300 break-all">{log.message}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

interface CurrentComponentProgressProps {
  progress: InstallProgressType;
}

function CurrentComponentProgress({ progress }: CurrentComponentProgressProps) {
  const Icon = PHASE_ICONS[progress.phase];
  const colorClass = PHASE_COLORS[progress.phase];
  const isAnimating = progress.phase !== "completed" && progress.phase !== "failed";

  return (
    <div className="p-4 bg-zinc-800/50 rounded-lg border border-zinc-700">
      <div className="flex items-center gap-3 mb-3">
        <div className={`p-2 rounded-lg bg-zinc-700 ${colorClass}`}>
          <Icon className={`w-5 h-5 ${isAnimating ? "animate-spin" : ""}`} />
        </div>
        <div>
          <h3 className="font-medium text-zinc-200">{progress.componentName}</h3>
          <p className="text-sm text-zinc-400">{progress.message}</p>
        </div>
      </div>

      <div className="space-y-2">
        <div className="flex items-center justify-between text-xs">
          <span className={colorClass}>{INSTALL_PHASE_NAMES[progress.phase]}</span>
          <span className="text-zinc-500">{progress.progressPercent}%</span>
        </div>
        <div className="h-1.5 bg-zinc-700 rounded-full overflow-hidden">
          <div
            className={`h-full transition-all duration-300 ${
              progress.phase === "failed" ? "bg-red-500" : "bg-blue-500"
            }`}
            style={{ width: `${progress.progressPercent}%` }}
          />
        </div>
      </div>

      {progress.error && (
        <div className="mt-3 p-2 bg-red-500/10 border border-red-500/20 rounded text-sm text-red-400">
          {progress.error}
        </div>
      )}
    </div>
  );
}

interface DownloadProgressBarProps {
  progress: InstallerDownloadProgress;
}

function DownloadProgressBar({ progress }: DownloadProgressBarProps) {
  return (
    <div className="p-4 bg-zinc-800/50 rounded-lg border border-zinc-700">
      <div className="flex items-center gap-3 mb-3">
        <div className="p-2 rounded-lg bg-zinc-700 text-blue-400">
          <Download className="w-5 h-5 animate-pulse" />
        </div>
        <div className="flex-1">
          <h3 className="font-medium text-zinc-200">{progress.componentName}</h3>
          <p className="text-sm text-zinc-400">
            {DOWNLOAD_STATUS_NAMES[progress.status]}
          </p>
        </div>
      </div>

      <div className="space-y-2">
        <div className="flex items-center justify-between text-xs">
          <span className="text-zinc-400">
            {formatInstallerBytes(progress.downloadedBytes)} / {formatInstallerBytes(progress.totalBytes)}
          </span>
          <span className="text-zinc-500">
            {formatSpeed(progress.speedBytesPerSec)} • 남은 시간: {formatEta(progress.etaSeconds)}
          </span>
        </div>
        <div className="h-1.5 bg-zinc-700 rounded-full overflow-hidden">
          <div
            className="h-full bg-blue-500 transition-all duration-300"
            style={{ width: `${progress.progressPercent}%` }}
          />
        </div>
      </div>

      {progress.error && (
        <div className="mt-3 p-2 bg-red-500/10 border border-red-500/20 rounded text-sm text-red-400">
          {progress.error}
        </div>
      )}
    </div>
  );
}

// Phase timeline component
interface PhaseTimelineProps {
  currentPhase: InstallPhase;
}

export function PhaseTimeline({ currentPhase }: PhaseTimelineProps) {
  const phases: InstallPhase[] = ["downloading", "extracting", "configuring", "verifying", "completed"];
  const currentIndex = phases.indexOf(currentPhase);

  return (
    <div className="flex items-center justify-between">
      {phases.map((phase, index) => {
        const Icon = PHASE_ICONS[phase];
        const isActive = index === currentIndex;
        const isComplete = index < currentIndex;
        const colorClass = isComplete ? "text-green-400" : isActive ? PHASE_COLORS[phase] : "text-zinc-600";

        return (
          <div key={phase} className="flex items-center">
            <div
              className={`flex flex-col items-center ${
                index < phases.length - 1 ? "flex-1" : ""
              }`}
            >
              <div
                className={`w-8 h-8 rounded-full flex items-center justify-center ${
                  isComplete ? "bg-green-500/20" : isActive ? "bg-blue-500/20" : "bg-zinc-800"
                }`}
              >
                <Icon className={`w-4 h-4 ${colorClass} ${isActive ? "animate-pulse" : ""}`} />
              </div>
              <span className={`text-xs mt-1 ${colorClass}`}>
                {INSTALL_PHASE_NAMES[phase]}
              </span>
            </div>
            {index < phases.length - 1 && (
              <div
                className={`h-0.5 flex-1 mx-2 ${
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
