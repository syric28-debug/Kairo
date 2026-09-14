import React, { useState, useEffect, useCallback } from "react";
import {
  Sliders,
  Terminal,
  Info,
  Lock,
  Loader,
  CheckCircle,
  XCircle,
} from "lucide-react";
import type { AppSettings } from "../../types";
import {
  getStartupSettings,
  setAppAutoStart,
  setServiceAutoStart,
} from "../../services/settingsManager";
import { UpdateSection } from "./UpdateSection";
import { APP_VERSION, APP_MILESTONE } from "../../constants/version";

// ─── Types ────────────────────────────────────────────────────────────────────

type ToggleState = "idle" | "loading" | "success" | "error";

interface ToggleFeedback {
  appAutoStart: ToggleState;
  serviceAutoStart: ToggleState;
}

// ─── Component ────────────────────────────────────────────────────────────────

export const SettingsView: React.FC = () => {
  const [settings, setSettings] = useState<AppSettings>({
    appAutoStart: false,
    serviceAutoStart: false,
    updateAutoCheck: true,
    updatedAt: "",
  });
  const [feedback, setFeedback] = useState<ToggleFeedback>({
    appAutoStart: "idle",
    serviceAutoStart: "idle",
  });
  const [loadError, setLoadError] = useState<string | null>(null);
  const [minimizeToTray, setMinimizeToTray] = useState(true);
  const [portCollisionWarning, setPortCollisionWarning] = useState(true);
  const [defaultKillTimeout, setDefaultKillTimeout] = useState("5000");

  // Load settings on mount, sync with live registry state.
  useEffect(() => {
    getStartupSettings()
      .then(setSettings)
      .catch((e) => setLoadError(String(e)));
  }, []);

  // Auto-clear success/error feedback after 2.5s.
  const setFieldFeedback = useCallback(
    (field: keyof ToggleFeedback, state: ToggleState) => {
      setFeedback((prev) => ({ ...prev, [field]: state }));
      if (state === "success" || state === "error") {
        setTimeout(() => {
          setFeedback((prev) => ({ ...prev, [field]: "idle" }));
        }, 2500);
      }
    },
    []
  );

  const handleAppAutoStart = async (checked: boolean) => {
    setFieldFeedback("appAutoStart", "loading");
    try {
      const updated = await setAppAutoStart(checked);
      setSettings(updated);
      setFieldFeedback("appAutoStart", "success");
    } catch {
      setFieldFeedback("appAutoStart", "error");
    }
  };

  const handleServiceAutoStart = async (checked: boolean) => {
    setFieldFeedback("serviceAutoStart", "loading");
    try {
      const updated = await setServiceAutoStart(checked);
      setSettings(updated);
      setFieldFeedback("serviceAutoStart", "success");
    } catch {
      setFieldFeedback("serviceAutoStart", "error");
    }
  };

  return (
    <div style={styles.container} className="animate-fade-in">
      {/* General Settings */}
      <section style={styles.section} className="glass-card">
        <div style={styles.sectionHeader}>
          <div style={styles.iconBox}>
            <Sliders size={18} strokeWidth={1.8} />
          </div>
          <div>
            <h3 style={styles.sectionTitle}>General Preferences</h3>
            <p style={styles.sectionDesc}>
              Configure runtime behavior and Windows desktop integration
            </p>
          </div>
        </div>

        <div style={styles.settingsList}>
          {/* Design System Locked Pill */}
          <div style={styles.settingItem}>
            <div>
              <div style={styles.settingTitleRow}>
                <span style={styles.settingTitle}>Visual Design System</span>
                <span style={styles.lockedBadge}>
                  <Lock size={10} strokeWidth={2} />
                  <span>Permanent</span>
                </span>
              </div>
              <p style={styles.settingSubtext}>
                Black + White + Transparent Glass aesthetic
              </p>
            </div>
            <span style={styles.activePill}>Active</span>
          </div>

          {/* Minimize to Tray */}
          <div style={styles.settingItem}>
            <div>
              <span style={styles.settingTitle}>Minimize to System Tray</span>
              <p style={styles.settingSubtext}>
                Closing the window hides it to the Windows notification tray instead of quitting
              </p>
            </div>
            <label style={styles.switch}>
              <input
                type="checkbox"
                checked={minimizeToTray}
                onChange={(e) => setMinimizeToTray(e.target.checked)}
              />
              <span style={styles.slider} />
            </label>
          </div>

          {/* ── Windows Startup — LIVE ── */}
          <div style={styles.settingItem}>
            <div style={{ flex: 1 }}>
              <div style={styles.settingTitleRow}>
                <span style={styles.settingTitle}>Launch on Windows Boot</span>
                <FeedbackIcon state={feedback.appAutoStart} />
              </div>
              <p style={styles.settingSubtext}>
                Register with Windows HKCU\Run to start automatically on user logon
              </p>
              {loadError && (
                <p style={styles.errorText}>Settings unavailable: {loadError}</p>
              )}
            </div>
            <label style={{
              ...styles.switch,
              opacity: feedback.appAutoStart === "loading" ? 0.5 : 1,
              pointerEvents: feedback.appAutoStart === "loading" ? "none" : "auto",
            }}>
              <input
                id="toggle-app-auto-start"
                type="checkbox"
                checked={settings.appAutoStart}
                onChange={(e) => handleAppAutoStart(e.target.checked)}
              />
              <span style={{
                ...styles.slider,
                backgroundColor: settings.appAutoStart
                  ? "rgba(255,255,255,0.55)"
                  : "rgba(255,255,255,0.15)",
              }} />
            </label>
          </div>

          {/* ── Service Auto-Start — LIVE ── */}
          <div style={styles.settingItem}>
            <div style={{ flex: 1 }}>
              <div style={styles.settingTitleRow}>
                <span style={styles.settingTitle}>
                  Start configured services on launch
                </span>
                <FeedbackIcon state={feedback.serviceAutoStart} />
              </div>
              <p style={styles.settingSubtext}>
                Automatically start all services with Auto-Start enabled when the app opens
              </p>
            </div>
            <label style={{
              ...styles.switch,
              opacity: feedback.serviceAutoStart === "loading" ? 0.5 : 1,
              pointerEvents: feedback.serviceAutoStart === "loading" ? "none" : "auto",
            }}>
              <input
                id="toggle-service-auto-start"
                type="checkbox"
                checked={settings.serviceAutoStart}
                onChange={(e) => handleServiceAutoStart(e.target.checked)}
              />
              <span style={{
                ...styles.slider,
                backgroundColor: settings.serviceAutoStart
                  ? "rgba(255,255,255,0.55)"
                  : "rgba(255,255,255,0.15)",
              }} />
            </label>
          </div>

          {/* Port collision detection */}
          <div style={styles.settingItem}>
            <div>
              <span style={styles.settingTitle}>Pre-flight Port Probing</span>
              <p style={styles.settingSubtext}>
                Check if target ports are already bound before launching service binaries
              </p>
            </div>
            <label style={styles.switch}>
              <input
                type="checkbox"
                checked={portCollisionWarning}
                onChange={(e) => setPortCollisionWarning(e.target.checked)}
              />
              <span style={styles.slider} />
            </label>
          </div>
        </div>
      </section>

      {/* Process Engine Defaults */}
      <section style={styles.section} className="glass-card">
        <div style={styles.sectionHeader}>
          <div style={styles.iconBox}>
            <Terminal size={18} strokeWidth={1.8} />
          </div>
          <div>
            <h3 style={styles.sectionTitle}>Process Engine Defaults</h3>
            <p style={styles.sectionDesc}>
              Process termination timeouts and Windows Job Object constraints
            </p>
          </div>
        </div>

        <div style={styles.settingsList}>
          <div style={styles.settingItem}>
            <div>
              <span style={styles.settingTitle}>Kill Grace Period</span>
              <p style={styles.settingSubtext}>
                Milliseconds to wait for graceful exit before hard terminating the Windows Job tree
              </p>
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
              <input
                type="number"
                className="input-glass"
                value={defaultKillTimeout}
                onChange={(e) => setDefaultKillTimeout(e.target.value)}
                style={{ width: "90px", textAlign: "right", fontFamily: "var(--font-mono)" }}
              />
              <span style={{ fontSize: "0.75rem", color: "var(--text-muted)" }}>ms</span>
            </div>
          </div>
        </div>
      </section>

      {/* Phase 12: Updates */}
      <UpdateSection />

      {/* Platform Information */}
      <section style={styles.section} className="glass-card">
        <div style={styles.sectionHeader}>
          <div style={styles.iconBox}>
            <Info size={18} strokeWidth={1.8} />
          </div>
          <div>
            <h3 style={styles.sectionTitle}>Application Information</h3>
            <p style={styles.sectionDesc}>
              System environment and compiled build specifications
            </p>
          </div>
        </div>

        <div style={styles.infoGrid}>
          <div style={styles.infoItem}>
            <span style={styles.infoLabel}>Application</span>
            <span style={styles.infoValue}>KAIRO</span>
          </div>
          <div style={styles.infoItem}>
            <span style={styles.infoLabel}>App Version</span>
            <span style={styles.infoValue}>{APP_VERSION}</span>
          </div>
          <div style={styles.infoItem}>
            <span style={styles.infoLabel}>Identifier</span>
            <span style={styles.infoValue}>com.kairo.localruntimemanager</span>
          </div>
          <div style={styles.infoItem}>
            <span style={styles.infoLabel}>Current Milestone</span>
            <span style={styles.infoValue}>{APP_MILESTONE} (Automatic Updates)</span>
          </div>
          <div style={styles.infoItem}>
            <span style={styles.infoLabel}>Tech Stack</span>
            <span style={styles.infoValue}>Tauri 2 • React 19 • Rust 1.98</span>
          </div>
          <div style={styles.infoItem}>
            <span style={styles.infoLabel}>Platform Target</span>
            <span style={styles.infoValue}>Windows x64 / x86</span>
          </div>
          <div style={styles.infoItem}>
            <span style={styles.infoLabel}>Design Language</span>
            <span style={styles.infoValue}>Black + White + Transparent Glass</span>
          </div>
        </div>
      </section>
    </div>
  );
};

