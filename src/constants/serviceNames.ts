/**
 * Display name mappings for services
 * Used to show user-friendly names in the UI
 */
export const SERVICE_DISPLAY_NAMES: Record<string, string> = {
  apache: "Apache",
  mariadb: "MariaDB",
  mysql: "MySQL",
  nginx: "Nginx",
  php: "PHP",
  phpmyadmin: "phpMyAdmin",
  redis: "Redis",
  memcached: "Memcached",
  postgresql: "PostgreSQL",
  mongodb: "MongoDB",
};

/**
 * Get the display name for a service
 * Falls back to the service ID if no mapping exists
 * @param serviceId - The service identifier
 * @returns User-friendly display name
 */
export function getServiceDisplayName(serviceId: string): string {
  return SERVICE_DISPLAY_NAMES[serviceId.toLowerCase()] || serviceId;
}

/**
 * Service descriptions for tooltips and help text
 */
export const SERVICE_DESCRIPTIONS: Record<string, string> = {
  apache: "HTTP web server",
  mariadb: "MySQL-compatible relational database",
  mysql: "Relational database management system",
  nginx: "High-performance HTTP server and reverse proxy",
  php: "Server-side scripting language",
  phpmyadmin: "Web-based MySQL/MariaDB administration tool",
  redis: "In-memory data structure store",
  memcached: "Distributed memory caching system",
  postgresql: "Object-relational database system",
  mongodb: "Document-oriented NoSQL database",
};
