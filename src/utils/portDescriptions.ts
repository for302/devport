// Well-known 포트 매핑
const WELL_KNOWN_PORTS: Record<number, { name: string; description: string }> = {
  21: { name: "FTP", description: "File Transfer Protocol" },
  22: { name: "SSH", description: "Secure Shell" },
  23: { name: "Telnet", description: "Telnet Protocol" },
  25: { name: "SMTP", description: "Simple Mail Transfer Protocol" },
  53: { name: "DNS", description: "Domain Name System" },
  80: { name: "HTTP", description: "Web Server" },
  110: { name: "POP3", description: "Post Office Protocol" },
  143: { name: "IMAP", description: "Internet Message Access Protocol" },
  443: { name: "HTTPS", description: "Secure Web Server" },
  445: { name: "SMB", description: "Windows File Sharing" },
  993: { name: "IMAPS", description: "Secure IMAP" },
  995: { name: "POP3S", description: "Secure POP3" },
  1433: { name: "MSSQL", description: "Microsoft SQL Server" },
  1521: { name: "Oracle", description: "Oracle Database" },
  3000: { name: "Dev Server", description: "Common development port" },
  3306: { name: "MySQL", description: "MySQL/MariaDB Database" },
  5432: { name: "PostgreSQL", description: "PostgreSQL Database" },
  5672: { name: "RabbitMQ", description: "Message Queue" },
  6379: { name: "Redis", description: "Redis Cache/Database" },
  8080: { name: "HTTP Alt", description: "Alternative HTTP / Proxy" },
  8443: { name: "HTTPS Alt", description: "Alternative HTTPS" },
  9200: { name: "Elasticsearch", description: "Elasticsearch" },
  27017: { name: "MongoDB", description: "MongoDB Database" },
};

// 프로세스명 → 서비스 매핑
const PROCESS_SERVICES: Record<string, { name: string; description: string }> = {
  "httpd.exe": { name: "Apache", description: "Apache HTTP Server" },
  "apache.exe": { name: "Apache", description: "Apache HTTP Server" },
  "nginx.exe": { name: "Nginx", description: "Nginx Web Server" },
  "mysqld.exe": { name: "MySQL", description: "MySQL Database Server" },
  "mariadbd.exe": { name: "MariaDB", description: "MariaDB Database Server" },
  "postgres.exe": { name: "PostgreSQL", description: "PostgreSQL Database" },
  "mongod.exe": { name: "MongoDB", description: "MongoDB Database" },
  "redis-server.exe": { name: "Redis", description: "Redis Cache" },
  "node.exe": { name: "Node.js", description: "Node.js Application" },
  "php.exe": { name: "PHP", description: "PHP Application" },
  "php-cgi.exe": { name: "PHP-CGI", description: "PHP FastCGI" },
  "python.exe": { name: "Python", description: "Python Application" },
  "pythonw.exe": { name: "Python", description: "Python Application" },
  "java.exe": { name: "Java", description: "Java Application" },
  "javaw.exe": { name: "Java", description: "Java Application" },
  "Code.exe": { name: "VS Code", description: "Visual Studio Code" },
  "docker.exe": { name: "Docker", description: "Docker Engine" },
  "dockerd.exe": { name: "Docker", description: "Docker Daemon" },
  "com.docker.backend.exe": { name: "Docker", description: "Docker Desktop" },
};

export interface PortDescription {
  name: string;
  description: string;
}

export function getPortDescription(port: number, processName?: string | null): PortDescription | null {
  // 1. 프로세스명으로 먼저 시도 (더 정확함)
  if (processName) {
    const lowerName = processName.toLowerCase();
    for (const [key, value] of Object.entries(PROCESS_SERVICES)) {
      if (lowerName === key.toLowerCase() || lowerName.includes(key.replace('.exe', '').toLowerCase())) {
        return value;
      }
    }
  }

  // 2. Well-known 포트로 폴백
  return WELL_KNOWN_PORTS[port] || null;
}
