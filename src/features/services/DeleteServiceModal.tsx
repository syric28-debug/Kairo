import React from "react";
import { AlertTriangle, Trash2, X } from "lucide-react";
import { ServiceState } from "../../types";

interface DeleteServiceModalProps {
  isOpen: boolean;
  service: ServiceState | null;
  onClose: () => void;
  onConfirm: (id: string) => void;
  isDeleting?: boolean;
}

export const DeleteServiceModal: React.FC<DeleteServiceModalProps> = ({
  isOpen,
  service,
  onClose,
  onConfirm,
  isDeleting = false,
}) => {
  if (!isOpen || !service) return null;

  return (
    <div style={styles.overlay} onClick={onClose}>
      <div
        style={styles.modal}
        className="glass-panel animate-fade-in"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div style={styles.header}>
          <div style={styles.headerLeft}>
            <div style={styles.warningIconBox}>
              <AlertTriangle size={18} strokeWidth={2} color="#f87171" />
            </div>
            <div>
              <h3 style={styles.title}>Delete Service</h3>
              <p style={styles.subtitle}>Confirm permanent configuration removal</p>
            </div>
          </div>
          <button onClick={onClose} className="btn-icon" aria-label="Close modal">
            <X size={16} strokeWidth={1.8} />
          </button>
        </div>

        {/* Content Body */}
        <div style={styles.body}>
          <p style={styles.promptText}>
            Are you sure you want to delete{" "}
            <strong style={styles.serviceNameHighlight}>
              {service.config.name}
            </strong>
            ?
          </p>
          <div style={styles.warningBox}>
            <p style={styles.warningText}>
              This action will permanently delete this service configuration from your local
              persistence store (<code>services.json</code>).
            </p>
          </div>
          <div style={styles.metaSummary}>
            <div style={styles.metaRow}>
              <span style={styles.metaLabel}>Executable:</span>
              <span style={styles.metaValue} className="code-pill">
                {service.config.executable}
              </span>
            </div>
            {service.config.port && (
              <div style={styles.metaRow}>
                <span style={styles.metaLabel}>Port:</span>
                <span style={styles.metaValue}>:{service.config.port}</span>
              </div>
            )}
          </div>
        </div>

        {/* Footer */}
        <div style={styles.footer}>
          <button
            type="button"
            onClick={onClose}
            className="btn-secondary"
            disabled={isDeleting}
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={() => onConfirm(service.config.id)}
            className="btn-danger"
            disabled={isDeleting}
            style={styles.deleteBtn}
          >
            <Trash2 size={14} strokeWidth={1.8} />
            <span>{isDeleting ? "Deleting..." : "Delete Service"}</span>
          </button>
        </div>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: "fixed",
    inset: 0,
    backgroundColor: "rgba(0, 0, 0, 0.75)",
    backdropFilter: "blur(8px)",
    WebkitBackdropFilter: "blur(8px)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    zIndex: 110,
  },
  modal: {
    width: "480px",
    maxWidth: "92vw",
    borderRadius: "var(--radius-lg)",
    backgroundColor: "var(--glass-bg-primary)",
    border: "1px solid var(--border-medium)",
    boxShadow: "var(--shadow-glass)",
    overflow: "hidden",
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
  warningIconBox: {
    width: "36px",
    height: "36px",
    borderRadius: "var(--radius-sm)",
    backgroundColor: "rgba(239, 68, 68, 0.1)",
    border: "1px solid rgba(239, 68, 68, 0.25)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
  },
  title: {
    fontSize: "0.9375rem",
    fontWeight: 600,
    color: "var(--text-primary)",
  },
  subtitle: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
  },
  body: {
    padding: "22px 24px",
    display: "flex",
    flexDirection: "column",
    gap: "16px",
  },
  promptText: {
    fontSize: "0.875rem",
    color: "var(--text-primary)",
    lineHeight: 1.5,
  },
  serviceNameHighlight: {
    color: "#ffffff",
    fontWeight: 600,
  },
  warningBox: {
    padding: "12px 14px",
    backgroundColor: "rgba(239, 68, 68, 0.06)",
    border: "1px solid rgba(239, 68, 68, 0.18)",
    borderRadius: "var(--radius-sm)",
  },
  warningText: {
    fontSize: "0.75rem",
    color: "#f87171",
    lineHeight: 1.4,
  },
  metaSummary: {
    display: "flex",
    flexDirection: "column",
    gap: "8px",
    padding: "12px",
    backgroundColor: "rgba(255, 255, 255, 0.02)",
    border: "1px solid var(--border-subtle)",
    borderRadius: "var(--radius-sm)",
  },
  metaRow: {
    display: "flex",
    alignItems: "center",
    gap: "8px",
    fontSize: "0.75rem",
  },
  metaLabel: {
    color: "var(--text-muted)",
    width: "70px",
  },
  metaValue: {
    color: "var(--text-secondary)",
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap",
    flex: 1,
  },
  footer: {
    display: "flex",
    alignItems: "center",
    justifyContent: "flex-end",
    gap: "10px",
    padding: "16px 24px",
    borderTop: "1px solid var(--border-subtle)",
    backgroundColor: "rgba(0, 0, 0, 0.2)",
  },
  deleteBtn: {
    display: "flex",
    alignItems: "center",
    gap: "6px",
    padding: "8px 16px",
    borderRadius: "var(--radius-sm)",
    backgroundColor: "rgba(239, 68, 68, 0.18)",
    border: "1px solid rgba(239, 68, 68, 0.4)",
    color: "#fca5a5",
    fontSize: "0.8125rem",
    fontWeight: 500,
    cursor: "pointer",
    transition: "all var(--transition-fast)",
  },
};
