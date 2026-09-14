import React, { useEffect, useState } from "react";
import {
  X,
  ArrowUpCircle,
  Download,
  ExternalLink,
  Loader,
  CheckCircle2,
  AlertCircle,
  XCircle,
} from "lucide-react";
import { useUpdateState } from "../../hooks/useUpdateState";
import {
  closeInstallDialog,
  dismissUpdateBanner,
  installAvailableUpdate,
  openInstallDialog,
} from "../../services/updateStore";
import { viewReleaseNotes } from "../../services/updateManager";
import { listServiceRuntimeStates } from "../../services/serviceManager";
import {
  computeProgressPercent,
  describeUpdateStatus,
} from "./updateStatusText";

// ─── Update-Available Banner ──────────────────────────────────────────────────

/**
 * Non-intrusive in-app banner shown when a newer stable release was found.
 * Never blocks interaction with the rest of KAIRO and never auto-installs.
 * Also appears when the window was previously hidden to tray, so the update
 * stays accessible without duplicating windows or tray icons.
 */
export const UpdateAvailableBanner: React.FC = () => {
  const update = useUpdateState();

  if (update.phase !== "available" || update.bannerDismissed) {
    return null;
  }

  return (
    <div style={styles.banner} className="animate-fade-in" role="status">
      <span style={styles.bannerDot} />
      <ArrowUpCircle size={15} strokeWidth={2} color="var(--text-primary)" />
      <div style={styles.bannerText}>
        <span style={styles.bannerTitle}>
          KAIRO {update.newVersion} is available.
        </span>
        <span style={styles.bannerSub}>
          Current version: {update.currentVersion ?? "unknown"} → New version:{" "}
          {update.newVersion}
        </span>
      </div>
      <button
        className="btn-secondary"
        style={{ padding: "4px 12px" }}
        onClick={openInstallDialog}
      >
        Install Update
      </button>
      <button
        className="btn-ghost"
        style={{ padding: "4px 10px" }}
        onClick={dismissUpdateBanner}
      >
        Later
      </button>
    </div>
  );
};

// ─── Install / Release-Notes Dialog ───────────────────────────────────────────

/**
 * Install dialog: release-notes preview, version comparison, running-service
 * warning, download progress, and explicit user-controlled install.
 */
