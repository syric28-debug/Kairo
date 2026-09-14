import React, { useCallback, useEffect, useState } from "react";
import {
  RefreshCw,
  CheckCircle2,
  XCircle,
  Loader,
  ArrowUpCircle,
  Info,
  Download,
} from "lucide-react";
import type { AppSettings } from "../../types";
import {
  getStartupSettings,
  setUpdateAutoCheck,
} from "../../services/settingsManager";
import { useUpdateState } from "../../hooks/useUpdateState";
import { installAvailableUpdate, runUpdateCheck } from "../../services/updateStore";
import {
  computeProgressPercent,
  describeUpdateStatus,
} from "../updates/updateStatusText";

type ToggleState = "idle" | "loading" | "success" | "error";

export const UpdateSection: React.FC = () => {
  const update = useUpdateState();
  const [autoCheck, setAutoCheck] = useState(true);
  const [feedback, setFeedback] = useState<ToggleState>("idle");
  const [settingsError, setSettingsError] = useState<string | null>(null);

  useEffect(() => {
    getStartupSettings()
      .then((s: AppSettings) => setAutoCheck(s.updateAutoCheck))
      .catch((e) => setSettingsError(String(e)));
  }, []);

  const setFieldFeedback = useCallback((next: ToggleState) => {
    setFeedback(next);
    if (next === "success" || next === "error") {
      setTimeout(() => setFeedback("idle"), 2500);
    }
  }, []);

  const handleAutoCheckToggle = async (checked: boolean) => {
    setFeedback("loading");
    try {
      const updated = await setUpdateAutoCheck(checked);
      setAutoCheck(updated.updateAutoCheck);
      setFieldFeedback("success");
    } catch {
      setFieldFeedback("error");
    }
  };

  const status = describeUpdateStatus(update);
  const busy =
    update.phase === "checking" ||
    update.phase === "downloading" ||
    update.phase === "installing";
  const progress = computeProgressPercent(update);

  const statusIcon = () => {
    switch (update.phase) {
      case "checking":
      case "downloading":
      case "installing":
        return (
          <span style={styles.spinner}>
            <Loader size={12} strokeWidth={2} />
          </span>
        );
      case "upToDate":
        return <CheckCircle2 size={14} strokeWidth={2} color="rgba(180,255,180,0.8)" />;
      case "available":
        return <ArrowUpCircle size={14} strokeWidth={2} color="var(--text-primary)" />;
      case "error":
        return <XCircle size={14} strokeWidth={2} color="rgba(255,140,140,0.85)" />;
      case "unavailable":
        return <Info size={14} strokeWidth={2} color="var(--text-muted)" />;
      default:
        return null;
    }
  };

  return (
    <section style={styles.section} className="glass-card">
      <div style={styles.sectionHeader}>
        <div style={styles.iconBox}>
          <RefreshCw size={18} strokeWidth={1.8} />
        </div>
        <div>
          <h3 style={styles.sectionTitle}>Updates</h3>
          <p style={styles.sectionDesc}>
            Automatic updates from the official KAIRO GitHub Releases channel,
            cryptographically verified
          </p>
        </div>
      </div>

      <div style={styles.settingsList}>
        {/* Current Version */}
        <div style={styles.settingItem}>
          <div>
            <span style={styles.settingTitle}>Current Version</span>
            <p style={styles.settingSubtext}>
              Installed build{update.updaterTarget ? ` • ${update.updaterTarget}` : ""}
            </p>
          </div>
          <span style={styles.activePill}>{update.currentVersion ?? "…"}</span>
        </div>


        {/* Update Status */}
        <div style={styles.settingItem}>
          <div>
            <div style={styles.settingTitleRow}>
              <span style={styles.settingTitle}>Update Status</span>
              {statusIcon()}
            </div>
            <p style={styles.settingSubtext}>{status.text}</p>
            {update.phase === "error" && update.errorMessage && (
              <p style={styles.errorText}>{update.errorMessage}</p>
            )}
            {update.phase === "unavailable" && (
              <p style={styles.settingSubtext}>
                Production update signing is not configured for this installation.
              </p>
            )}
          </div>
          {update.phase === "available" && (
            <button
              className="btn-primary"
              onClick={() => void installAvailableUpdate()}
              disabled={busy}
              style={styles.actionButton}
            >
              <Download size={13} strokeWidth={2} />
              Install Update
            </button>
          )}
        </div>

        {/* Download progress bar */}
        {(update.phase === "downloading" || update.phase === "installing") && (
          <div style={styles.progressTrack}>
            <div
              style={{
                ...styles.progressFill,
                width: progress !== null ? `${progress}%` : "40%",
                opacity: progress !== null ? 1 : 0.5,
              }}
            />
          </div>
        )}

        {/* Check for Updates */}
        <div style={styles.settingItem}>
          <div>
            <span style={styles.settingTitle}>Check for Updates</span>
            <p style={styles.settingSubtext}>
              Manually query the official stable release channel (HTTPS,
              signature-verified)
            </p>
          </div>
          <button
            className="btn-secondary"
            onClick={() => void runUpdateCheck(false)}
            disabled={busy}
            style={styles.actionButton}
          >
            <RefreshCw
              size={13}
              strokeWidth={2}
              className={update.phase === "checking" ? "animate-spin" : undefined}
            />
            {update.phase === "checking" ? "Checking…" : "Check for Updates"}
          </button>
        </div>

        {/* Auto-check on startup */}
        <div style={styles.settingItem}>
          <div>
            <div style={styles.settingTitleRow}>
              <span style={styles.settingTitle}>Check for updates on startup</span>
              {feedback === "loading" && (
                <span style={styles.spinner}>
                  <Loader size={12} strokeWidth={2} />
                </span>
              )}
              {feedback === "success" && (
                <CheckCircle2 size={12} strokeWidth={2} color="rgba(180,255,180,0.85)" />
              )}
              {feedback === "error" && (
                <XCircle size={12} strokeWidth={2} color="rgba(255,140,140,0.85)" />
              )}
            </div>
            <p style={styles.settingSubtext}>
              Runs quietly in the background after launch — never blocks startup
            </p>
            {settingsError && (
              <p style={styles.errorText}>Settings unavailable: {settingsError}</p>
            )}
          </div>
          <label style={styles.switch}>
            <input
              id="toggle-update-auto-check"
              type="checkbox"
              checked={autoCheck}
              onChange={(e) => void handleAutoCheckToggle(e.target.checked)}
            />
            <span
              style={{
                ...styles.slider,
                backgroundColor: autoCheck
                  ? "rgba(255,255,255,0.55)"
                  : "rgba(255,255,255,0.15)",
              }}
            />
          </label>
        </div>
      </div>
    </section>
  );
};

