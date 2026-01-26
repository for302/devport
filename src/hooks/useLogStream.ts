import { useEffect, useRef, useCallback, useState } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { LogEntry } from "@/types";

interface LogUpdatePayload {
  source: string;
  entries: LogEntry[];
}

interface UseLogStreamOptions {
  serviceId?: string;
  projectName?: string;
  logType: string;
  maxEntries?: number;
  onError?: (error: Error) => void;
}

interface UseLogStreamResult {
  entries: LogEntry[];
  isStreaming: boolean;
  startStream: () => Promise<void>;
  stopStream: () => Promise<void>;
  clearEntries: () => void;
}

export function useLogStream({
  serviceId,
  projectName,
  logType,
  maxEntries = 1000,
  onError,
}: UseLogStreamOptions): UseLogStreamResult {
  const [entries, setEntries] = useState<LogEntry[]>([]);
  const [isStreaming, setIsStreaming] = useState(false);
  const unlistenRef = useRef<UnlistenFn | null>(null);
  const sourceId = serviceId || projectName || "";

  // Add new entries to the log
  const addEntries = useCallback(
    (newEntries: LogEntry[]) => {
      setEntries((prev) => {
        const combined = [...prev, ...newEntries];
        // Keep only the last maxEntries
        if (combined.length > maxEntries) {
          return combined.slice(-maxEntries);
        }
        return combined;
      });
    },
    [maxEntries]
  );

  // Start streaming logs
  const startStream = useCallback(async () => {
    if (isStreaming) return;

    try {
      // Set up the event listener first
      const unlisten = await listen<LogUpdatePayload>("log-update", (event) => {
        const payload = event.payload;
        // Only process events for our source
        if (payload.source === sourceId) {
          addEntries(payload.entries);
        }
      });
      unlistenRef.current = unlisten;

      // Start the backend stream
      const started = await invoke<boolean>("start_log_stream", {
        serviceId: serviceId || null,
        projectName: projectName || null,
        logType,
      });

      if (started) {
        setIsStreaming(true);
      } else {
        // Stream was already active or failed to start
        unlisten();
        unlistenRef.current = null;
      }
    } catch (error) {
      const err = error instanceof Error ? error : new Error(String(error));
      onError?.(err);
      console.error("Failed to start log stream:", err);
    }
  }, [serviceId, projectName, logType, sourceId, isStreaming, addEntries, onError]);

  // Stop streaming logs
  const stopStream = useCallback(async () => {
    if (!isStreaming) return;

    try {
      // Stop the backend stream
      await invoke<boolean>("stop_log_stream", {
        serviceId: serviceId || null,
        projectName: projectName || null,
        logType,
      });

      // Clean up the event listener
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }

      setIsStreaming(false);
    } catch (error) {
      const err = error instanceof Error ? error : new Error(String(error));
      onError?.(err);
      console.error("Failed to stop log stream:", err);
    }
  }, [serviceId, projectName, logType, isStreaming, onError]);

  // Clear all entries
  const clearEntries = useCallback(() => {
    setEntries([]);
  }, []);

  // Cleanup on unmount or when dependencies change
  useEffect(() => {
    return () => {
      // Stop stream and clean up listener on unmount
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }
      // Fire and forget the stop command
      if (isStreaming) {
        invoke("stop_log_stream", {
          serviceId: serviceId || null,
          projectName: projectName || null,
          logType,
        }).catch(() => {
          // Ignore errors during cleanup
        });
      }
    };
  }, [serviceId, projectName, logType, isStreaming]);

  return {
    entries,
    isStreaming,
    startStream,
    stopStream,
    clearEntries,
  };
}
