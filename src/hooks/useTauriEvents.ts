import { useEffect } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useLogStore, useProcessStore } from "@/stores";
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

export function useTauriEvents() {
  const addLog = useLogStore((state) => state.addLog);
  const setProcessInfo = useProcessStore((state) => state.setProcessInfo);

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

    // Cleanup
    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [addLog, setProcessInfo]);
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
