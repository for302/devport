export interface ApachePortEntry {
  id: string;
  name: string;           // User-defined display name
  port: number;
  domain: string;
  url: string;
  documentRoot: string;
  isSsl: boolean;
  serverAlias: string[];
  configFile: string;
  framework: string;
  hasVhostBlock: boolean;  // true if from <VirtualHost> block, false if Listen-only
  serviceUrl: string | null;   // Actual service URL (user-registered)
  githubUrl: string | null;    // GitHub repo URL (auto-detected)
}

export interface ApacheVHostRequest {
  name: string;           // User-defined display name
  port: number;
  domain: string;
  documentRoot: string;
  serverAlias: string[];
  isSsl: boolean;
  serviceUrl?: string;         // Actual service URL
}
