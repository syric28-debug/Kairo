/**
 * Phase 12 — Pure updater error-message mapping.
 *
 * Kept free of runtime imports so the failure-handling logic can be unit
 * tested without a Tauri shell or a network connection. Update-check and
 * install failures are always reduced to a single friendly status line —
 * KAIRO keeps working normally.
 */

/**
 * Maps an arbitrary thrown value to a user-facing, non-intrusive status text.
 *
 * - Signature/key problems are reported explicitly (never hidden).
 * - Network/reachability problems are reported as a temporary failure.
 * - Anything else degrades to `fallback` when it is empty or overly long,
 *   otherwise the original short message is surfaced.
 */
export function friendlyErrorMessage(error: unknown, fallback: string): string {
  const raw =
    typeof error === "string" ? error : error instanceof Error ? error.message : "";
  const message = raw.trim();
  if (!message) return fallback;
  if (/invalid public key|no public key|signature/i.test(message)) {
    return "Update signature verification failed or the signing key is not configured.";
  }
  if (/network|dns|timed out|timeout|connection|unreachable|github/i.test(message)) {
    return "Unable to reach the official update source. KAIRO keeps working normally.";
  }
  return message.length > 200 ? fallback : message;
}