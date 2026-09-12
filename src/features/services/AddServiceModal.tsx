import React, { useState, useEffect } from "react";
import { X, Save, Layers, AlertCircle } from "lucide-react";
import { ServiceState } from "../../types";
import { CreateServiceDto, UpdateServiceDto, validateServiceInput } from "../../services/serviceManager";

interface AddServiceModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (data: { dto: CreateServiceDto | UpdateServiceDto; id?: string }) => Promise<void>;
  initialService?: ServiceState | null;
}

export const AddServiceModal: React.FC<AddServiceModalProps> = ({
  isOpen,
  onClose,
  onSave,
  initialService,
}) => {
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [executable, setExecutable] = useState("");
  const [argumentsStr, setArgumentsStr] = useState("");
  const [workingDirectory, setWorkingDirectory] = useState("C:\\");
  const [port, setPort] = useState<string>("");
  const [envVarsStr, setEnvVarsStr] = useState("");
  const [healthCheck, setHealthCheck] = useState("");
  const [autoStart, setAutoStart] = useState(false);
  const [autoRestart, setAutoRestart] = useState(false);

  const [errors, setErrors] = useState<Record<string, string>>({});
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  useEffect(() => {
    if (initialService) {
      setName(initialService.config.name);
      setDescription(initialService.config.description || "");
      setExecutable(initialService.config.executable);
      setArgumentsStr(initialService.config.arguments.join(" "));
      setWorkingDirectory(initialService.config.workingDirectory || "C:\\");
      setPort(
        initialService.config.port !== null && initialService.config.port !== undefined
          ? String(initialService.config.port)
          : ""
      );
      setHealthCheck(initialService.config.healthCheck || "");
      setAutoStart(initialService.config.autoStart);
      setAutoRestart(initialService.config.autoRestart);

      // Format env vars to KEY=VALUE lines
      if (initialService.config.environmentVariables) {
        const lines = Object.entries(initialService.config.environmentVariables).map(
          ([k, v]) => `${k}=${v}`
        );
        setEnvVarsStr(lines.join("\n"));
      } else {
        setEnvVarsStr("");
      }
    } else {
      setName("");
      setDescription("");
      setExecutable("");
      setArgumentsStr("");
      setWorkingDirectory("C:\\");
      setPort("");
      setEnvVarsStr("");
      setHealthCheck("");
      setAutoStart(false);
      setAutoRestart(false);
    }
    setErrors({});
    setSubmitError(null);
  }, [initialService, isOpen]);

  if (!isOpen) return null;

  const parseEnvVars = (raw: string): Record<string, string> => {
    const map: Record<string, string> = {};
    const lines = raw.split(/\r?\n/);
    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith("#")) continue;
      const eqIdx = trimmed.indexOf("=");
      if (eqIdx > 0) {
        const key = trimmed.substring(0, eqIdx).trim();
        const value = trimmed.substring(eqIdx + 1).trim();
        if (key) map[key] = value;
      }
    }
    return map;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitError(null);

    const parsedPort = port.trim() ? parseInt(port.trim(), 10) : null;
    const validation = validateServiceInput({
      name,
      executable,
      description,
      port: parsedPort,
      workingDirectory,
    });

    if (!validation.valid) {
      setErrors(validation.errors);
      return;
    }

    const parsedArgs = argumentsStr
      .trim()
      .split(/\s+/)
      .filter((arg) => arg.length > 0);

    const envMap = parseEnvVars(envVarsStr);

    const dto: CreateServiceDto | UpdateServiceDto = {
      name: name.trim(),
      description: description.trim(),
      executable: executable.trim(),
      arguments: parsedArgs,
      workingDirectory: workingDirectory.trim() || "C:\\",
      environmentVariables: envMap,
      port: parsedPort,
      autoStart,
      autoRestart,
      healthCheck: healthCheck.trim() || null,
    };

    setIsSubmitting(true);
    try {
      await onSave({
        dto,
        id: initialService?.config.id,
      });
      onClose();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setSubmitError(msg);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div style={styles.overlay} onClick={onClose}>
      <div
        style={styles.modal}
        className="glass-panel animate-fade-in"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Modal Header */}
        <div style={styles.header}>
          <div style={styles.headerLeft}>
            <div style={styles.headerIcon}>
              <Layers size={18} strokeWidth={1.8} />
            </div>
            <div>
              <h2 style={styles.title}>
                {initialService ? "Edit Service Configuration" : "Add Generic Service"}
              </h2>
              <p style={styles.subtitle}>
                Persist executable, arguments, environment and lifecycle parameters
              </p>
            </div>
          </div>
          <button onClick={onClose} className="btn-icon" aria-label="Close modal">
            <X size={16} strokeWidth={1.8} />
          </button>
        </div>

        {/* Global Error Banner */}
        {submitError && (
          <div style={styles.errorBanner}>
            <AlertCircle size={14} strokeWidth={2} color="#f87171" />
            <span style={styles.errorBannerText}>{submitError}</span>
          </div>
        )}

        {/* Modal Form */}
        <form onSubmit={handleSubmit} style={styles.form}>
          <div style={styles.formGrid}>
            {/* Service Name */}
            <div style={styles.fieldGroup}>
              <label style={styles.label}>
                Service Name <span style={{ color: "#ef4444" }}>*</span>
              </label>
              <input
                type="text"
                className={`input-glass ${errors.name ? "input-error" : ""}`}
                placeholder="e.g. Local Redis Server"
                value={name}
                maxLength={100}
                onChange={(e) => {
                  setName(e.target.value);
                  if (errors.name) setErrors((prev) => ({ ...prev, name: "" }));
                }}
                required
              />
              {errors.name && <span style={styles.errorText}>{errors.name}</span>}
            </div>

            {/* Target Port */}
            <div style={styles.fieldGroup}>
              <label style={styles.label}>Port (1–65535)</label>
              <input
                type="number"
                min={1}
                max={65535}
                className={`input-glass ${errors.port ? "input-error" : ""}`}
                placeholder="e.g. 6379"
                value={port}
                onChange={(e) => {
                  setPort(e.target.value);
                  if (errors.port) setErrors((prev) => ({ ...prev, port: "" }));
                }}
              />
              {errors.port && <span style={styles.errorText}>{errors.port}</span>}
            </div>
          </div>

          {/* Description */}
          <div style={styles.fieldGroup}>
            <label style={styles.label}>Description</label>
            <input
              type="text"
              className="input-glass"
              placeholder="Brief description of service purpose..."
              value={description}
              maxLength={500}
              onChange={(e) => setDescription(e.target.value)}
            />
          </div>

          {/* Executable Path */}
          <div style={styles.fieldGroup}>
            <label style={styles.label}>
              Executable Path or Command <span style={{ color: "#ef4444" }}>*</span>
            </label>
            <input
              type="text"
              className={`input-glass ${errors.executable ? "input-error" : ""}`}
              style={{ fontFamily: "var(--font-mono)" }}
              placeholder="C:\Tools\redis-server.exe or binary name"
              value={executable}
              maxLength={1024}
              onChange={(e) => {
                setExecutable(e.target.value);
                if (errors.executable) setErrors((prev) => ({ ...prev, executable: "" }));
              }}
              required
            />
            {errors.executable && <span style={styles.errorText}>{errors.executable}</span>}
          </div>

          {/* Arguments */}
          <div style={styles.fieldGroup}>
            <label style={styles.label}>Command Arguments (space separated)</label>
            <input
              type="text"
              className="input-glass"
              style={{ fontFamily: "var(--font-mono)" }}
              placeholder="--port 6379 --protected-mode no"
              value={argumentsStr}
              onChange={(e) => setArgumentsStr(e.target.value)}
            />
          </div>

          {/* Working Directory */}
          <div style={styles.fieldGroup}>
            <label style={styles.label}>Working Directory</label>
            <input
              type="text"
              className="input-glass"
              style={{ fontFamily: "var(--font-mono)" }}
              placeholder="C:\"
              value={workingDirectory}
              maxLength={1024}
              onChange={(e) => setWorkingDirectory(e.target.value)}
            />
          </div>

          {/* Health Check */}
          <div style={styles.fieldGroup}>
            <label style={styles.label}>Health Check Endpoint / URL (Optional)</label>
            <input
              type="text"
              className="input-glass"
              style={{ fontFamily: "var(--font-mono)" }}
              placeholder="http://localhost:6379/health or /ping"
              value={healthCheck}
              onChange={(e) => setHealthCheck(e.target.value)}
            />
          </div>

          {/* Environment Variables */}
          <div style={styles.fieldGroup}>
            <label style={styles.label}>
              Environment Variables (<code>KEY=VALUE</code> per line)
            </label>
            <textarea
              className="input-glass"
              style={{
                fontFamily: "var(--font-mono)",
                minHeight: "68px",
                resize: "vertical",
              }}
              placeholder="NODE_ENV=development&#10;DEBUG=true"
              value={envVarsStr}
              onChange={(e) => setEnvVarsStr(e.target.value)}
            />
          </div>

          {/* Checkbox Toggles */}
          <div style={styles.togglesRow}>
            <label style={styles.toggleLabel}>
              <input
                type="checkbox"
                checked={autoStart}
                onChange={(e) => setAutoStart(e.target.checked)}
                style={styles.checkbox}
              />
              <span>Auto-start on application launch</span>
            </label>

            <label style={styles.toggleLabel}>
              <input
                type="checkbox"
                checked={autoRestart}
                onChange={(e) => setAutoRestart(e.target.checked)}
                style={styles.checkbox}
              />
              <span>Auto-restart if process terminates unexpectedly</span>
            </label>
          </div>

          {/* Modal Footer */}
          <div style={styles.footer}>
            <button
              type="button"
              onClick={onClose}
              className="btn-secondary"
              disabled={isSubmitting}
            >
              Cancel
            </button>
            <button
              type="submit"
              className="btn-primary"
              disabled={isSubmitting}
            >
              <Save size={14} strokeWidth={2} />
              <span>
                {isSubmitting
                  ? "Saving..."
                  : initialService
                  ? "Save Changes"
                  : "Create Service"}
              </span>
            </button>
          </div>
        </form>
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
    zIndex: 100,
  },
  modal: {
    width: "580px",
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
  },
  subtitle: {
    fontSize: "0.75rem",
    color: "var(--text-muted)",
  },
  errorBanner: {
    margin: "16px 24px 0",
    padding: "10px 14px",
    backgroundColor: "rgba(239, 68, 68, 0.1)",
    border: "1px solid rgba(239, 68, 68, 0.25)",
    borderRadius: "var(--radius-sm)",
    display: "flex",
    alignItems: "center",
    gap: "10px",
  },
  errorBannerText: {
    fontSize: "0.75rem",
    color: "#fca5a5",
  },
  form: {
    padding: "20px 24px 24px",
    display: "flex",
    flexDirection: "column",
    gap: "16px",
  },
  formGrid: {
    display: "grid",
    gridTemplateColumns: "1fr 140px",
    gap: "16px",
  },
  fieldGroup: {
    display: "flex",
    flexDirection: "column",
    gap: "6px",
  },
  label: {
    fontSize: "0.75rem",
    fontWeight: 500,
    color: "var(--text-secondary)",
  },
  errorText: {
    fontSize: "0.6875rem",
    color: "#f87171",
  },
  togglesRow: {
    display: "flex",
    flexDirection: "column",
    gap: "10px",
    padding: "12px 14px",
    backgroundColor: "rgba(255, 255, 255, 0.02)",
    border: "1px solid var(--border-subtle)",
    borderRadius: "var(--radius-sm)",
  },
  toggleLabel: {
    display: "flex",
    alignItems: "center",
    gap: "10px",
    fontSize: "0.8125rem",
    color: "var(--text-secondary)",
    cursor: "pointer",
  },
  checkbox: {
    accentColor: "#ffffff",
    cursor: "pointer",
  },
  footer: {
    display: "flex",
    alignItems: "center",
    justifyContent: "flex-end",
    gap: "10px",
    paddingTop: "12px",
    borderTop: "1px solid var(--border-subtle)",
  },
};
