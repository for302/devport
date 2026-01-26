export interface PortInfo {
  port: number;
  pid: number | null;
  processName: string | null;
  projectId: string | null;
  protocol: string;
  state: string;
  localAddress: string;
}
