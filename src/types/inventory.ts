import {
  Cpu,
  Globe,
  Database,
  Hammer,
  Layers,
  Package,
  Wrench,
  type LucideIcon,
} from "lucide-react";

export type InventoryCategory =
  | "runtime"
  | "webServer"
  | "database"
  | "buildTool"
  | "framework"
  | "packageManager"
  | "devTool";

export type InstallSource =
  | "system"
  | "devPort"
  | "xampp"
  | "laragon"
  | "wamp"
  | "scoop"
  | "chocolatey"
  | "manual"
  | "unknown";

export interface InventoryItem {
  id: string;
  name: string;
  category: InventoryCategory;
  isInstalled: boolean;
  version: string | null;
  executablePath: string | null;
  installSource: InstallSource;
  isRunning: boolean;
  port: number | null;
}

export interface InventoryResult {
  runtimes: InventoryItem[];
  webServers: InventoryItem[];
  databases: InventoryItem[];
  buildTools: InventoryItem[];
  frameworks: InventoryItem[];
  packageManagers: InventoryItem[];
  devTools: InventoryItem[];
  scannedAt: string;
  scanDurationMs: number;
}

export interface CategoryMeta {
  key: keyof Omit<InventoryResult, "scannedAt" | "scanDurationMs">;
  label: string;
  icon: LucideIcon;
  color: string;
}

export const CATEGORY_META: CategoryMeta[] = [
  {
    key: "webServers",
    label: "Web Servers",
    icon: Globe,
    color: "text-blue-400",
  },
  {
    key: "databases",
    label: "Databases",
    icon: Database,
    color: "text-green-400",
  },
  { key: "runtimes", label: "Runtimes", icon: Cpu, color: "text-yellow-400" },
  {
    key: "buildTools",
    label: "Build Tools",
    icon: Hammer,
    color: "text-orange-400",
  },
  {
    key: "frameworks",
    label: "Frameworks",
    icon: Layers,
    color: "text-purple-400",
  },
  {
    key: "packageManagers",
    label: "Package Managers",
    icon: Package,
    color: "text-cyan-400",
  },
  { key: "devTools", label: "Dev Tools", icon: Wrench, color: "text-pink-400" },
];

export const INSTALL_SOURCE_LABELS: Record<InstallSource, string> = {
  system: "System",
  devPort: "DevPort",
  xampp: "XAMPP",
  laragon: "Laragon",
  wamp: "WampServer",
  scoop: "Scoop",
  chocolatey: "Chocolatey",
  manual: "Manual",
  unknown: "Unknown",
};

export const INSTALL_SOURCE_COLORS: Record<InstallSource, string> = {
  system: "bg-slate-600",
  devPort: "bg-blue-600",
  xampp: "bg-orange-600",
  laragon: "bg-sky-600",
  wamp: "bg-pink-600",
  scoop: "bg-cyan-600",
  chocolatey: "bg-amber-600",
  manual: "bg-slate-500",
  unknown: "bg-slate-700",
};

// Inventory item ID → Service ID mapping
export const INVENTORY_TO_SERVICE_MAP: Record<string, string> = {
  apache: "apache",
  mariadb: "mariadb",
  mysql: "mariadb",
};

// Categories that can be controlled as services
export const CONTROLLABLE_CATEGORIES: InventoryCategory[] = ["webServer", "database"];
