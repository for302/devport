import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppShell } from "./components/layout/AppShell";
import { SetupWizard } from "./components/installer/SetupWizard";
import { Loader2 } from "lucide-react";
import type { InstalledComponent } from "./types";
import { useUpdaterStore, useUiStore } from "./stores";
import { initServiceLogListener } from "./stores/serviceStore";

function App() {
  const [isLoading, setIsLoading] = useState(true);
  const [isFirstRun, setIsFirstRun] = useState(false);
  const checkForUpdates = useUpdaterStore((state) => state.checkForUpdates);
  const updateStatus = useUpdaterStore((state) => state.status);
  const isDismissed = useUpdaterStore((state) => state.isDismissed);
  const openModal = useUiStore((state) => state.openModal);

  useEffect(() => {
    checkFirstRun();

    // Initialize service log listener for real-time log streaming
    const unlistenPromise = initServiceLogListener();

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  // Check for updates after app loads (only when not in first run)
  useEffect(() => {
    if (!isLoading && !isFirstRun) {
      // Delay update check to avoid blocking startup
      const timer = setTimeout(() => {
        checkForUpdates();
      }, 3000);
      return () => clearTimeout(timer);
    }
  }, [isLoading, isFirstRun, checkForUpdates]);

  // Open update modal when update is available
  useEffect(() => {
    if (updateStatus === "available" && !isDismissed) {
      openModal("update");
    }
  }, [updateStatus, isDismissed, openModal]);

  const checkFirstRun = async () => {
    try {
      // Check if any components are installed
      const installed = await invoke<InstalledComponent[]>(
        "get_installed_components"
      );

      // If no components are installed, this is a first run
      setIsFirstRun(installed.length === 0);
    } catch (error) {
      // If we can't check, assume first run to show setup wizard
      console.error("Failed to check installed components:", error);
      setIsFirstRun(true);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSetupComplete = () => {
    setIsFirstRun(false);
  };

  // Show loading screen while checking first run status
  if (isLoading) {
    return (
      <div className="flex flex-col items-center justify-center h-screen bg-zinc-900">
        <Loader2 className="w-8 h-8 animate-spin text-blue-400 mb-4" />
        <p className="text-zinc-400">ClickDevPort 로딩 중...</p>
      </div>
    );
  }

  const handleSkipSetup = () => {
    setIsFirstRun(false);
  };

  // Show SetupWizard on first run
  if (isFirstRun) {
    return (
      <div className="h-screen bg-zinc-900">
        <SetupWizard onComplete={handleSetupComplete} onCancel={handleSkipSetup} />
      </div>
    );
  }

  // Show main app
  return <AppShell />;
}

export default App;
