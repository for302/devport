// Service-specific color mappings for badges
export interface ServiceColor {
  bg: string;
  text: string;
  border: string;
}

const SERVICE_COLORS: Record<string, ServiceColor> = {
  // Web Servers
  apache: { bg: "bg-orange-500/20", text: "text-orange-400", border: "border-orange-500/30" },
  nginx: { bg: "bg-green-500/20", text: "text-green-400", border: "border-green-500/30" },

  // Databases
  mariadb: { bg: "bg-cyan-500/20", text: "text-cyan-400", border: "border-cyan-500/30" },
  mysql: { bg: "bg-blue-500/20", text: "text-blue-400", border: "border-blue-500/30" },
  postgresql: { bg: "bg-indigo-500/20", text: "text-indigo-400", border: "border-indigo-500/30" },
  mongodb: { bg: "bg-emerald-500/20", text: "text-emerald-400", border: "border-emerald-500/30" },
  redis: { bg: "bg-red-500/20", text: "text-red-400", border: "border-red-500/30" },

  // Runtimes
  php: { bg: "bg-violet-500/20", text: "text-violet-400", border: "border-violet-500/30" },
  node: { bg: "bg-lime-500/20", text: "text-lime-400", border: "border-lime-500/30" },
  nodejs: { bg: "bg-lime-500/20", text: "text-lime-400", border: "border-lime-500/30" },
  python: { bg: "bg-yellow-500/20", text: "text-yellow-400", border: "border-yellow-500/30" },
  java: { bg: "bg-amber-500/20", text: "text-amber-400", border: "border-amber-500/30" },

  // Tools
  phpmyadmin: { bg: "bg-orange-500/20", text: "text-orange-300", border: "border-orange-500/30" },
  composer: { bg: "bg-amber-500/20", text: "text-amber-400", border: "border-amber-500/30" },
  npm: { bg: "bg-red-500/20", text: "text-red-400", border: "border-red-500/30" },
  yarn: { bg: "bg-sky-500/20", text: "text-sky-400", border: "border-sky-500/30" },
  git: { bg: "bg-orange-600/20", text: "text-orange-500", border: "border-orange-600/30" },
  docker: { bg: "bg-blue-500/20", text: "text-blue-400", border: "border-blue-500/30" },
};

// Default color for unknown services
const DEFAULT_COLOR: ServiceColor = {
  bg: "bg-slate-500/20",
  text: "text-slate-400",
  border: "border-slate-500/30",
};

export function getServiceColor(serviceId: string): ServiceColor {
  const lowerId = serviceId.toLowerCase();

  // Try exact match first
  if (SERVICE_COLORS[lowerId]) {
    return SERVICE_COLORS[lowerId];
  }

  // Try partial match
  for (const [key, color] of Object.entries(SERVICE_COLORS)) {
    if (lowerId.includes(key) || key.includes(lowerId)) {
      return color;
    }
  }

  return DEFAULT_COLOR;
}

// Get display name for service (capitalize first letter)
export function getServiceDisplayName(serviceId: string): string {
  // Map of special display names
  const displayNames: Record<string, string> = {
    phpmyadmin: "phpMyAdmin",
    mariadb: "MariaDB",
    mysql: "MySQL",
    postgresql: "PostgreSQL",
    mongodb: "MongoDB",
    nodejs: "Node.js",
  };

  const lowerId = serviceId.toLowerCase();
  if (displayNames[lowerId]) {
    return displayNames[lowerId];
  }

  // Default: capitalize first letter
  return serviceId.charAt(0).toUpperCase() + serviceId.slice(1).toLowerCase();
}
