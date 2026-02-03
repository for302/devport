export interface ApachePortEntry {
  id: string;
  port: number;
  domain: string;
  url: string;
  documentRoot: string;
  isSsl: boolean;
  serverAlias: string[];
  configFile: string;
  framework: string;
}

export interface ApacheVHostRequest {
  port: number;
  domain: string;
  documentRoot: string;
  serverAlias: string[];
  isSsl: boolean;
}
