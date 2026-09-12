/**
 * Core type definitions for KAIRO.
 * Generic service representations across frontend and backend IPC.
 */

export type ServiceStatus =
  | "stopped"
  | "starting"
  | "running"
  | "ready"
  | "stopping"
  | "restarting"
  | "failed"
  | "unknown";

export interface ProcessRuntimeInfo {
  serviceId: string;
  pid?: number | null;
  state: ServiceStatus;
  startedAt?: string | null;
  errorMessage?: string | null;
  port?: number | null;
  portListening?: boolean | null;
}

export interface ServiceConfig {
  id: string;
  name: string;
  description: string;
  executable: string;
  arguments: string[];
  workingDirectory: string;
  environmentVariables: Record<string, string>;
  port?: number | null;
  autoStart: boolean;
  autoRestart: boolean;
  healthCheck?: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface ProcessMetrics {
  pid?: number;
  cpuPercent?: number;
  memoryBytes?: number;
  uptimeSeconds?: number;
}

export interface ServiceState {
  config: ServiceConfig;
  status: ServiceStatus;
  metrics?: ProcessMetrics;
  lastStartedAt?: string;
  lastExitedAt?: string;
  exitCode?: number;
  errorMessage?: string;
}

export type NavigationPage = "dashboard" | "services" | "logs" | "settings";
export type StatusFilter = "all" | "running" | "stopped" | "failed";

// ─── Phase 6: Startup Settings ───────────────────────────────────────────────

/** Application-level startup settings persisted in settings.json. */
export interface AppSettings {
  appAutoStart: boolean;
  serviceAutoStart: boolean;
  updatedAt: string;
}

/** Live progress payload emitted during service auto-start sequencing. */
export interface StartupProgress {
  total: number;
  started: number;
  failed: number;
  isComplete: boolean;
}

// ─── Phase 9: Logs ───────────────────────────────────────────────────────────

export type LogStream = "stdout" | "stderr";

export interface LogEntry {
  id: string;
  serviceId: string;
  timestamp: string;
  stream: LogStream;
  message: string;
}

export type LogStreamFilter = "all" | "stdout" | "stderr";

