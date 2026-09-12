/**
 * IPC client for Phase 6 startup settings commands.
 * All calls go directly to the Rust backend via Tauri invoke/listen — no shell, no HTTP.
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AppSettings, StartupProgress } from "../types";

/**
 * Fetches current startup settings.
 * The `appAutoStart` field always reflects the live Windows registry state.
 */
export async function getStartupSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_startup_settings");
}

/**
 * Enables or disables Windows logon registration (HKCU\Run).
 * Returns the updated settings with live registry state.
 */
export async function setAppAutoStart(enabled: boolean): Promise<AppSettings> {
  return invoke<AppSettings>("set_app_auto_start", { enabled });
}

/**
 * Enables or disables the global service auto-start master switch.
 * Returns the updated settings.
 */
export async function setServiceAutoStart(
  enabled: boolean
): Promise<AppSettings> {
  return invoke<AppSettings>("set_service_auto_start", { enabled });
}

/**
 * Subscribes to `startup-progress` events emitted during service auto-start.
 * Returns an unlisten function to clean up the listener.
 */
export async function onStartupProgress(
  callback: (progress: StartupProgress) => void
): Promise<UnlistenFn> {
  return listen<StartupProgress>("startup-progress", (event) => {
    callback(event.payload);
  });
}