// ─── Feedback Icon ─────────────────────────────────────────────────────────────

const FeedbackIcon: React.FC<{ state: ToggleState }> = ({ state }) => {
  if (state === "loading") {
    return (
      <span style={feedbackStyles.spinner}>
        <Loader size={12} strokeWidth={2} />
      </span>
    );
  }
  if (state === "success") {
    return (
      <span style={{ ...feedbackStyles.badge, color: "rgba(180,255,180,0.85)" }}>
        <CheckCircle size={12} strokeWidth={2} />
      </span>
    );
  }
  if (state === "error") {
    return (
      <span style={{ ...feedbackStyles.badge, color: "rgba(255,140,140,0.85)" }}>
        <XCircle size={12} strokeWidth={2} />
      </span>
    );
  }
  return null;
};

const feedbackStyles: Record<string, React.CSSProperties> = {
  spinner: {
    display: "inline-flex",
    alignItems: "center",
    color: "var(--text-muted)",
    animation: "spin 1s linear infinite",
  },
  badge: {
    display: "inline-flex",
    alignItems: "center",
    transition: "opacity 0.3s",
  },
};

// ─── Styles ────────────────────────────────────────────────────────────────────

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: "flex",
    flexDirection: "column",
    gap: "20px",
    maxWidth: "880px",
  },
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
  },
  sectionDesc: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
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
  lockedBadge: {
    display: "inline-flex",
    alignItems: "center",
    gap: "4px",
    fontSize: "0.625rem",
    fontFamily: "var(--font-mono)",
    padding: "1px 6px",
    borderRadius: "var(--radius-xs)",
    backgroundColor: "rgba(255, 255, 255, 0.05)",
    border: "1px solid var(--border-subtle)",
    color: "var(--text-secondary)",
  },
  activePill: {
    fontSize: "0.75rem",
    fontFamily: "var(--font-mono)",
    color: "var(--text-primary)",
    padding: "3px 10px",
    borderRadius: "var(--radius-xs)",
    backgroundColor: "rgba(255, 255, 255, 0.06)",
    border: "1px solid var(--border-subtle)",
  },
  switch: {
    position: "relative",
    display: "inline-block",
    width: "38px",
    height: "20px",
    flexShrink: 0,
    transition: "opacity 0.2s",
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
};
