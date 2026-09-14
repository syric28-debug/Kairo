import React, { useState, useEffect, useCallback, useRef } from "react";
import { Sidebar } from "./components/layout/Sidebar";
import { Header } from "./components/layout/Header";
import { DashboardView } from "./features/dashboard/DashboardView";
import { ServicesView } from "./features/services/ServicesView";
import { LogsView } from "./features/logs/LogsView";
import { SettingsView } from "./features/settings/SettingsView";
import { AddServiceModal, DeleteServiceModal } from "./features/services";
import {
  listServices,
  createService,
  updateService,
  deleteService,
  startService,
  stopService,
  restartService,
  listServiceRuntimeStates,
  onServiceRuntimeUpdated,
  CreateServiceDto,
  UpdateServiceDto,
} from "./services/serviceManager";
import { onStartupProgress, getStartupSettings } from "./services/settingsManager";
import { ServiceState, NavigationPage, StartupProgress } from "./types";
import { UpdateAvailableBanner, UpdateInstallDialog } from "./features/updates/UpdateNotifications";
import { scheduleStartupUpdateCheck } from "./services/updateStore";
import "./styles/index.css";

export const App: React.FC = () => {
  const [currentPage, setCurrentPage] = useState<NavigationPage>("dashboard");
  const [services, setServices] = useState<ServiceState[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [globalError, setGlobalError] = useState<string | null>(null);

  // Phase 6: startup progress banner state
  const [startupProgress, setStartupProgress] = useState<StartupProgress | null>(null);
  const startupDismissTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Modal State
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [editingService, setEditingService] = useState<ServiceState | null>(null);

  // Delete Confirmation State
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);
  const [serviceToDelete, setServiceToDelete] = useState<ServiceState | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  // Phase 12: optional background update check — never blocks startup and
  // failures are silent (status text only, no popups).
  useEffect(() => {
    let cancelled = false;
    getStartupSettings()
      .then((settings) => {
        if (!cancelled) scheduleStartupUpdateCheck(settings.updateAutoCheck ?? true);
      })
      .catch(() => {
        if (!cancelled) scheduleStartupUpdateCheck(true);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // Load services from repository and reconcile with live process states
  const loadServices = useCallback(async () => {
    try {
      setGlobalError(null);
      const [configs, runtimeStates] = await Promise.all([
        listServices(),
        listServiceRuntimeStates().catch(() => []),
      ]);

      const runtimeMap = new Map(runtimeStates.map((r) => [r.serviceId, r]));

      setServices((prev) => {
        const prevMap = new Map(prev.map((s) => [s.config.id, s]));

        return configs.map((config) => {
          const prevService = prevMap.get(config.id);
          const liveRuntime = runtimeMap.get(config.id);

          if (liveRuntime) {
            return {
              config,
              status: liveRuntime.state,
              metrics: liveRuntime.pid
                ? {
                    pid: liveRuntime.pid,
                    cpuPercent: prevService?.metrics?.cpuPercent,
                    memoryBytes: prevService?.metrics?.memoryBytes,
                    uptimeSeconds: prevService?.metrics?.uptimeSeconds,
                  }
                : undefined,
              lastStartedAt: liveRuntime.startedAt || prevService?.lastStartedAt,
              errorMessage: liveRuntime.errorMessage || undefined,
            };
          }

          if (prevService) {
            return {
              ...prevService,
              config,
            };
          }

          return {
            config,
            status: "stopped",
            metrics: undefined,
          };
        });
      });
    } catch (err: unknown) {
      console.error("Failed to load services:", err);
      const msg = err instanceof Error ? err.message : String(err);
      setGlobalError(`Failed to load persisted services: ${msg}`);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadServices();
  }, [loadServices]);

  // Centralized subscription to live background watchdog notifications
  useEffect(() => {
    let unlistenFn: (() => void) | undefined;

    const setupListener = async () => {
      try {
        unlistenFn = await onServiceRuntimeUpdated((runtime) => {
          setServices((prev) =>
            prev.map((s) => {
              if (s.config.id === runtime.serviceId) {
                return {
                  ...s,
                  status: runtime.state,
                  metrics: runtime.pid ? { pid: runtime.pid } : undefined,
                  lastStartedAt: runtime.startedAt || s.lastStartedAt,
                  errorMessage: runtime.errorMessage || undefined,
                };
              }
              return s;
            })
          );
        });
      } catch (err) {
        console.warn("Could not register watchdog event listener:", err);
      }
    };

    setupListener();

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, []);

  // Phase 6: subscribe to startup-progress events for the auto-start banner.
  useEffect(() => {
    let unlistenFn: (() => void) | undefined;

    const setupStartupListener = async () => {
      try {
        unlistenFn = await onStartupProgress((progress) => {
          if (progress.total === 0) return; // Nothing to show for empty auto-start.
          setStartupProgress(progress);
          if (progress.isComplete) {
            // Auto-dismiss the banner 3s after completion.
            if (startupDismissTimer.current) clearTimeout(startupDismissTimer.current);
            startupDismissTimer.current = setTimeout(() => {
              setStartupProgress(null);
            }, 3000);
          }
        });
      } catch (err) {
        console.warn("Could not register startup-progress listener:", err);
      }
    };

    setupStartupListener();

    return () => {
      if (unlistenFn) unlistenFn();
      if (startupDismissTimer.current) clearTimeout(startupDismissTimer.current);
    };
  }, []);

  // Live Counts for Dashboard
  const serviceCounts = {
    total: services.length,
    running: services.filter((s) => s.status === "running" || s.status === "ready").length,
    stopped: services.filter((s) => s.status === "stopped").length,
    failed: services.filter((s) => s.status === "failed").length,
  };

  // Real Process Lifecycle Handlers
  const handleStart = async (id: string) => {
    // 1. Immediate optimistic transition to 'starting'
    setServices((prev) =>
      prev.map((s) =>
        s.config.id === id ? { ...s, status: "starting", errorMessage: undefined } : s
      )
    );

    try {
      // 2. Real backend execution
      const runtime = await startService(id);
      setServices((prev) =>
        prev.map((s) => {
          if (s.config.id === id) {
            return {
              ...s,
              status: runtime.state,
              metrics: runtime.pid
                ? {
                    pid: runtime.pid,
                    cpuPercent: 0,
                    memoryBytes: 0,
                    uptimeSeconds: 0,
                  }
                : undefined,
              lastStartedAt: runtime.startedAt || new Date().toISOString(),
              errorMessage: runtime.errorMessage || undefined,
            };
          }
          return s;
        })
      );
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setServices((prev) =>
        prev.map((s) =>
          s.config.id === id
            ? {
                ...s,
                status: "failed",
                errorMessage: msg,
              }
            : s
        )
      );
    }
  };

  const handleStop = async (id: string) => {
    // 1. Immediate transition to 'stopping'
    setServices((prev) =>
      prev.map((s) => (s.config.id === id ? { ...s, status: "stopping" } : s))
    );

    try {
      // 2. Real backend process termination & wait
      const runtime = await stopService(id);
      setServices((prev) =>
        prev.map((s) => {
          if (s.config.id === id) {
            return {
              ...s,
              status: runtime.state,
              metrics: undefined,
              lastExitedAt: new Date().toISOString(),
              errorMessage: runtime.errorMessage || undefined,
            };
          }
          return s;
        })
      );
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setServices((prev) =>
        prev.map((s) =>
          s.config.id === id
            ? {
                ...s,
                status: "running",
                errorMessage: msg,
              }
            : s
        )
      );
    }
  };

  const handleRestart = async (id: string) => {
    // 1. Immediate transition to 'restarting'
    setServices((prev) =>
      prev.map((s) =>
        s.config.id === id ? { ...s, status: "restarting", errorMessage: undefined } : s
      )
    );

    try {
      // 2. Real backend restart (stops if running, waits, starts anew)
      const runtime = await restartService(id);
      setServices((prev) =>
        prev.map((s) => {
          if (s.config.id === id) {
            return {
              ...s,
              status: runtime.state,
              metrics: runtime.pid
                ? {
                    pid: runtime.pid,
                    cpuPercent: 0,
                    memoryBytes: 0,
                    uptimeSeconds: 0,
                  }
                : undefined,
              lastStartedAt: runtime.startedAt || new Date().toISOString(),
              errorMessage: runtime.errorMessage || undefined,
            };
          }
          return s;
        })
      );
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setServices((prev) =>
        prev.map((s) =>
          s.config.id === id
            ? {
                ...s,
                status: "failed",
                errorMessage: msg,
              }
            : s
        )
      );
    }
  };

  // Service Management Handlers
  const handleOpenAddService = () => {
    setEditingService(null);
    setIsAddModalOpen(true);
  };

  const handleEditService = (service: ServiceState) => {
    setEditingService(service);
    setIsAddModalOpen(true);
  };

  const handleSaveService = async (data: {
    dto: CreateServiceDto | UpdateServiceDto;
    id?: string;
  }) => {
    if (data.id) {
      // Update existing service
      const updatedConfig = await updateService(data.id, data.dto as UpdateServiceDto);
      setServices((prev) =>
        prev.map((s) =>
          s.config.id === data.id
            ? { ...s, config: updatedConfig }
            : s
        )
      );
    } else {
      // Create new service
      const newConfig = await createService(data.dto as CreateServiceDto);
      const newServiceState: ServiceState = {
        config: newConfig,
        status: "stopped",
        metrics: undefined,
      };
      setServices((prev) => [newServiceState, ...prev]);
      setCurrentPage("services");
    }
  };

  const handleDeleteRequest = (service: ServiceState) => {
    setServiceToDelete(service);
    setIsDeleteModalOpen(true);
  };

  const handleConfirmDelete = async (id: string) => {
    setIsDeleting(true);
    try {
      const removed = await deleteService(id);
      if (removed) {
        setServices((prev) => prev.filter((s) => s.config.id !== id));
        setIsDeleteModalOpen(false);
        setServiceToDelete(null);
      }
    } catch (err: unknown) {
      console.error("Failed to delete service:", err);
      const msg = err instanceof Error ? err.message : String(err);
      setGlobalError(`Failed to delete service: ${msg}`);
    } finally {
      setIsDeleting(false);
    }
  };

  const handleRefresh = async () => {
    setIsRefreshing(true);
    await loadServices();
    setTimeout(() => {
      setIsRefreshing(false);
    }, 400);
  };

  return (
    <div style={styles.appRoot}>
      {/* Background Subtle Gradient Glow */}
      <div style={styles.ambientGlow} />

      {/* Main Layout: Sidebar */}
      <Sidebar
        currentPage={currentPage}
        onNavigate={setCurrentPage}
        serviceCounts={serviceCounts}
      />

      {/* Main Layout: Content Frame */}
      <div style={styles.contentFrame}>
        {/* Top Header */}
        <Header
          currentPage={currentPage}
          searchQuery={searchQuery}
          onSearchChange={setSearchQuery}
          onRefresh={handleRefresh}
          onOpenAddService={handleOpenAddService}
          isRefreshing={isRefreshing}
        />

        {/* Phase 6: Startup Progress Banner */}
        {startupProgress && (
          <div style={{
            ...styles.startupBanner,
            opacity: startupProgress.isComplete ? 0.7 : 1,
          }}>
            <span style={styles.startupBannerDot}
              className={startupProgress.isComplete ? undefined : "pulse-dot"} />
            <span style={styles.startupBannerText}>
              {startupProgress.isComplete
                ? `Auto-start complete — ${startupProgress.started} started${
                    startupProgress.failed > 0
                      ? `, ${startupProgress.failed} failed`
                      : ""
                  }`
                : `Auto-starting services (${startupProgress.started + startupProgress.failed}/${startupProgress.total})…`}
            </span>
            <button
              style={styles.startupBannerDismiss}
              onClick={() => setStartupProgress(null)}
              aria-label="Dismiss startup banner"
            >✕</button>
          </div>
        )}

        {/* Global Error Notice if any */}
        {globalError && (
          <div style={styles.errorNotice}>
            <span>{globalError}</span>
            <button
              onClick={() => setGlobalError(null)}
              style={styles.errorDismiss}
            >
              Dismiss
            </button>
          </div>
        )}

        {/* Phase 12: Update-available banner (non-intrusive, user-controlled) */}
        <UpdateAvailableBanner />

        {/* Dynamic Viewport Container */}
        <main style={styles.mainContent}>
          {isLoading ? (
            <div style={styles.loadingContainer}>
              <span style={styles.loadingText}>Loading services...</span>
            </div>
          ) : (
            <>
              {currentPage === "dashboard" && (
                <DashboardView
                  services={services}
                  onNavigate={setCurrentPage}
                  onStart={handleStart}
                  onStop={handleStop}
                  onEdit={handleEditService}
                />
              )}

              {currentPage === "services" && (
                <ServicesView
                  services={services}
                  searchQuery={searchQuery}
                  onSearchChange={setSearchQuery}
                  onStart={handleStart}
                  onStop={handleStop}
                  onRestart={handleRestart}
                  onEdit={handleEditService}
                  onDelete={handleDeleteRequest}
                  onOpenAddService={handleOpenAddService}
                />
              )}

              {currentPage === "logs" && <LogsView services={services} />}

              {currentPage === "settings" && <SettingsView />}
            </>
          )}
        </main>
      </div>

      {/* Add / Edit Service Modal */}
      <AddServiceModal
        isOpen={isAddModalOpen}
        onClose={() => setIsAddModalOpen(false)}
        onSave={handleSaveService}
        initialService={editingService}
      />

      {/* Phase 12: Update install dialog (release notes, progress, install) */}
      <UpdateInstallDialog />

      {/* Delete Confirmation Modal */}
      <DeleteServiceModal
        isOpen={isDeleteModalOpen}
        service={serviceToDelete}
        onClose={() => {
          if (!isDeleting) {
            setIsDeleteModalOpen(false);
            setServiceToDelete(null);
          }
        }}
        onConfirm={handleConfirmDelete}
        isDeleting={isDeleting}
      />
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  appRoot: {
    display: "flex",
    width: "100vw",
    height: "100vh",
    backgroundColor: "var(--bg-canvas)",
    color: "var(--text-primary)",
    position: "relative",
    overflow: "hidden",
  },
  ambientGlow: {
    position: "absolute",
    top: "-15%",
    right: "10%",
    width: "700px",
    height: "600px",
    borderRadius: "50%",
    background: "radial-gradient(circle, rgba(255, 255, 255, 0.025) 0%, rgba(0, 0, 0, 0) 70%)",
    pointerEvents: "none",
    zIndex: 0,
  },
  contentFrame: {
    flex: 1,
    display: "flex",
    flexDirection: "column",
    height: "100%",
    position: "relative",
    zIndex: 1,
    overflow: "hidden",
  },
  mainContent: {
    flex: 1,
    padding: "24px 28px",
    overflowY: "auto",
  },
  errorNotice: {
    margin: "12px 28px 0",
    padding: "10px 16px",
    backgroundColor: "rgba(239, 68, 68, 0.12)",
    border: "1px solid rgba(239, 68, 68, 0.3)",
    borderRadius: "var(--radius-sm)",
    color: "#fca5a5",
    fontSize: "0.8125rem",
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
  },
  errorDismiss: {
    background: "transparent",
    border: "none",
    color: "#f87171",
    cursor: "pointer",
    fontSize: "0.75rem",
    textDecoration: "underline",
  },
  loadingContainer: {
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    height: "60vh",
  },
  loadingText: {
    fontSize: "0.875rem",
    color: "var(--text-muted)",
  },
  startupBanner: {
    margin: "10px 28px 0",
    padding: "8px 14px",
    backgroundColor: "rgba(255, 255, 255, 0.04)",
    backdropFilter: "blur(8px)",
    border: "1px solid rgba(255, 255, 255, 0.10)",
    borderRadius: "var(--radius-sm)",
    display: "flex",
    alignItems: "center",
    gap: "10px",
    transition: "opacity 0.6s ease",
  },
  startupBannerDot: {
    width: "7px",
    height: "7px",
    borderRadius: "50%",
    backgroundColor: "rgba(255,255,255,0.55)",
    flexShrink: 0,
  },
  startupBannerText: {
    flex: 1,
    fontSize: "0.75rem",
    fontFamily: "var(--font-mono)",
    color: "var(--text-secondary)",
    letterSpacing: "0.01em",
  },
  startupBannerDismiss: {
    background: "transparent",
    border: "none",
    color: "var(--text-muted)",
    cursor: "pointer",
    fontSize: "0.75rem",
    lineHeight: 1,
    padding: "2px 4px",
  },
};

export default App;