export const UpdateInstallDialog: React.FC = () => {
  const update = useUpdateState();
  const [runningServices, setRunningServices] = useState<number | null>(null);

  // Detect currently running managed services (existing PID-based runtime model).
  useEffect(() => {
    if (!update.installDialogOpen) {
      setRunningServices(null);
      return;
    }
    let cancelled = false;
    listServiceRuntimeStates()
      .then((states) => {
        if (cancelled) return;
        setRunningServices(
          states.filter(
            (s) =>
              s.state === "running" ||
              s.state === "ready" ||
              s.state === "starting"
          ).length
        );
      })
      .catch(() => {
        if (!cancelled) setRunningServices(null);
      });
    return () => {
      cancelled = true;
    };
  }, [update.installDialogOpen]);

  if (!update.installDialogOpen) return null;

  const status = describeUpdateStatus(update);
  const busy = update.phase === "downloading" || update.phase === "installing";
  const progress = computeProgressPercent(update);
  const isInstalling = update.phase === "installing";
  const notes = update.releaseNotes;

  return (
    <div style={styles.overlay} onClick={() => !busy && closeInstallDialog()}>
      <div
        style={styles.modal}
        className="glass-panel animate-fade-in"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div style={styles.header}>
          <div style={styles.headerLeft}>
            <div style={styles.headerIcon}>
              <ArrowUpCircle size={18} strokeWidth={1.8} />
            </div>
            <div>
              <h2 style={styles.title}>
                KAIRO {update.newVersion} is available.
              </h2>
              <p style={styles.subtitle}>
                Update from the official GitHub Releases channel
              </p>
            </div>
          </div>
          <button
            onClick={() => !busy && closeInstallDialog()}
            className="btn-icon"
            aria-label="Close dialog"
            disabled={busy}
          >
            <X size={16} strokeWidth={1.8} />
          </button>
        </div>

        <div style={styles.body}>
          {/* Version comparison */}
          <div style={styles.infoGrid}>
            <div style={styles.infoItem}>
              <span style={styles.infoLabel}>Current version</span>
              <span style={styles.infoValue}>
                {update.currentVersion ?? "unknown"}
              </span>
            </div>
            <div style={styles.infoItem}>
              <span style={styles.infoLabel}>New version</span>
              <span style={styles.infoValue}>{update.newVersion ?? "unknown"}</span>
            </div>
          </div>

          {/* Release notes preview — only real data from the official release */}
          {notes ? (
            <div style={styles.notesBox}>
              <span style={styles.infoLabel}>Release notes</span>
              <div style={styles.notesPreview}>{notes}</div>
            </div>
          ) : (
            <p style={styles.notesEmpty}>
              Release notes are not available for this release. Only the version
              information is shown — nothing was invented.
            </p>
          )}

          {/* Running-services warning */}
          {runningServices !== null && runningServices > 0 && (
            <div style={styles.warningBox}>
              <AlertCircle size={14} strokeWidth={2} color="var(--text-secondary)" />
              <span style={styles.warningText}>
                {runningServices} running service{runningServices === 1 ? "" : "s"}{" "}
                will be stopped cleanly (by exact PID) before the update is
                applied.
              </span>
            </div>
          )}

          {/* Progress */}
          {busy && (
            <div style={styles.progressArea}>
              <div style={styles.progressTrack}>
                <div
                  style={{
                    ...styles.progressFill,
                    width: progress !== null ? `${progress}%` : "40%",
                    opacity: progress !== null ? 1 : 0.5,
                  }}
                />
              </div>
              <span style={styles.statusText}>{status.text}</span>
            </div>
          )}

          {/* Error */}
          {update.phase === "error" && update.errorMessage && (
            <div style={styles.errorBox}>
              <XCircle size={14} strokeWidth={2} color="rgba(255,140,140,0.85)" />
              <span style={styles.errorBoxText}>{update.errorMessage}</span>
            </div>
          )}

          {isInstalling && (
            <div style={styles.installingNote}>
              <CheckCircle2 size={14} strokeWidth={2} color="rgba(180,255,180,0.85)" />
              <span style={styles.installingText}>
                Update installed. KAIRO is restarting into the new version.
              </span>
            </div>
          )}
        </div>

        {/* Footer */}
        <div style={styles.footer}>
          <button
            className="btn-ghost"
            onClick={() => void viewReleaseNotes()}
            disabled={busy}
          >
            <ExternalLink size={13} strokeWidth={2} />
            View Release Notes
          </button>
          <div style={{ flex: 1 }} />
          <button className="btn-ghost" onClick={closeInstallDialog} disabled={busy}>
            Later
          </button>
          <button
            className="btn-primary"
            onClick={() => void installAvailableUpdate()}
            disabled={
              busy || (update.phase !== "available" && update.phase !== "error")
            }
          >
            {busy ? (
              <span style={styles.spinner}>
                <Loader size={13} strokeWidth={2} />
              </span>
            ) : (
              <Download size={13} strokeWidth={2} />
            )}
            {update.phase === "downloading"
              ? "Downloading…"
              : update.phase === "installing"
              ? "Installing…"
              : update.phase === "error"
              ? "Retry Install"
              : "Install Update"}
          </button>
        </div>
      </div>
    </div>
  );
};


// ─── Styles (Black + White + Transparent Glass, matching existing patterns) ───

