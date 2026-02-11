import { useEffect } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useLogStore, useProcessStore, useActivityLogStore, useProjectStore } from "@/stores";
import type { ProcessLog } from "@/types";

interface ProcessStartedPayload {
  projectId: string;
  pid: number;
}

interface ProcessStoppedPayload {
  projectId: string;
}

interface ProcessLogPayload {
  projectId: string;
  line: string;
  stream: "stdout" | "stderr";
}

interface BuildStatusPayload {
  projectId: string;
  status: "starting" | "compiling" | "compiled" | "launched" | "error" | "progress";
  message?: string;
}

export function useTauriEvents() {
  const addLog = useLogStore((state) => state.addLog);
  const setProcessInfo = useProcessStore((state) => state.setProcessInfo);
  const setBuildStatus = useProcessStore((state) => state.setBuildStatus);
  const addActivityLog = useActivityLogStore((state) => state.addLog);

  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];

    // Listen for process started events
    listen<ProcessStartedPayload>("process-started", (event) => {
      const { projectId, pid } = event.payload;
      setProcessInfo(projectId, {
        projectId,
        pid,
        status: "running",
        startedAt: new Date().toISOString(),
        port: 0,
        cpuUsage: null,
        memoryUsage: null,
      });
    }).then((unlisten) => unlisteners.push(unlisten));

    // Listen for process stopped events
    listen<ProcessStoppedPayload>("process-stopped", (event) => {
      const { projectId } = event.payload;
      setProcessInfo(projectId, null);
      useProcessStore.getState().clearBuildLoading(projectId);
    }).then((unlisten) => unlisteners.push(unlisten));

    // Listen for process log events
    listen<ProcessLogPayload>("process-log", (event) => {
      const { projectId, line, stream } = event.payload;
      const log: ProcessLog = {
        projectId,
        line,
        stream,
        timestamp: new Date().toISOString(),
      };
      addLog(log);
    }).then((unlisten) => unlisteners.push(unlisten));

    // Listen for build status events
    listen<BuildStatusPayload>("build-status", (event) => {
      const { projectId, status, message } = event.payload;

      // "progress" is informational only — don't overwrite build status state
      if (status !== "progress") {
        setBuildStatus(projectId, status);
      }

      const project = useProjectStore.getState().getProjectById(projectId);
      const name = project?.name ?? projectId;

      switch (status) {
        case "starting":
          addActivityLog(name, "Starting project...", "info");
          break;
        case "compiling":
          addActivityLog(name, message || "Building...", "warning");
          break;
        case "compiled":
          addActivityLog(name, message || "Build complete", "success");
          break;
        case "launched":
          addActivityLog(name, message ? `Launched — ${message}` : "App launched", "success");
          useProcessStore.getState().clearBuildLoading(projectId);
          break;
        case "error":
          addActivityLog(name, message || "Build error — check Logs tab", "error");
          useProcessStore.getState().clearBuildLoading(projectId);
          break;
        case "progress":
          if (message) {
            addActivityLog(name, message, "info");
          }
          break;
      }
    }).then((unlisten) => unlisteners.push(unlisten));

    // Cleanup
    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [addLog, setProcessInfo, setBuildStatus, addActivityLog]);
}

export function useHealthCheck(projectId: string, url: string | null, interval = 30000) {
  const checkHealth = useProcessStore((state) => state.checkHealth);
  const isRunning = useProcessStore((state) => state.isProjectRunning(projectId));

  useEffect(() => {
    if (!isRunning || !url) return;

    // Initial check
    checkHealth(projectId, url);

    // Set up interval
    const timer = setInterval(() => {
      checkHealth(projectId, url);
    }, interval);

    return () => clearInterval(timer);
  }, [projectId, url, isRunning, interval, checkHealth]);
}
