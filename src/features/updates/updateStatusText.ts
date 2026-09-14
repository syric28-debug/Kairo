/**
 * Phase 12 — Human-readable updater status text shared by the Settings
 * section and the update notification UI (monochrome status model only).
 */

import type { UpdateState } from "../../services/updateStore";

export interface UpdateStatusText {
  text: string;
  tone: "neutral" | "ok" | "busy" | "attention" | "error";
}

export function describeUpdateStatus(state: UpdateState): UpdateStatusText {
  switch (state.phase) {
    case "idle":
      return { text: "Not checked yet.", tone: "neutral" };
    case "checking":
      return { text: "Checking for updates…", tone: "busy" };
    case "upToDate":
      return { text: "You're up to date.", tone: "ok" };
    case "available":
      return {
        text: `KAIRO ${state.newVersion ?? ""} is available.`,
        tone: "attention",
      };
    case "downloading": {
      const pct = computeProgressPercent(state);
      return {
        text:
          pct !== null
            ? `Downloading update… ${pct}%`
            : "Downloading update…",
        tone: "busy",
      };
    }
    case "installing":
      return {
        text: "Installing update… KAIRO will restart automatically.",
        tone: "busy",
      };
    case "error":
      return { text: "Unable to check for updates.", tone: "error" };
    case "unavailable":
      return {
        text: "Updates are not configured in this build.",
        tone: "neutral",
      };
    default:
      return { text: "", tone: "neutral" };
  }
}

/** 0–100 progress when the total size is known, otherwise `null`. */
export function computeProgressPercent(state: UpdateState): number | null {
  if (state.totalBytes === null || state.totalBytes <= 0) return null;
  const pct = Math.floor((state.downloadedBytes / state.totalBytes) * 100);
  return Math.max(0, Math.min(100, pct));
}
