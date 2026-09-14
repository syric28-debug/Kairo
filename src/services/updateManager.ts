/**
 * Phase 12 — IPC client for the KAIRO auto updater.
 *
 * Every operation goes through the official Tauri updater plugin against the
 * official KAIRO GitHub Releases manifest. No direct downloads, no shell
 * execution, no arbitrary URLs — the endpoint is pinned in tauri.conf.json
 * and update packages must carry a valid cryptographic signature.
 */

import { invoke } from "@tauri-apps/api/core";
import { check, type Update, type DownloadEvent } from "@tauri-apps/plugin-updater";
import { openUrl } from "@tauri-apps/plugin-opener";

export interface UpdaterStatus {
  appVersion: string;
  updaterTarget: string | null;
  arch: string;
  signerConfigured: boolean;
}

export interface UpdateMetadata {
  currentVersion: string;
  newVersion: string;
  releaseDate: string | null;
  releaseNotes: string | null;
}

function isTauri(): boolean {
  return (
    typeof window !== "undefined" &&
    Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__)
  );
}

/** Official GitHub Releases page for KAIRO (stable channel only). */
export const KAIRO_RELEASES_URL =
  "https://github.com/syric28-debug/Kairo/releases/latest";

/** Returns true when running inside the Tauri desktop shell. */
export function isTauriRuntime(): boolean {
  return isTauri();
}

/**
 * Fetches installed version, architecture, and updater readiness.
 * `signerConfigured` is false when no production signing key is configured
 * (e.g. development builds) — updates are intentionally unavailable then.
 */
export async function getUpdaterStatus(): Promise<UpdaterStatus> {
  return invoke<UpdaterStatus>("get_updater_status");
}

/**
 * Checks the official GitHub Releases channel for a newer stable version.
 * Resolves `null` when KAIRO is up to date. Version comparison (semver) and
 * signature handling are performed by the Tauri updater plugin.
 */
export async function checkForUpdate(): Promise<{
  metadata: UpdateMetadata;
  update: Update;
} | null> {
  const update: Update | null = await check();
  if (!update) {
    return null;
  }

  const notes = typeof update.body === "string" ? update.body.trim() : "";
  return {
    metadata: {
      currentVersion: update.currentVersion,
      newVersion: update.version,
      releaseDate: update.date ?? null,
      releaseNotes: notes.length > 0 ? notes : null,
    },
    update,
  };
}

/**
 * Downloads and installs the update. The package is verified with the
 * configured Tauri updater signature before installation; an invalid or
 * missing signature aborts the install and rejects the promise.
 *
 * On Windows the app exits after the verified installer is launched and the
 * NSIS installer restarts the new version automatically (passive mode).
 */
export async function downloadAndInstallUpdate(
  update: Update,
  onDownloadEvent: (event: DownloadEvent) => void
): Promise<void> {
  await update.downloadAndInstall(onDownloadEvent, {
    restartAfterInstall: true,
  });
}

/** Releases the underlying updater resource after a check/install cycle. */
export async function releaseUpdateResource(update: Update | null): Promise<void> {
  if (!update) return;
  try {
    await update.close();
  } catch {
    // Resource cleanup is best-effort; never surface this to the user.
  }
}

/** Opens the official GitHub Releases page in the system browser. */
export async function viewReleaseNotes(): Promise<void> {
  await openUrl(KAIRO_RELEASES_URL);
}

/** Restarts KAIRO (post-install fallback; the installer normally restarts). */
export async function restartApp(): Promise<void> {
  await invoke("restart_app");
}
