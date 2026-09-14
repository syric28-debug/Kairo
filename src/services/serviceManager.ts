import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ServiceConfig, ProcessRuntimeInfo } from "../types";

export interface CreateServiceDto {
  name: string;
  description?: string;
  executable: string;
  arguments?: string[];
  workingDirectory?: string;
  environmentVariables?: Record<string, string>;
  port?: number | null;
  autoStart?: boolean;
  autoRestart?: boolean;
  healthCheck?: string | null;
  /** Optional user-defined API base path (e.g. "/v1"). */
  apiBasePath?: string | null;
  /** Optional user-defined direct-link path (e.g. "/"). */
  directUrlPath?: string | null;
  /** Optional user-defined health-check path (e.g. "/health"). */
  healthCheckPath?: string | null;
}

export interface UpdateServiceDto {
  name: string;
  description?: string;
  executable: string;
  arguments?: string[];
  workingDirectory?: string;
  environmentVariables?: Record<string, string>;
  port?: number | null;
  autoStart?: boolean;
  autoRestart?: boolean;
  healthCheck?: string | null;
  /** Optional user-defined API base path (e.g. "/v1"). */
  apiBasePath?: string | null;
  /** Optional user-defined direct-link path (e.g. "/"). */
  directUrlPath?: string | null;
  /** Optional user-defined health-check path (e.g. "/health"). */
  healthCheckPath?: string | null;
}

const STORAGE_KEY = "lsm_services_fallback";

function isTauri(): boolean {
  return (
    typeof window !== "undefined" &&
    Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__)
  );
}

function getFallbackServices(): ServiceConfig[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return [];
    }
    return JSON.parse(raw);
  } catch {
    return [];
  }
}

function saveFallbackServices(services: ServiceConfig[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(services));
  } catch (err) {
    console.error("Failed to save fallback services to localStorage", err);
  }
}

/**
 * Lists all configured services from persistent storage.
 */
export async function listServices(): Promise<ServiceConfig[]> {
  if (isTauri()) {
    return invoke<ServiceConfig[]>("list_services");
  }
  return getFallbackServices();
}

/**
 * Gets a single service by ID.
 */
export async function getService(id: string): Promise<ServiceConfig> {
  if (isTauri()) {
    return invoke<ServiceConfig>("get_service", { id });
  }
  const services = getFallbackServices();
  const found = services.find((s) => s.id === id);
  if (!found) {
    throw new Error(`Service with id '${id}' not found`);
  }
  return found;
}

/**
 * Validates service inputs on frontend before submission.
 */
export function validateServiceInput(dto: {
  name: string;
  executable: string;
  description?: string;
  port?: number | null;
  workingDirectory?: string;
}): { valid: boolean; errors: Record<string, string> } {
  const errors: Record<string, string> = {};

  const trimmedName = dto.name.trim();
  if (!trimmedName) {
    errors.name = "Service name is required";
  } else if (trimmedName.length > 100) {
    errors.name = "Service name cannot exceed 100 characters";
  }

  const trimmedExec = dto.executable.trim();
  if (!trimmedExec) {
    errors.executable = "Executable path or command is required";
  } else if (trimmedExec.length > 1024) {
    errors.executable = "Executable path cannot exceed 1024 characters";
  }

  if (dto.description && dto.description.length > 500) {
    errors.description = "Description cannot exceed 500 characters";
  }

  if (dto.workingDirectory && dto.workingDirectory.length > 1024) {
    errors.workingDirectory = "Working directory cannot exceed 1024 characters";
  }

  if (dto.port !== undefined && dto.port !== null) {
    if (isNaN(dto.port) || dto.port < 1 || dto.port > 65535) {
      errors.port = "Port must be an integer between 1 and 65535";
    }
  }

  return {
    valid: Object.keys(errors).length === 0,
    errors,
  };
}

/**
 * Creates a new generic service in persistent storage.
 */
export async function createService(dto: CreateServiceDto): Promise<ServiceConfig> {
  const validation = validateServiceInput(dto);
  if (!validation.valid) {
    const msg = Object.values(validation.errors).join(", ");
    throw new Error(msg);
  }

  if (isTauri()) {
    return invoke<ServiceConfig>("create_service", { dto });
  }

  const services = getFallbackServices();
  const now = new Date().toISOString();
  const newService: ServiceConfig = {
    id: `svc-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`,
    name: dto.name.trim(),
    description: dto.description?.trim() || "",
    executable: dto.executable.trim(),
    arguments: dto.arguments || [],
    workingDirectory: dto.workingDirectory?.trim() || "C:\\",
    environmentVariables: dto.environmentVariables || {},
    port: dto.port ?? null,
    autoStart: dto.autoStart ?? false,
    autoRestart: dto.autoRestart ?? false,
    healthCheck: dto.healthCheck ?? null,
    apiBasePath: dto.apiBasePath?.trim() || null,
    directUrlPath: dto.directUrlPath?.trim() || null,
    healthCheckPath: dto.healthCheckPath?.trim() || null,
    createdAt: now,
    updatedAt: now,
  };

  services.push(newService);
  saveFallbackServices(services);
  return newService;
}

