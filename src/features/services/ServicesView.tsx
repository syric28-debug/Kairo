import React, { useState } from "react";
import { Search, Plus, ServerOff, X } from "lucide-react";
import { ServiceState, StatusFilter } from "../../types";
import { ServiceCard } from "./ServiceCard";

interface ServicesViewProps {
  services: ServiceState[];
  searchQuery: string;
  onSearchChange: (query: string) => void;
  onStart: (id: string) => void;
  onStop: (id: string) => void;
  onRestart: (id: string) => void;
  onEdit: (service: ServiceState) => void;
  onDelete: (service: ServiceState) => void;
  onOpenAddService: () => void;
}

export const ServicesView: React.FC<ServicesViewProps> = ({
  services,
  searchQuery,
  onSearchChange,
  onStart,
  onStop,
  onRestart,
  onEdit,
  onDelete,
  onOpenAddService,
}) => {
  const [statusFilter, setStatusFilter] = useState<StatusFilter>("all");

  // Filtering by search (name, command, port, description) & status
  const filteredServices = services.filter((svc) => {
    // 1. Status Filter
    if (statusFilter !== "all") {
      if (statusFilter === "running") {
        if (svc.status !== "running" && svc.status !== "ready") {
          return false;
        }
      } else if (svc.status !== statusFilter) {
        return false;
      }
    }

    // 2. Search Query (name, command, port, description, status)
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      const nameMatch = svc.config.name.toLowerCase().includes(q);
      const descMatch = (svc.config.description || "").toLowerCase().includes(q);
      const cmdMatch = `${svc.config.executable} ${svc.config.arguments.join(" ")}`
        .toLowerCase()
        .includes(q);
      const portMatch = svc.config.port ? String(svc.config.port).includes(q) : false;
      const statusMatch = svc.status.toLowerCase().includes(q);

      return nameMatch || descMatch || cmdMatch || portMatch || statusMatch;
    }

    return true;
  });

  const counts = {
    all: services.length,
    running: services.filter((s) => s.status === "running" || s.status === "ready").length,
    stopped: services.filter((s) => s.status === "stopped").length,
    failed: services.filter((s) => s.status === "failed").length,
  };

  const filterTabs: { id: StatusFilter; label: string; count: number }[] = [
    { id: "all", label: "All", count: counts.all },
    { id: "running", label: "Running", count: counts.running },
    { id: "stopped", label: "Stopped", count: counts.stopped },
    { id: "failed", label: "Failed", count: counts.failed },
  ];

  return (
    <div style={styles.container} className="animate-fade-in">
      {/* Control Bar: Filters & Quick Search */}
      <div style={styles.controlBar}>
        {/* Status Filter Tabs */}
        <div style={styles.filterTabs}>
          {filterTabs.map((tab) => {
            const isActive = statusFilter === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => setStatusFilter(tab.id)}
                style={{
                  ...styles.filterButton,
                  ...(isActive ? styles.filterButtonActive : {}),
                }}
              >
                <span>{tab.label}</span>
                <span
                  style={{
                    ...styles.filterCount,
                    backgroundColor: isActive
                      ? "rgba(255, 255, 255, 0.15)"
                      : "rgba(255, 255, 255, 0.05)",
                  }}
                >
                  {tab.count}
                </span>
              </button>
            );
          })}
        </div>

        {/* Search & Action Right */}
        <div style={styles.rightControls}>
          <div style={styles.inlineSearch}>
            <Search size={14} strokeWidth={1.7} color="var(--text-muted)" />
            <input
              type="text"
              placeholder="Filter list..."
              value={searchQuery}
              onChange={(e) => onSearchChange(e.target.value)}
              style={styles.inlineInput}
            />
            {searchQuery && (
              <button
                onClick={() => onSearchChange("")}
                style={styles.clearBtn}
                title="Clear filter"
              >
                <X size={12} strokeWidth={2} />
              </button>
            )}
          </div>

          <button onClick={onOpenAddService} className="btn-secondary">
            <Plus size={14} strokeWidth={2} />
            <span>New Service</span>
          </button>
        </div>
      </div>

      {/* Services Grid or Empty State */}
      {filteredServices.length > 0 ? (
        <div style={styles.grid}>
          {filteredServices.map((service) => (
            <ServiceCard
              key={service.config.id}
              service={service}
              onStart={onStart}
              onStop={onStop}
              onRestart={onRestart}
              onEdit={onEdit}
              onDelete={onDelete}
            />
          ))}
        </div>
      ) : (
        /* Empty State */
        <div style={styles.emptyContainer} className="glass-card">
          <div style={styles.emptyIconBox}>
            <ServerOff size={28} strokeWidth={1.5} color="var(--text-muted)" />
          </div>
          <h3 style={styles.emptyTitle}>No services found</h3>
          <p style={styles.emptyDesc}>
            {searchQuery || statusFilter !== "all"
              ? "No configured services match your current query or filter criteria."
              : "You have not configured any local services yet."}
          </p>
          <div style={styles.emptyActions}>
            {(searchQuery || statusFilter !== "all") && (
              <button
                onClick={() => {
                  onSearchChange("");
                  setStatusFilter("all");
                }}
                className="btn-secondary"
              >
                Reset Filters
              </button>
            )}
            <button onClick={onOpenAddService} className="btn-primary">
              <Plus size={14} strokeWidth={2} />
              <span>Add Your First Service</span>
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: "flex",
    flexDirection: "column",
    gap: "20px",
  },
  controlBar: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    gap: "16px",
    flexWrap: "wrap",
  },
  filterTabs: {
    display: "flex",
    alignItems: "center",
    gap: "4px",
    padding: "3px",
    backgroundColor: "rgba(255, 255, 255, 0.03)",
    border: "1px solid var(--border-subtle)",
    borderRadius: "var(--radius-sm)",
  },
  filterButton: {
    display: "flex",
    alignItems: "center",
    gap: "6px",
    padding: "5px 12px",
    fontSize: "0.75rem",
    fontWeight: 500,
    color: "var(--text-secondary)",
    backgroundColor: "transparent",
    border: "none",
    borderRadius: "var(--radius-xs)",
    cursor: "pointer",
    transition: "all var(--transition-fast)",
  },
  filterButtonActive: {
    backgroundColor: "rgba(255, 255, 255, 0.09)",
    color: "#ffffff",
  },
  filterCount: {
    fontFamily: "var(--font-mono)",
    fontSize: "0.6875rem",
    padding: "1px 5px",
    borderRadius: "var(--radius-xs)",
  },
  rightControls: {
    display: "flex",
    alignItems: "center",
    gap: "12px",
  },
  inlineSearch: {
    display: "flex",
    alignItems: "center",
    gap: "8px",
    padding: "6px 10px",
    backgroundColor: "rgba(255, 255, 255, 0.03)",
    border: "1px solid var(--border-subtle)",
    borderRadius: "var(--radius-sm)",
    width: "220px",
  },
  inlineInput: {
    background: "transparent",
    border: "none",
    outline: "none",
    color: "var(--text-primary)",
    fontSize: "0.75rem",
    width: "100%",
    fontFamily: "var(--font-sans)",
  },
  clearBtn: {
    background: "transparent",
    border: "none",
    color: "var(--text-muted)",
    cursor: "pointer",
    display: "flex",
    alignItems: "center",
    padding: 0,
  },
  grid: {
    display: "grid",
    gridTemplateColumns: "repeat(auto-fill, minmax(380px, 1fr))",
    gap: "18px",
  },
  emptyContainer: {
    padding: "56px 24px",
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "center",
    textAlign: "center",
    gap: "12px",
  },
  emptyIconBox: {
    width: "56px",
    height: "56px",
    borderRadius: "var(--radius-md)",
    backgroundColor: "rgba(255, 255, 255, 0.03)",
    border: "1px solid var(--border-subtle)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    marginBottom: "4px",
  },
  emptyTitle: {
    fontSize: "1rem",
    fontWeight: 600,
    color: "var(--text-primary)",
  },
  emptyDesc: {
    fontSize: "0.8125rem",
    color: "var(--text-muted)",
    maxWidth: "400px",
    lineHeight: 1.5,
  },
  emptyActions: {
    display: "flex",
    alignItems: "center",
    gap: "10px",
    marginTop: "8px",
  },
};
