import { useState, useEffect } from "react";
import { useProcessStore } from "@/stores";

export function useBuildElapsedTime(projectId: string): string | null {
  const startTime = useProcessStore(state => state.buildStartTimes[projectId]);
  const isLoading = useProcessStore(state => state.isLoading[projectId] || false);
  const [elapsed, setElapsed] = useState<string | null>(null);

  useEffect(() => {
    if (!isLoading || !startTime) {
      setElapsed(null);
      return;
    }

    const update = () => {
      const diff = Math.floor((Date.now() - startTime) / 1000);
      const m = Math.floor(diff / 60);
      const s = diff % 60;
      setElapsed(`${m}:${s.toString().padStart(2, "0")}`);
    };

    update();
    const interval = setInterval(update, 1000);
    return () => clearInterval(interval);
  }, [isLoading, startTime]);

  return elapsed;
}
