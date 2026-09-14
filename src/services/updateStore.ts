/**
 * Phase 12 — Shared updater state store.
 *
 * A tiny pub/sub store so the update-available banner (App), the Settings
 * section, and the install dialog all observe a single source of truth.
 * The store never blocks startup: all work is asynchronous and every failure
 * is reduced to a status message — KAIRO keeps working normally.
 */

import {
  checkForUpdate,
  downloadAndInstallUpdate,
  getUpdaterStatus,
  releaseUpdateResource,
  type UpdateMetadata,
} from "./updateManager";
import { friendlyErrorMessage } from "./updateErrors";
import { listServiceRuntimeStates, stopService } from "./serviceManager";
import type { Update } from "@tauri-apps/plugin-updater";

export type UpdatePhase =
  | "idle" // no check performed yet
  | "checking" // check in progress
  | "upToDate" // latest stable version is installed
  | "available" // newer stable version found
  | "downloading" // verified package downloading
  | "installing" // installer launched; app restarts automatically
  | "error" // check or install failed — app remains fully usable
  | "unavailable"; // updater not configured in this build (no signing key)

export interface UpdateState {
  phase: UpdatePhase;
  currentVersion: string | null;
  updaterTarget: string | null;
  newVersion: string | null;
  releaseNotes: string | null;
  releaseDate: string | null;
  downloadedBytes: number;
  totalBytes: number | null;
  errorMessage: string | null;
  lastCheckedAt: string | null;
  bannerDismissed: boolean;
  installDialogOpen: boolean;
  runningServicesDetected: number | null;
}

const initialState: UpdateState = {
  phase: "idle",
  currentVersion: null,
  updaterTarget: null,
  newVersion: null,
  releaseNotes: null,
  releaseDate: null,
  downloadedBytes: 0,
  totalBytes: null,
  errorMessage: null,
  lastCheckedAt: null,
  bannerDismissed: false,
  installDialogOpen: false,
  runningServicesDetected: null,
};

let state: UpdateState = initialState;

const listeners = new Set<() => void>();

export function subscribeUpdateState(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function getUpdateState(): UpdateState {
  return state;
}

function patch(partial: Partial<UpdateState>): void {
  state = { ...state, ...partial };
  listeners.forEach((listener) => listener());
}

// The live updater resource from the most recent check (not rendered state).
let activeUpdate: Update | null = null;
let checkInFlight = false;
let installInFlight = false;

/**
 * Runs one update check against the official GitHub Releases channel.
 *
 * Safe to call repeatedly (coalesced). Never throws: every failure is
 * converted into status text. `silent` is used for the optional startup
 * check — failures then leave no intrusive UI, only status text.
 */
export async function runUpdateCheck(silent = false): Promise<void> {
  if (checkInFlight || installInFlight) return;
  checkInFlight = true;

  patch({
    phase: "checking",
    errorMessage: silent ? state.errorMessage : null,
  });

  try {
    const status = await getUpdaterStatus();
    patch({ currentVersion: status.appVersion, updaterTarget: status.updaterTarget });

    if (!status.signerConfigured) {
      await releaseUpdateResource(activeUpdate);
      activeUpdate = null;
      patch({
        phase: "unavailable",
        newVersion: null,
        releaseNotes: null,
        releaseDate: null,
        lastCheckedAt: new Date().toISOString(),
        errorMessage: null,
      });
      return;
    }

    const previous = activeUpdate;
    activeUpdate = null;
    await releaseUpdateResource(previous);

    const result = await checkForUpdate();
    if (!result) {
      patch({
        phase: "upToDate",
        newVersion: null,
        releaseNotes: null,
        releaseDate: null,
        errorMessage: null,
        lastCheckedAt: new Date().toISOString(),
      });
      return;
    }

    // Keep the live plugin `Update` resource for the install step.
    activeUpdate = result.update;
    applyAvailableUpdate(result.metadata);
  } catch (error) {
    patch({
      phase: "error",
      errorMessage: friendlyErrorMessage(error, "Unable to check for updates."),
      lastCheckedAt: new Date().toISOString(),
    });
  } finally {
    checkInFlight = false;
  }
}

function applyAvailableUpdate(metadata: UpdateMetadata): void {
  const isNewRelease = state.newVersion !== metadata.newVersion;
  patch({
    phase: "available",
    newVersion: metadata.newVersion,
    releaseNotes: metadata.releaseNotes,
    releaseDate: metadata.releaseDate,
    errorMessage: null,
    lastCheckedAt: new Date().toISOString(),
    // Re-arm the banner for a different release than the one dismissed.
    bannerDismissed: isNewRelease ? false : state.bannerDismissed,
  });
}

/**
 * Safely shuts down managed services that are currently running, using the
 * existing exact-PID based stop commands (no name-based process killing).
 * Returns how many running services were detected.
 */
async function stopRunningServicesForUpdate(): Promise<number | null> {
  try {
    const runtimeStates = await listServiceRuntimeStates();
    const active = runtimeStates.filter(
      (s) => s.state === "running" || s.state === "ready" || s.state === "starting"
    );
    for (const service of active) {
      await stopService(service.serviceId).catch(() => null);
    }
    return active.length;
  } catch {
    // If states cannot be read, continue — the verified NSIS installer
    // still handles a busy application safely.
    return null;
  }
}

/**
 * Downloads, signature-verifies, and installs the available update.
 * Running managed services are stopped first through the existing
 * PID-based lifecycle (never killed blindly).
 */
export async function installAvailableUpdate(): Promise<void> {
  const update = activeUpdate;
  if (!update || installInFlight) return;
  if (state.phase !== "available" && state.phase !== "error") return;

  installInFlight = true;
  patch({
    phase: "downloading",
    downloadedBytes: 0,
    totalBytes: null,
    errorMessage: null,
  });

  try {
    const stopped = await stopRunningServicesForUpdate();
    patch({ runningServicesDetected: stopped });

    await downloadAndInstallUpdate(update, (event) => {
      if (event.event === "Started") {
        patch({ downloadedBytes: 0, totalBytes: event.data.contentLength ?? null });
      } else if (event.event === "Progress") {
        patch({ downloadedBytes: state.downloadedBytes + event.data.chunkLength });
      }
    });

    patch({ phase: "installing" });
    // On Windows the updater exits the app automatically after launching the
    // verified installer, and the installer relaunches the new version.
  } catch (error) {
    patch({
      phase: "error",
      errorMessage: friendlyErrorMessage(
        error,
        "The update could not be installed. KAIRO was left untouched and fully usable."
      ),
      bannerDismissed: false,
    });
  } finally {
    installInFlight = false;
  }
}

/** Dismisses the update-available banner ("Later"). Update stays available. */
export function dismissUpdateBanner(): void {
  patch({ bannerDismissed: true });
}

/** Opens the non-intrusive install/update dialog. */
export function openInstallDialog(): void {
  patch({ installDialogOpen: true });
}

/** Closes the install/update dialog. */
export function closeInstallDialog(): void {
  patch({ installDialogOpen: false });
}

/**
 * Optional startup check. Reads the persisted setting and defers the actual
 * network request so application startup is never slowed down.
 */
export function scheduleStartupUpdateCheck(enabled: boolean, delayMs = 2500): void {
  if (!enabled) return;
  setTimeout(() => {
    void runUpdateCheck(true);
  }, delayMs);
}