// ─── Styles (matching the existing Settings section conventions) ─────────────

const styles: Record<string, React.CSSProperties> = {
  section: {
    padding: "20px 24px",
    display: "flex",
    flexDirection: "column",
    gap: "18px",
  },
  sectionHeader: {
    display: "flex",
    alignItems: "center",
    gap: "14px",
    paddingBottom: "14px",
    borderBottom: "1px solid var(--border-subtle)",
  },
  iconBox: {
    width: "36px",
    height: "36px",
    borderRadius: "var(--radius-sm)",
    backgroundColor: "rgba(255, 255, 255, 0.04)",
    border: "1px solid var(--border-subtle)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
  },
  sectionTitle: {
    fontSize: "0.9375rem",
    fontWeight: 600,
    color: "var(--text-primary)",
    margin: 0,
  },
  sectionDesc: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
    margin: 0,
  },
  settingsList: {
    display: "flex",
    flexDirection: "column",
    gap: "16px",
  },
  settingItem: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    gap: "16px",
  },
  settingTitleRow: {
    display: "flex",
    alignItems: "center",
    gap: "8px",
  },
  settingTitle: {
    fontSize: "0.8125rem",
    fontWeight: 500,
    color: "var(--text-primary)",
  },
  settingSubtext: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
    marginTop: "2px",
  },
  errorText: {
    fontSize: "0.7rem",
    color: "rgba(255,140,140,0.75)",
    marginTop: "4px",
    fontFamily: "var(--font-mono)",
  },
  activePill: {
    fontSize: "0.75rem",
    fontFamily: "var(--font-mono)",
    color: "var(--text-primary)",
    padding: "3px 10px",
    borderRadius: "var(--radius-xs)",
    backgroundColor: "rgba(255, 255, 255, 0.06)",
    border: "1px solid var(--border-subtle)",
    flexShrink: 0,
  },
  actionButton: {
    flexShrink: 0,
  },
  switch: {
    position: "relative",
    display: "inline-block",
    width: "38px",
    height: "20px",
    flexShrink: 0,
  },
  slider: {
    position: "absolute",
    cursor: "pointer",
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: "rgba(255, 255, 255, 0.15)",
    borderRadius: "20px",
    transition: "background-color 0.25s",
  },
  spinner: {
    display: "inline-flex",
    alignItems: "center",
    color: "var(--text-muted)",
    animation: "spin 1s linear infinite",
  },
  progressTrack: {
    height: "4px",
    width: "100%",
    borderRadius: "var(--radius-full)",
    backgroundColor: "rgba(255, 255, 255, 0.06)",
    overflow: "hidden",
  },
  progressFill: {
    height: "100%",
    borderRadius: "var(--radius-full)",
    backgroundColor: "rgba(255, 255, 255, 0.55)",
    transition: "width 0.2s ease, opacity 0.3s ease",
  },
};

