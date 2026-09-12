import React, { useState } from "react";
import { Search, RotateCw, Plus, Command } from "lucide-react";
import { NavigationPage } from "../../types";

interface HeaderProps {
  currentPage: NavigationPage;
  searchQuery: string;
  onSearchChange: (query: string) => void;
  onRefresh: () => void;
  onOpenAddService: () => void;
  isRefreshing?: boolean;
}

export const Header: React.FC<HeaderProps> = ({
  currentPage,
  searchQuery,
  onSearchChange,
  onRefresh,
  onOpenAddService,
  isRefreshing = false,
}) => {
  const [isFocused, setIsFocused] = useState(false);

  const pageDetails: Record<NavigationPage, { title: string; desc: string }> = {
    dashboard: {
      title: "Dashboard",
      desc: "Overview of local services, runtime telemetry, and health states",
    },
    services: {
      title: "Services",
      desc: "Manage and inspect configured local processes and command servers",
    },
    logs: {
      title: "Logs",
      desc: "Real-time process stdout/stderr stream console",
    },
    settings: {
      title: "Settings",
      desc: "System preferences, Windows integrations, and execution defaults",
    },
  };

  const { title, desc } = pageDetails[currentPage] || pageDetails.dashboard;

  return (
    <header style={styles.header} className="glass-panel">
      {/* Title & Description */}
      <div style={styles.titleContainer}>
        <h1 style={styles.title}>{title}</h1>
        <p style={styles.subtitle}>{desc}</p>
      </div>

      {/* Action Toolbar */}
      <div style={styles.toolbar}>
        {/* Quick Search */}
        <div
          style={{
            ...styles.searchBox,
            borderColor: isFocused ? "var(--border-focus)" : "var(--border-subtle)",
            backgroundColor: isFocused ? "rgba(255, 255, 255, 0.05)" : "rgba(255, 255, 255, 0.03)",
          }}
        >
          <Search size={15} strokeWidth={1.7} color="var(--text-muted)" />
          <input
            type="text"
            placeholder="Search services, ports, commands..."
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            onFocus={() => setIsFocused(true)}
            onBlur={() => setIsFocused(false)}
            style={styles.searchInput}
          />
          <div style={styles.shortcutPill}>
            <Command size={10} strokeWidth={1.8} style={{ marginRight: 2 }} />
            <span>K</span>
          </div>
        </div>

        {/* Refresh Button */}
        <button
          onClick={onRefresh}
          className="btn-icon"
          title="Refresh Status"
          aria-label="Refresh Status"
        >
          <RotateCw
            size={15}
            strokeWidth={1.7}
            className={isRefreshing ? "animate-spin" : ""}
          />
        </button>

        {/* Add Service Button */}
        <button onClick={onOpenAddService} className="btn-primary">
          <Plus size={15} strokeWidth={2.2} />
          <span>Add Service</span>
        </button>
      </div>
    </header>
  );
};

const styles: Record<string, React.CSSProperties> = {
  header: {
    height: "var(--header-height)",
    minHeight: "var(--header-height)",
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: "0 28px",
    borderBottom: "1px solid var(--border-subtle)",
    borderTop: "none",
    borderLeft: "none",
    borderRight: "none",
    backgroundColor: "var(--glass-bg-primary)",
    zIndex: 9,
  },
  titleContainer: {
    display: "flex",
    flexDirection: "column",
    gap: "2px",
  },
  title: {
    fontSize: "1.125rem",
    fontWeight: 600,
    letterSpacing: "-0.02em",
    color: "var(--text-primary)",
  },
  subtitle: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
  },
  toolbar: {
    display: "flex",
    alignItems: "center",
    gap: "12px",
  },
  searchBox: {
    display: "flex",
    alignItems: "center",
    gap: "8px",
    padding: "6px 12px",
    borderRadius: "var(--radius-sm)",
    border: "1px solid var(--border-subtle)",
    width: "280px",
    transition: "all var(--transition-fast)",
  },
  searchInput: {
    background: "transparent",
    border: "none",
    outline: "none",
    color: "var(--text-primary)",
    fontSize: "0.8125rem",
    width: "100%",
    fontFamily: "var(--font-sans)",
  },
  shortcutPill: {
    display: "flex",
    alignItems: "center",
    fontFamily: "var(--font-mono)",
    fontSize: "0.625rem",
    color: "var(--text-muted)",
    padding: "2px 4px",
    borderRadius: "var(--radius-xs)",
    backgroundColor: "rgba(255, 255, 255, 0.05)",
    border: "1px solid var(--border-subtle)",
  },
};
