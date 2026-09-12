import React from "react";
import {
  Layers,
  Play,
  Square,
  AlertTriangle,
  Server,
  ArrowRight,
  ShieldCheck,
  Cpu,
  HardDrive,
} from "lucide-react";
import { ServiceState, NavigationPage } from "../../types";

interface DashboardViewProps {
  services: ServiceState[];
  onNavigate: (page: NavigationPage) => void;
  onStart: (id: string) => void;
  onStop: (id: string) => void;
  onEdit: (service: ServiceState) => void;
}

export const DashboardView: React.FC<DashboardViewProps> = ({
  services,
  onNavigate,
  onStart,
  onStop,
  onEdit,
}) => {
  const total = services.length;
  const runningServices = services.filter((s) => s.status === "running" || s.status === "ready");
  const running = runningServices.length;
  const ready = services.filter((s) => s.status === "ready").length;
  const stopped = services.filter((s) => s.status === "stopped").length;
  const failed = services.filter((s) => s.status === "failed").length;

  const recentServices = services.slice(0, 4);

  return (
    <div style={styles.container} className="animate-fade-in">
      {/* 4 Overview Metric Cards */}
      <section style={styles.metricsGrid}>
        {/* Total Services */}
        <div style={styles.metricCard} className="glass-card">
          <div style={styles.metricHeader}>
            <span style={styles.metricTitle}>Total Services</span>
            <div style={styles.metricIconBox}>
              <Layers size={16} strokeWidth={1.7} color="#ffffff" />
            </div>
          </div>
          <div style={styles.metricValueRow}>
            <span style={styles.metricNumber}>{total}</span>
            <span style={styles.metricSubtext}>Configured</span>
          </div>
        </div>

        {/* Running */}
        <div style={styles.metricCard} className="glass-card">
          <div style={styles.metricHeader}>
            <span style={styles.metricTitle}>Running</span>
            <div style={{ ...styles.metricIconBox, borderColor: "rgba(34, 197, 94, 0.3)" }}>
              <Play size={16} strokeWidth={2} color="#22c55e" />
            </div>
          </div>
          <div style={styles.metricValueRow}>
            <span style={{ ...styles.metricNumber, color: "#ffffff" }}>{running}</span>
            <span style={{ ...styles.metricSubtext, color: "#4ade80" }}>
              {ready > 0 ? `${ready} Ready • Active` : "Active processes"}
            </span>
          </div>
        </div>

        {/* Stopped */}
        <div style={styles.metricCard} className="glass-card">
          <div style={styles.metricHeader}>
            <span style={styles.metricTitle}>Stopped</span>
            <div style={styles.metricIconBox}>
              <Square size={16} strokeWidth={1.8} color="#71717a" />
            </div>
          </div>
          <div style={styles.metricValueRow}>
            <span style={styles.metricNumber}>{stopped}</span>
            <span style={styles.metricSubtext}>Idle / Ready</span>
          </div>
        </div>

        {/* Failed */}
        <div style={styles.metricCard} className="glass-card">
          <div style={styles.metricHeader}>
            <span style={styles.metricTitle}>Failed</span>
            <div style={{ ...styles.metricIconBox, borderColor: "rgba(239, 68, 68, 0.3)" }}>
              <AlertTriangle size={16} strokeWidth={1.8} color="#ef4444" />
            </div>
          </div>
          <div style={styles.metricValueRow}>
            <span style={{ ...styles.metricNumber, color: failed > 0 ? "#f87171" : "#ffffff" }}>
              {failed}
            </span>
            <span style={{ ...styles.metricSubtext, color: failed > 0 ? "#f87171" : "var(--text-muted)" }}>
              {failed > 0 ? "Requires attention" : "No errors"}
            </span>
          </div>
        </div>
      </section>

      {/* System Status Banner */}
      <section style={styles.systemBanner} className="glass-card">
        <div style={styles.bannerLeft}>
          <div style={styles.bannerIcon}>
            <ShieldCheck size={20} strokeWidth={1.8} color="#ffffff" />
          </div>
          <div>
            <h4 style={styles.bannerTitle}>Desktop Engine Active</h4>
            <p style={styles.bannerDesc}>
              Windows Job Objects enabled • Automatic process hierarchy kill tree ready
            </p>
          </div>
        </div>
        <div style={styles.bannerRight}>
          <div style={styles.bannerStat}>
            <Cpu size={13} strokeWidth={1.6} color="var(--text-muted)" />
            <span>Process Monitor: Active</span>
          </div>
          <div style={styles.bannerStat}>
            <HardDrive size={13} strokeWidth={1.6} color="var(--text-muted)" />
            <span>Loopback IPC: Connected</span>
          </div>
        </div>
      </section>

      {/* Recent Services Section */}
      <section style={styles.recentSection}>
        <div style={styles.sectionHeader}>
          <div>
            <h3 style={styles.sectionTitle}>Recent Services</h3>
            <p style={styles.sectionSubtitle}>
              Quick actions for actively managed processes
            </p>
          </div>
          <button
            onClick={() => onNavigate("services")}
            className="btn-ghost"
            style={{ fontSize: "0.8125rem", gap: "6px" }}
          >
            <span>View All Services ({total})</span>
            <ArrowRight size={14} strokeWidth={1.8} />
          </button>
        </div>

        {/* Recent Service Table / Rows */}
        <div style={styles.tableContainer} className="glass-card">
          <div style={styles.tableHeader}>
            <div style={{ flex: 2 }}>Service</div>
            <div style={{ flex: 1 }}>Status</div>
            <div style={{ flex: 1 }}>Port</div>
            <div style={{ flex: 2 }}>Command</div>
            <div style={{ width: "120px", textAlign: "right" }}>Actions</div>
          </div>

          {recentServices.length === 0 ? (
            <div style={{ padding: "36px 20px", textAlign: "center", color: "var(--text-muted)", fontSize: "0.8125rem" }}>
              No services configured yet. Navigate to Services or use the header button to add your first service.
            </div>
          ) : (
            recentServices.map((svc) => {
              const isActive = svc.status === "running" || svc.status === "ready";
              return (
                <div key={svc.config.id} style={styles.tableRow}>
                  {/* Service Name & Desc */}
                  <div style={{ flex: 2, display: "flex", flexDirection: "column", gap: "2px" }}>
                    <span style={styles.serviceName}>{svc.config.name}</span>
                    <span style={styles.serviceDesc}>{svc.config.description}</span>
                  </div>

                  {/* Status */}
                  <div style={{ flex: 1 }}>
                    <span className={`status-badge ${svc.status}`}>
                      <span className="status-dot" />
                      <span>{svc.status}</span>
                    </span>
                  </div>

                  {/* Port */}
                  <div style={{ flex: 1, fontFamily: "var(--font-mono)", fontSize: "0.75rem", color: "var(--text-secondary)" }}>
                    {svc.config.port ? `:${svc.config.port}` : "—"}
                  </div>

                  {/* Command */}
                  <div style={{ flex: 2, overflow: "hidden" }}>
                    <span
                      className="code-pill"
                      style={{ maxWidth: "90%" }}
                      title={`${svc.config.executable} ${svc.config.arguments.join(" ")}`}
                    >
                      {svc.config.executable}
                    </span>
                  </div>

                  {/* Quick Action Button */}
                  <div style={{ width: "120px", display: "flex", justifyContent: "flex-end", gap: "6px" }}>
                    {isActive ? (
                      <button
                        onClick={() => onStop(svc.config.id)}
                        className="btn-action"
                        style={{ color: "#f87171" }}
                        title="Stop Service"
                      >
                        <Square size={11} strokeWidth={2} />
                        <span>Stop</span>
                      </button>
                    ) : (
                      <button
                        onClick={() => onStart(svc.config.id)}
                        className="btn-action"
                        title="Start Service"
                      >
                        <Play size={11} strokeWidth={2} />
                        <span>Start</span>
                      </button>
                    )}
                    <button
                      onClick={() => onEdit(svc)}
                      className="btn-icon"
                      style={{ width: "26px", height: "26px" }}
                      title="Configure"
                    >
                      <Server size={12} strokeWidth={1.8} />
                    </button>
                  </div>
                </div>
              );
            })
          )}
        </div>
      </section>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: "flex",
    flexDirection: "column",
    gap: "24px",
  },
  metricsGrid: {
    display: "grid",
    gridTemplateColumns: "repeat(4, 1fr)",
    gap: "16px",
  },
  metricCard: {
    padding: "18px 20px",
    display: "flex",
    flexDirection: "column",
    gap: "14px",
  },
  metricHeader: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
  },
  metricTitle: {
    fontSize: "0.8125rem",
    fontWeight: 500,
    color: "var(--text-secondary)",
  },
  metricIconBox: {
    width: "30px",
    height: "30px",
    borderRadius: "var(--radius-sm)",
    backgroundColor: "rgba(255, 255, 255, 0.04)",
    border: "1px solid var(--border-subtle)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
  },
  metricValueRow: {
    display: "flex",
    alignItems: "baseline",
    gap: "10px",
  },
  metricNumber: {
    fontSize: "1.75rem",
    fontWeight: 600,
    letterSpacing: "-0.03em",
    color: "var(--text-primary)",
    fontFamily: "var(--font-mono)",
  },
  metricSubtext: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
  },
  systemBanner: {
    padding: "16px 20px",
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
  },
  bannerLeft: {
    display: "flex",
    alignItems: "center",
    gap: "14px",
  },
  bannerIcon: {
    width: "38px",
    height: "38px",
    borderRadius: "var(--radius-sm)",
    backgroundColor: "rgba(255, 255, 255, 0.05)",
    border: "1px solid var(--border-medium)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
  },
  bannerTitle: {
    fontSize: "0.875rem",
    fontWeight: 600,
    color: "var(--text-primary)",
  },
  bannerDesc: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
  },
  bannerRight: {
    display: "flex",
    alignItems: "center",
    gap: "20px",
  },
  bannerStat: {
    display: "flex",
    alignItems: "center",
    gap: "6px",
    fontSize: "0.75rem",
    color: "var(--text-secondary)",
  },
  recentSection: {
    display: "flex",
    flexDirection: "column",
    gap: "14px",
  },
  sectionHeader: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
  },
  sectionTitle: {
    fontSize: "1rem",
    fontWeight: 600,
    letterSpacing: "-0.01em",
    color: "var(--text-primary)",
  },
  sectionSubtitle: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
  },
  tableContainer: {
    overflow: "hidden",
  },
  tableHeader: {
    display: "flex",
    alignItems: "center",
    padding: "12px 20px",
    fontSize: "0.6875rem",
    fontWeight: 600,
    letterSpacing: "0.05em",
    color: "var(--text-muted)",
    borderBottom: "1px solid var(--border-subtle)",
    textTransform: "uppercase",
  },
  tableRow: {
    display: "flex",
    alignItems: "center",
    padding: "14px 20px",
    borderBottom: "1px solid var(--border-subtle)",
    transition: "background-color var(--transition-fast)",
  },
  serviceName: {
    fontSize: "0.875rem",
    fontWeight: 500,
    color: "var(--text-primary)",
  },
  serviceDesc: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
    whiteSpace: "nowrap",
    overflow: "hidden",
    textOverflow: "ellipsis",
    maxWidth: "280px",
  },
};
