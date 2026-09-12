import React from "react";
import {
  Play,
  Square,
  RotateCw,
  Edit3,
  Trash2,
  Folder,
  Radio,
  Zap,
  RefreshCcw,
  AlertCircle,
  Terminal,
} from "lucide-react";
import { ServiceState } from "../../types";

interface ServiceCardProps {
  service: ServiceState;
  onStart: (id: string) => void;
  onStop: (id: string) => void;
  onRestart: (id: string) => void;
  onEdit: (service: ServiceState) => void;
  onDelete: (service: ServiceState) => void;
}

export const ServiceCard: React.FC<ServiceCardProps> = ({
  service,
  onStart,
  onStop,
  onRestart,
  onEdit,
  onDelete,
}) => {
  const { config, status, metrics, errorMessage } = service;
  const isRunning = status === "running";
  const isReady = status === "ready";
  const isActive = isRunning || isReady;
  const isStarting = status === "starting";
  const isStopping = status === "stopping";
  const isRestarting = status === "restarting";
  const isPending = isStopping || isRestarting;

  const fullCommand = `${config.executable} ${config.arguments.join(" ")}`.trim();

  return (
    <div style={styles.card} className="glass-card animate-fade-in">
      {/* Top Header Row */}
      <div style={styles.header}>
        <div style={styles.titleArea}>
          <div style={styles.nameRow}>
            <h3 style={styles.name}>{config.name}</h3>
            <span className={`status-badge ${status}`}>
              <span className="status-dot" />
              <span>{status}</span>
            </span>
            {isActive && metrics?.pid && (
              <span className="code-pill" style={{ fontSize: "0.6875rem", padding: "2px 8px" }}>
                PID: {metrics.pid}
              </span>
            )}
          </div>
          {config.description && (
            <p style={styles.description}>{config.description}</p>
          )}
        </div>

        {/* Action icons */}
        <div style={styles.topActions}>
          <button
            onClick={() => onEdit(service)}
            className="btn-icon"
            title="Edit Service Configuration"
            aria-label="Edit Service Configuration"
            disabled={isPending || isStarting}
          >
            <Edit3 size={14} strokeWidth={1.7} />
          </button>
          <button
            onClick={() => onDelete(service)}
            className="btn-icon btn-icon-danger"
            title="Delete Service"
            aria-label="Delete Service"
            disabled={isPending || isStarting || isActive}
          >
            <Trash2 size={14} strokeWidth={1.7} />
          </button>
        </div>
      </div>

      {/* Error Message if Failed or error present */}
      {(status === "failed" || errorMessage) && (
        <div style={styles.errorBox}>
          <AlertCircle size={13} strokeWidth={1.8} color="#ef4444" />
          <span style={styles.errorText}>{errorMessage || "Service failed unexpectedly"}</span>
        </div>
      )}

      {/* Metadata Fields */}
      <div style={styles.metaContainer}>
        {/* Command */}
        <div style={styles.metaRow}>
          <span style={styles.metaLabel}>
            <Terminal size={12} strokeWidth={1.6} />
            Command
          </span>
          <span className="code-pill" style={styles.codeText} title={fullCommand}>
            {fullCommand}
          </span>
        </div>

        {/* Working Directory */}
        <div style={styles.metaRow}>
          <span style={styles.metaLabel}>
            <Folder size={12} strokeWidth={1.6} />
            Directory
          </span>
          <span className="code-pill" style={styles.codeText} title={config.workingDirectory}>
            {config.workingDirectory}
          </span>
        </div>

        {/* Port & Flags Row */}
        <div style={styles.tagsRow}>
          {/* Port & Readiness */}
          <div style={styles.tagItem}>
          <Radio
            size={12}
            strokeWidth={1.6}
            color={
              isReady
                ? "#22c55e"
                : isRunning || isStarting
                ? "#eab308"
                : "var(--text-muted)"
            }
          />
          <span style={styles.tagLabel}>Port:</span>
          {config.port ? (
            <span style={{ display: "inline-flex", alignItems: "center", gap: "6px" }}>
              <span style={{ ...styles.tagValue, fontFamily: "var(--font-mono)" }}>
                :{config.port}
              </span>
              {isReady ? (
                <span
                  className="code-pill"
                  style={{
                    fontSize: "0.625rem",
                    padding: "1px 6px",
                    color: "#4ade80",
                    borderColor: "rgba(34, 197, 94, 0.3)",
                    backgroundColor: "rgba(34, 197, 94, 0.08)",
                  }}
                >
                  Ready
                </span>
              ) : isRunning || isStarting ? (
                <span
                  className="code-pill"
                  style={{
                    fontSize: "0.625rem",
                    padding: "1px 6px",
                    color: "#facc15",
                    borderColor: "rgba(234, 179, 8, 0.3)",
                    backgroundColor: "rgba(234, 179, 8, 0.08)",
                  }}
                >
                  Waiting...
                </span>
              ) : (
                <span
                  className="code-pill"
                  style={{
                    fontSize: "0.625rem",
                    padding: "1px 6px",
                    color: "var(--text-dim)",
                    borderColor: "var(--border-subtle)",
                  }}
                >
                  Unavailable
                </span>
              )}
            </span>
          ) : (
            <span style={{ ...styles.tagValue, color: "var(--text-dim)" }}>
              None
            </span>
          )}
        </div>

        {/* Auto-start */}
        <div style={styles.tagItem}>
          <Zap
            size={12}
            strokeWidth={1.6}
            color={config.autoStart ? "#ffffff" : "var(--text-dim)"}
          />
          <span style={styles.tagLabel}>Auto-Start:</span>
          <span
            style={{
              ...styles.tagValue,
              color: config.autoStart ? "var(--text-primary)" : "var(--text-dim)",
            }}
          >
            {config.autoStart ? "Enabled" : "Disabled"}
          </span>
        </div>

        {/* Auto-restart */}
        <div style={styles.tagItem}>
          <RefreshCcw
            size={12}
            strokeWidth={1.6}
            color={config.autoRestart ? "#ffffff" : "var(--text-dim)"}
          />
          <span style={styles.tagLabel}>Auto-Restart:</span>
          <span
            style={{
              ...styles.tagValue,
              color: config.autoRestart ? "var(--text-primary)" : "var(--text-dim)",
            }}
          >
            {config.autoRestart ? "Enabled" : "Disabled"}
          </span>
        </div>
      </div>
    </div>

    {/* Metrics Row (if active) */}
    {isActive && metrics && (
      <div style={styles.metricsBar}>
        <div style={styles.metricItem}>
          <span style={styles.metricLabel}>PID:</span>
          <span style={styles.metricValue}>{metrics.pid}</span>
        </div>
        <div style={styles.metricItem}>
          <span style={styles.metricLabel}>CPU:</span>
          <span style={styles.metricValue}>{metrics.cpuPercent?.toFixed(1)}%</span>
        </div>
        <div style={styles.metricItem}>
          <span style={styles.metricLabel}>Memory:</span>
          <span style={styles.metricValue}>
            {metrics.memoryBytes ? `${(metrics.memoryBytes / (1024 * 1024)).toFixed(0)} MB` : "—"}
          </span>
        </div>
        <div style={styles.metricItem}>
          <span style={styles.metricLabel}>Uptime:</span>
          <span style={styles.metricValue}>
            {metrics.uptimeSeconds ? `${Math.floor(metrics.uptimeSeconds / 60)}m` : "—"}
          </span>
        </div>
      </div>
    )}

    {/* Card Actions Footer */}
    <div style={styles.actionsFooter}>
      <div style={styles.actionLeft}>
        {isActive ? (
          <button
            onClick={() => onStop(config.id)}
            className="btn-action"
            style={{ color: "#f87171" }}
            disabled={isPending}
          >
            {isStopping ? (
              <>
                <RotateCw size={12} strokeWidth={2} className="animate-spin" />
                <span>Stopping...</span>
              </>
            ) : (
              <>
                <Square size={12} strokeWidth={2} />
                <span>Stop</span>
              </>
            )}
          </button>
        ) : (
          <button
            onClick={() => onStart(config.id)}
            className="btn-action"
            disabled={isPending || isStarting}
          >
            {isStarting ? (
              <>
                <RotateCw size={12} strokeWidth={2} className="animate-spin" />
                <span>Starting...</span>
              </>
            ) : (
              <>
                <Play size={12} strokeWidth={2} />
                <span>Start</span>
              </>
            )}
          </button>
        )}

        <button
          onClick={() => onRestart(config.id)}
            className="btn-action"
            title="Restart Service"
            disabled={isPending}
          >
            {isRestarting ? (
              <>
                <RotateCw size={12} strokeWidth={1.8} className="animate-spin" />
                <span>Restarting...</span>
              </>
            ) : (
              <>
                <RotateCw size={12} strokeWidth={1.8} />
                <span>Restart</span>
              </>
            )}
          </button>
        </div>

        <div style={styles.actionRight}>
          <button
            onClick={() => onEdit(service)}
            className="btn-ghost"
            style={{ fontSize: "0.75rem", padding: "4px 8px" }}
          >
            <span>Configure</span>
          </button>
        </div>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  card: {
    padding: "20px",
    display: "flex",
    flexDirection: "column",
    gap: "16px",
    position: "relative",
  },
  header: {
    display: "flex",
    alignItems: "flex-start",
    justifyContent: "space-between",
    gap: "12px",
  },
  titleArea: {
    display: "flex",
    flexDirection: "column",
    gap: "4px",
    flex: 1,
  },
  nameRow: {
    display: "flex",
    alignItems: "center",
    gap: "10px",
    flexWrap: "wrap",
  },
  name: {
    fontSize: "0.9375rem",
    fontWeight: 600,
    color: "var(--text-primary)",
    letterSpacing: "-0.01em",
  },
  description: {
    fontSize: "0.8125rem",
    color: "var(--text-secondary)",
    lineHeight: 1.4,
  },
  topActions: {
    display: "flex",
    alignItems: "center",
    gap: "6px",
  },
  errorBox: {
    display: "flex",
    alignItems: "center",
    gap: "8px",
    padding: "8px 12px",
    backgroundColor: "rgba(239, 68, 68, 0.08)",
    border: "1px solid rgba(239, 68, 68, 0.2)",
    borderRadius: "var(--radius-sm)",
  },
  errorText: {
    fontSize: "0.75rem",
    color: "#f87171",
  },
  metaContainer: {
    display: "flex",
    flexDirection: "column",
    gap: "8px",
  },
  metaRow: {
    display: "flex",
    alignItems: "center",
    gap: "12px",
    fontSize: "0.75rem",
  },
  metaLabel: {
    display: "flex",
    alignItems: "center",
    gap: "6px",
    color: "var(--text-muted)",
    width: "78px",
    minWidth: "78px",
  },
  codeText: {
    flex: 1,
  },
  tagsRow: {
    display: "flex",
    alignItems: "center",
    gap: "16px",
    flexWrap: "wrap",
    paddingTop: "4px",
  },
  tagItem: {
    display: "flex",
    alignItems: "center",
    gap: "5px",
    fontSize: "0.75rem",
  },
  tagLabel: {
    color: "var(--text-muted)",
  },
  tagValue: {
    color: "var(--text-secondary)",
  },
  metricsBar: {
    display: "flex",
    alignItems: "center",
    gap: "16px",
    padding: "8px 12px",
    backgroundColor: "rgba(255, 255, 255, 0.02)",
    border: "1px solid var(--border-subtle)",
    borderRadius: "var(--radius-sm)",
  },
  metricItem: {
    display: "flex",
    alignItems: "center",
    gap: "4px",
    fontSize: "0.75rem",
  },
  metricLabel: {
    color: "var(--text-muted)",
  },
  metricValue: {
    fontFamily: "var(--font-mono)",
    color: "var(--text-primary)",
  },
  actionsFooter: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    paddingTop: "12px",
    borderTop: "1px solid var(--border-subtle)",
  },
  actionLeft: {
    display: "flex",
    alignItems: "center",
    gap: "8px",
  },
  actionRight: {
    display: "flex",
    alignItems: "center",
    gap: "8px",
  },
};
