import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppShell } from "./components/layout/AppShell";
import { SetupWizard } from "./components/installer/SetupWizard";
import { Loader2 } from "lucide-react";
import type { InstalledComponent } from "./types";

function App() {
  const [isLoading, setIsLoading] = useState(true);
  const [isFirstRun, setIsFirstRun] = useState(false);

  useEffect(() => {
    checkFirstRun();
  }, []);

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