const styles: Record<string, React.CSSProperties> = {
  banner: {
    margin: "10px 28px 0",
    padding: "8px 14px",
    backgroundColor: "rgba(255, 255, 255, 0.05)",
    backdropFilter: "blur(8px)",
    WebkitBackdropFilter: "blur(8px)",
    border: "1px solid rgba(255, 255, 255, 0.14)",
    borderRadius: "var(--radius-sm)",
    display: "flex",
    alignItems: "center",
    gap: "10px",
    flexShrink: 0,
  },
  bannerDot: {
    width: "7px",
    height: "7px",
    borderRadius: "50%",
    backgroundColor: "rgba(255,255,255,0.65)",
    flexShrink: 0,
  },
  bannerText: {
    flex: 1,
    display: "flex",
    flexDirection: "column",
    gap: "1px",
    minWidth: 0,
  },
  bannerTitle: {
    fontSize: "0.78rem",
    fontWeight: 600,
    color: "var(--text-primary)",
  },
  bannerSub: {
    fontSize: "0.7rem",
    fontFamily: "var(--font-mono)",
    color: "var(--text-muted)",
  },
  overlay: {
    position: "fixed",
    inset: 0,
    backgroundColor: "rgba(0, 0, 0, 0.75)",
    backdropFilter: "blur(8px)",
    WebkitBackdropFilter: "blur(8px)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    zIndex: 100,
  },
  modal: {
    width: "560px",
    maxWidth: "92vw",
    maxHeight: "92vh",
    overflowY: "auto",
    borderRadius: "var(--radius-lg)",
    backgroundColor: "var(--glass-bg-primary)",
    border: "1px solid var(--border-medium)",
    boxShadow: "var(--shadow-glass)",
  },
  header: {
    padding: "18px 24px",
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    borderBottom: "1px solid var(--border-subtle)",
  },
  headerLeft: {
    display: "flex",
    alignItems: "center",
    gap: "12px",
  },
  headerIcon: {
    width: "36px",
    height: "36px",
    borderRadius: "var(--radius-sm)",
    backgroundColor: "rgba(255, 255, 255, 0.05)",
    border: "1px solid var(--border-subtle)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
  },
  title: {
    fontSize: "1rem",
    fontWeight: 600,
    color: "var(--text-primary)",
    margin: 0,
  },
  subtitle: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
    margin: 0,
  },
  body: {
    padding: "20px 24px",
    display: "flex",
    flexDirection: "column",
    gap: "16px",
  },
  infoGrid: {
    display: "grid",
    gridTemplateColumns: "repeat(2, 1fr)",
    gap: "14px 24px",
  },
  infoItem: {
    display: "flex",
    flexDirection: "column",
    gap: "3px",
  },
  infoLabel: {
    fontSize: "0.6875rem",
    textTransform: "uppercase",
    letterSpacing: "0.05em",
    color: "var(--text-muted)",
  },
  infoValue: {
    fontSize: "0.8125rem",
    fontFamily: "var(--font-mono)",
    color: "var(--text-secondary)",
  },
  notesBox: {
    display: "flex",
    flexDirection: "column",
    gap: "6px",
  },
  notesPreview: {
    padding: "12px 14px",
    backgroundColor: "rgba(255, 255, 255, 0.02)",
    border: "1px solid var(--border-subtle)",
    borderRadius: "var(--radius-sm)",
    fontSize: "0.78rem",
    lineHeight: 1.55,
    color: "var(--text-secondary)",
    whiteSpace: "pre-wrap",
    wordBreak: "break-word",
    maxHeight: "180px",
    overflowY: "auto",
  },
  notesEmpty: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
    fontStyle: "italic",
    margin: 0,
  },
  warningBox: {
    display: "flex",
    alignItems: "center",
    gap: "10px",
    padding: "10px 14px",
    backgroundColor: "rgba(255, 255, 255, 0.03)",
    border: "1px solid var(--border-medium)",
    borderRadius: "var(--radius-sm)",
  },
  warningText: {
    fontSize: "0.75rem",
    color: "var(--text-secondary)",
  },
  progressArea: {
    display: "flex",
    flexDirection: "column",
    gap: "8px",
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
  statusText: {
    fontSize: "0.72rem",
    fontFamily: "var(--font-mono)",
    color: "var(--text-muted)",
  },
  errorBox: {
    display: "flex",
    alignItems: "center",
    gap: "10px",
    padding: "10px 14px",
    backgroundColor: "rgba(239, 68, 68, 0.08)",
    border: "1px solid rgba(239, 68, 68, 0.25)",
    borderRadius: "var(--radius-sm)",
  },
  errorBoxText: {
    fontSize: "0.75rem",
    color: "#fca5a5",
  },
  installingNote: {
    display: "flex",
    alignItems: "center",
    gap: "10px",
  },
  installingText: {
    fontSize: "0.75rem",
    color: "var(--text-secondary)",
  },
  footer: {
    display: "flex",
    alignItems: "center",
    justifyContent: "flex-end",
    gap: "10px",
    padding: "14px 24px",
    borderTop: "1px solid var(--border-subtle)",
  },
  spinner: {
    display: "inline-flex",
    alignItems: "center",
    color: "var(--accent-contrast)",
    animation: "spin 1s linear infinite",
  },
};

