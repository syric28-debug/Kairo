/**
 * Phase 12 — Updater logic unit tests (dependency-free).
 *
 * Run with:  node src/features/updates/updateLogic.test.ts
 * (Node >= 22 executes TypeScript natively; no test framework required.)
 *
 * No network is used and no fake GitHub release is claimed to exist.
 * Update-check failure handling is verified with synthetic (mocked) errors.
 */
import { strict as assert } from "node:assert";
import type { UpdateState } from "../../services/updateStore.ts";
import { describeUpdateStatus, computeProgressPercent } from "./updateStatusText.ts";
import { friendlyErrorMessage } from "../../services/updateErrors.ts";

let passed = 0;
let failed = 0;

function test(name: string, fn: () => void): void {
  try {
    fn();
    passed++;
    console.log(`PASS: ${name}`);
  } catch (err) {
    failed++;
    console.error(`FAIL: ${name}`);
    console.error(`  ${(err as Error).message}`);
  }
}

function makeState(overrides: Partial<UpdateState>): UpdateState {
  return {
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
    ...overrides,
  };
}

// ─── Update status text (availability / no-update / checking / failure) ──────

test('idle phase reports "Not checked yet."', () => {
  assert.equal(describeUpdateStatus(makeState({ phase: "idle" })).text, "Not checked yet.");
});

test('checking phase reports "Checking for updates…"', () => {
  const s = describeUpdateStatus(makeState({ phase: "checking" }));
  assert.equal(s.text, "Checking for updates…");
  assert.equal(s.tone, "busy");
});

test('up-to-date phase reports "You\'re up to date."', () => {
  const s = describeUpdateStatus(makeState({ phase: "upToDate" }));
  assert.equal(s.text, "You're up to date.");
  assert.equal(s.tone, "ok");
});

test("update-available phase shows the new version", () => {
  const s = describeUpdateStatus(
    makeState({ phase: "available", newVersion: "1.1.0" })
  );
  assert.equal(s.text, "KAIRO 1.1.0 is available.");
  assert.equal(s.tone, "attention");
});

test("downloading phase shows percent when total size is known", () => {
  const s = describeUpdateStatus(
    makeState({ phase: "downloading", downloadedBytes: 25, totalBytes: 100 })
  );
  assert.equal(s.text, "Downloading update… 25%");
  assert.equal(s.tone, "busy");
});

test("downloading phase omits percent when total size is unknown", () => {
  const s = describeUpdateStatus(
    makeState({ phase: "downloading", downloadedBytes: 25, totalBytes: null })
  );
  assert.equal(s.text, "Downloading update…");
});

test("installing phase reports restart", () => {
  const s = describeUpdateStatus(makeState({ phase: "installing" }));
  assert.equal(s.text, "Installing update… KAIRO will restart automatically.");
  assert.equal(s.tone, "busy");
});

test("error phase reports a friendly failure (app stays usable)", () => {
  const s = describeUpdateStatus(makeState({ phase: "error" }));
  assert.equal(s.text, "Unable to check for updates.");
  assert.equal(s.tone, "error");
});

test("unavailable phase reports updates are not configured in this build", () => {
  const s = describeUpdateStatus(makeState({ phase: "unavailable" }));
  assert.equal(s.text, "Updates are not configured in this build.");
  assert.equal(s.tone, "neutral");
});

// ─── Download progress calculation ───────────────────────────────────────────

test("progress is null when total size is unknown or zero", () => {
  assert.equal(computeProgressPercent(makeState({ phase: "downloading", totalBytes: null })), null);
  assert.equal(computeProgressPercent(makeState({ phase: "downloading", totalBytes: 0 })), null);
});

test("progress is floored/clamped to 0..100", () => {
  assert.equal(
    computeProgressPercent(makeState({ phase: "downloading", downloadedBytes: 150, totalBytes: 100 })),
    100
  );
  assert.equal(
    computeProgressPercent(makeState({ phase: "downloading", downloadedBytes: -5, totalBytes: 100 })),
    0
  );
});

test("progress computes expected percentage", () => {
  assert.equal(
    computeProgressPercent(makeState({ phase: "downloading", downloadedBytes: 50, totalBytes: 200 })),
    25
  );
});

// ─── Failure handling (network / signature mocked with synthetic errors) ─────

test("signature verification failures are reported explicitly", () => {
  const msg = friendlyErrorMessage(
    new Error("invalid public key: unable to verify signature"),
    "fallback"
  );
  assert.equal(
    msg,
    "Update signature verification failed or the signing key is not configured."
  );
});

test("network-unreachable failures become a temporary-status message", () => {
  const cases = [
    "failed to connect to github.com: timeout",
    "Error: Network is unreachable",
    "dns resolution failed",
    "connection refused",
    "timed out",
  ];
  for (const c of cases) {
    const msg = friendlyErrorMessage(new Error(c), "fallback");
    assert.equal(
      msg,
      "Unable to reach the official update source. KAIRO keeps working normally."
    );
  }
});

test("empty/unknown errors degrade to the fallback text", () => {
  assert.equal(friendlyErrorMessage(undefined, "fallback text"), "fallback text");
  assert.equal(friendlyErrorMessage("", "fallback text"), "fallback text");
  assert.equal(friendlyErrorMessage(42, "fallback text"), "fallback text");
});

test("short non-classified errors are surfaced, long ones fall back", () => {
  assert.equal(friendlyErrorMessage(new Error("broken JSON"), "fallback text"), "broken JSON");
  const long = "x".repeat(201);
  assert.equal(friendlyErrorMessage(new Error(long), "fallback text"), "fallback text");
});

console.log(`\nUpdate Logic Tests: ${passed} passed, ${failed} failed`);
if (failed > 0) {
  process.exit(1);
}