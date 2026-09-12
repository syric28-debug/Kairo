import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { LogEntry } from "../types";

/**
 * Event name emitted by backend when a new service log entry is appended.
 */
export const SERVICE_LOG_APPENDED_EVENT = "service-log-appended";

/**
 * Fetches the bounded in-memory log buffer for a specific service.
 */
export async function getServiceLogs(serviceId: string): Promise<LogEntry[]> {
  try {
    return await invoke<LogEntry[]>("get_service_logs", { id: serviceId });
  } catch (error) {
    console.error(`Failed to get logs for service ${serviceId}:`, error);
    throw error;
  }
}

/**
 * Clears the in-memory log buffer for a specific service.
 */
export async function clearServiceLogs(serviceId: string): Promise<void> {
  try {
    await invoke<void>("clear_service_logs", { id: serviceId });
  } catch (error) {
    console.error(`Failed to clear logs for service ${serviceId}:`, error);
    throw error;
  }
}

/**
 * Clears the in-memory log buffer across all tracked services.
 */
export async function clearAllServiceLogs(): Promise<void> {
  try {
    await invoke<void>("clear_all_service_logs");
  } catch (error) {
    console.error("Failed to clear all service logs:", error);
    throw error;
  }
}

/**
 * Subscribes to live log append events from the Tauri backend.
 * Returns an unlisten function to cleanly unsubscribe when unmounting.
 */
export async function onServiceLogAppended(
  callback: (entry: LogEntry) => void
): Promise<UnlistenFn> {
  return await listen<LogEntry>(SERVICE_LOG_APPENDED_EVENT, (event) => {
    callback(event.payload);
  });
}