/**
 * Updates an existing service configuration.
 */
export async function updateService(
  id: string,
  dto: UpdateServiceDto
): Promise<ServiceConfig> {
  const validation = validateServiceInput(dto);
  if (!validation.valid) {
    const msg = Object.values(validation.errors).join(", ");
    throw new Error(msg);
  }

  if (isTauri()) {
    return invoke<ServiceConfig>("update_service", { id, dto });
  }

  const services = getFallbackServices();
  const index = services.findIndex((s) => s.id === id);
  if (index === -1) {
    throw new Error(`Service with id '${id}' does not exist`);
  }

  const updated: ServiceConfig = {
    ...services[index],
    name: dto.name.trim(),
    description: dto.description?.trim() || "",
    executable: dto.executable.trim(),
    arguments: dto.arguments || [],
    workingDirectory: dto.workingDirectory?.trim() || "C:\\",
    environmentVariables: dto.environmentVariables || {},
    port: dto.port ?? null,
    autoStart: dto.autoStart ?? false,
    autoRestart: dto.autoRestart ?? false,
    healthCheck: dto.healthCheck ?? null,
    apiBasePath: dto.apiBasePath?.trim() || null,
    directUrlPath: dto.directUrlPath?.trim() || null,
    healthCheckPath: dto.healthCheckPath?.trim() || null,
    updatedAt: new Date().toISOString(),
  };

  services[index] = updated;
  saveFallbackServices(services);
  return updated;
}

/**
 * Deletes a service configuration by ID.
 */
export async function deleteService(id: string): Promise<boolean> {
  if (isTauri()) {
    return invoke<boolean>("delete_service", { id });
  }

  const services = getFallbackServices();
  const initialLen = services.length;
  const filtered = services.filter((s) => s.id !== id);
  if (filtered.length !== initialLen) {
    saveFallbackServices(filtered);
    return true;
  }
  return false;
}

/**
 * Starts a service by ID using real generic backend process launching.
 */
export async function startService(serviceId: string): Promise<ProcessRuntimeInfo> {
  if (isTauri()) {
    return invoke<ProcessRuntimeInfo>("start_service", { id: serviceId });
  }
  return {
    serviceId,
    pid: Math.floor(Math.random() * 20000) + 1000,
    state: "running",
    startedAt: new Date().toISOString(),
    errorMessage: null,
  };
}

/**
 * Stops a service by ID, terminating the exact tracked child process.
 */
export async function stopService(serviceId: string): Promise<ProcessRuntimeInfo> {
  if (isTauri()) {
    return invoke<ProcessRuntimeInfo>("stop_service", { id: serviceId });
  }
  return {
    serviceId,
    pid: null,
    state: "stopped",
    startedAt: null,
    errorMessage: null,
  };
}

/**
 * Restarts a service by ID (terminates existing process if running, then starts afresh).
 */
export async function restartService(serviceId: string): Promise<ProcessRuntimeInfo> {
  if (isTauri()) {
    return invoke<ProcessRuntimeInfo>("restart_service", { id: serviceId });
  }
  return {
    serviceId,
    pid: Math.floor(Math.random() * 20000) + 1000,
    state: "running",
    startedAt: new Date().toISOString(),
    errorMessage: null,
  };
}

/**
 * Retrieves the current live runtime state for a single service.
 */
export async function getServiceRuntimeState(serviceId: string): Promise<ProcessRuntimeInfo> {
  if (isTauri()) {
    return invoke<ProcessRuntimeInfo>("get_service_runtime_state", { id: serviceId });
  }
  return {
    serviceId,
    pid: null,
    state: "stopped",
    startedAt: null,
    errorMessage: null,
  };
}

/**
 * Retrieves current runtime states for all tracked services.
 */
export async function listServiceRuntimeStates(): Promise<ProcessRuntimeInfo[]> {
  if (isTauri()) {
    return invoke<ProcessRuntimeInfo[]>("list_service_runtime_states");
  }
  return [];
}

/**
 * Event name emitted by backend ProcessMonitor when a tracked service's runtime state changes.
 */
export const SERVICE_RUNTIME_UPDATED_EVENT = "service-runtime-updated";

/**
 * Subscribes to live process runtime state updates emitted by the ProcessMonitor watchdog.
 * Returns an unlisten function to cleanly tear down the subscription.
 */
export async function onServiceRuntimeUpdated(
  callback: (info: ProcessRuntimeInfo) => void
): Promise<() => void> {
  if (isTauri()) {
    return listen<ProcessRuntimeInfo>(
      SERVICE_RUNTIME_UPDATED_EVENT,
      (event) => {
        callback(event.payload);
      }
    );
  }
  return () => {};
}
