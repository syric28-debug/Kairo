import React from "react";
import { LayoutDashboard, Server, Terminal, Settings, Layers, Activity } from "lucide-react";
import { NavigationPage } from "../../types";
import { APP_VERSION_LABEL } from "../../constants/version";

interface SidebarProps {
  currentPage: NavigationPage;
  onNavigate: (page: NavigationPage) => void;
  serviceCounts: {
    total: number;
    running: number;
  };
}

export const Sidebar: React.FC<SidebarProps> = ({
  currentPage,
  onNavigate,
  serviceCounts,
}) => {
  const navItems: { id: NavigationPage; label: string; icon: React.ReactNode; badge?: string }[] = [
    {
      id: "dashboard",
      label: "Dashboard",
      icon: <LayoutDashboard size={17} strokeWidth={1.7} />,
    },
    {
      id: "services",
      label: "Services",
      icon: <Server size={17} strokeWidth={1.7} />,
      badge: `${serviceCounts.running}/${serviceCounts.total}`,
    },
    {
      id: "logs",
      label: "Logs",
      icon: <Terminal size={17} strokeWidth={1.7} />,
    },
    {
      id: "settings",
      label: "Settings",
      icon: <Settings size={17} strokeWidth={1.7} />,
    },
  ];

  return (
    <aside style={styles.sidebar} className="glass-panel">
      {/* Brand & Logo Header */}
      <div style={styles.brandContainer}>
        <div style={styles.logoBadge}>
          <Layers size={18} strokeWidth={1.8} color="#ffffff" />
        </div>
        <div style={styles.brandInfo}>
          <span style={styles.brandName}>KAIRO</span>
          <span style={styles.brandTagline}>Runtime Manager</span>
        </div>
      </div>

      {/* Navigation Links */}
      <nav style={styles.nav}>
        <div style={styles.navSectionLabel}>NAVIGATION</div>
        {navItems.map((item) => {
          const isActive = currentPage === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onNavigate(item.id)}
              style={{
                ...styles.navButton,
                ...(isActive ? styles.navButtonActive : {}),
              }}
            >
              <span style={isActive ? styles.iconActive : styles.iconInactive}>
                {item.icon}
              </span>
              <span style={{ ...styles.navLabel, color: isActive ? "#ffffff" : "var(--text-secondary)" }}>
                {item.label}
              </span>
              {item.badge && (
                <span
                  style={{
                    ...styles.badge,
                    backgroundColor: isActive ? "rgba(255, 255, 255, 0.12)" : "rgba(255, 255, 255, 0.04)",
                  }}
                >
                  {item.badge}
                </span>
              )}
            </button>
          );
        })}
      </nav>

      {/* Sidebar Footer Metadata */}
      <div style={styles.footer}>
        <div style={styles.systemStatus}>
          <Activity size={13} strokeWidth={1.6} color="#22c55e" />
          <span style={styles.statusText}>Engine Ready</span>
        </div>
        <div style={styles.versionContainer}>
          <span style={styles.versionPill}>{APP_VERSION_LABEL}</span>
        </div>
      </div>
    </aside>
  );
};

const styles: Record<string, React.CSSProperties> = {
  sidebar: {
    width: "var(--sidebar-width)",
    minWidth: "var(--sidebar-width)",
    height: "100%",
    display: "flex",
    flexDirection: "column",
    borderRight: "1px solid var(--border-subtle)",
    borderTop: "none",
    borderBottom: "none",
    borderLeft: "none",
    backgroundColor: "var(--glass-bg-primary)",
    zIndex: 10,
  },
  brandContainer: {
    height: "var(--header-height)",
    display: "flex",
    alignItems: "center",
    gap: "12px",
    padding: "0 20px",
    borderBottom: "1px solid var(--border-subtle)",
  },
  logoBadge: {
    width: "32px",
    height: "32px",
    borderRadius: "var(--radius-sm)",
    backgroundColor: "rgba(255, 255, 255, 0.05)",
    border: "1px solid var(--border-medium)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
  },
  brandInfo: {
    display: "flex",
    flexDirection: "column",
  },
  brandName: {
    fontSize: "0.875rem",
    fontWeight: 600,
    letterSpacing: "-0.02em",
    color: "var(--text-primary)",
    whiteSpace: "nowrap",
  },
  brandTagline: {
    fontSize: "0.6875rem",
    color: "var(--text-muted)",
    letterSpacing: "0.01em",
  },
  nav: {
    flex: 1,
    padding: "18px 12px",
    display: "flex",
    flexDirection: "column",
    gap: "4px",
  },
  navSectionLabel: {
    fontSize: "0.625rem",
    fontWeight: 600,
    letterSpacing: "0.08em",
    color: "var(--text-dim)",
    padding: "0 10px 8px 10px",
  },
  navButton: {
    width: "100%",
    display: "flex",
    alignItems: "center",
    gap: "10px",
    padding: "9px 12px",
    borderRadius: "var(--radius-sm)",
    border: "1px solid transparent",
    backgroundColor: "transparent",
    cursor: "pointer",
    textAlign: "left",
    transition: "all var(--transition-fast)",
  },
  navButtonActive: {
    backgroundColor: "rgba(255, 255, 255, 0.07)",
    borderColor: "var(--border-subtle)",
  },
  iconActive: {
    color: "#ffffff",
    display: "flex",
    alignItems: "center",
  },
  iconInactive: {
    color: "var(--text-muted)",
    display: "flex",
    alignItems: "center",
  },
  navLabel: {
    fontSize: "0.8125rem",
    fontWeight: 500,
    flex: 1,
  },
  badge: {
    fontFamily: "var(--font-mono)",
    fontSize: "0.6875rem",
    color: "var(--text-secondary)",
    padding: "1px 6px",
    borderRadius: "var(--radius-xs)",
    border: "1px solid var(--border-subtle)",
  },
  footer: {
    padding: "16px 20px",
    borderTop: "1px solid var(--border-subtle)",
    display: "flex",
    flexDirection: "column",
    gap: "8px",
  },
  systemStatus: {
    display: "flex",
    alignItems: "center",
    gap: "6px",
  },
  statusText: {
    fontSize: "0.75rem",
    color: "var(--text-secondary)",
  },
  versionContainer: {
    display: "flex",
  },
  versionPill: {
    fontFamily: "var(--font-mono)",
    fontSize: "0.6875rem",
    color: "var(--text-muted)",
  },
};
