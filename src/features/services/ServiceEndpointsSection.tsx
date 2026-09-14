import React, { useState, useMemo } from 'react';
import {
  Copy,
  Check,
  ExternalLink,
} from 'lucide-react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { ServiceConfig } from '../../types';
import { buildServiceEndpoints } from '../../utils/serviceUrl';

interface ServiceEndpointsSectionProps {
  config: ServiceConfig;
  status: string;
}

function StatusBadge({ text, color }: { text: string; color: string }) {
  return (
    <span style={{
      fontFamily: 'var(--font-mono)',
      fontSize: '0.6875rem',
      padding: '2px 6px',
      borderRadius: 'var(--radius-xs)',
      marginLeft: '8px',
      fontWeight: 500,
      backgroundColor: color === '#22c55e' ? 'rgba(34,197,94,0.12)'
        : color === '#fbbf24' ? 'rgba(251,191,36,0.12)'
        : 'rgba(156,163,175,0.12)',
      color,
      border: color === '#22c55e' ? '1px solid rgba(34,197,94,0.25)'
        : color === '#fbbf24' ? '1px solid rgba(251,191,36,0.25)'
        : '1px solid var(--border-subtle)',
    }}>
      {text}
    </span>
  );
}

interface EndpointRowProps {
  label: string;
  url: string;
  onCopy: () => void;
  onOpen?: () => void;
  showOpen?: boolean;
  copySuccess: boolean;
  openSuccess?: boolean;
}

function EndpointRow({ label, url, onCopy, onOpen, showOpen, copySuccess, openSuccess }: EndpointRowProps) {
  return (
    <div style={styles.endpointRow}>
      <div style={{ flex: 1, minWidth: 0 }}>
        <span style={styles.endpointLabel}>{label}</span>
        <span style={styles.endpointUrl}>
          {url}
        </span>
      </div>
      <div style={styles.endpointActions}>
        {showOpen && onOpen && (
          <button
            style={openSuccess ? styles.endpointButtonSuccess : styles.endpointButtonOpen}
            onClick={onOpen}
            title='Open in browser'
            disabled={openSuccess}
          >
            {openSuccess ? <Check size={11} strokeWidth={2.4} /> : <ExternalLink size={11} strokeWidth={1.6} />}
            <span>{openSuccess ? 'Opened' : 'Open'}</span>
          </button>
        )}
        <button
          style={copySuccess ? styles.endpointButtonSuccess : styles.endpointButton}
          onClick={onCopy}
          title='Copy to clipboard'
          disabled={copySuccess}
        >
          {copySuccess ? <Check size={11} strokeWidth={2.4} /> : <Copy size={11} strokeWidth={1.8} />}
          <span>{copySuccess ? 'Copied' : 'Copy'}</span>
        </button>
      </div>
    </div>
  );
}

export const ServiceEndpointsSection: React.FC<ServiceEndpointsSectionProps> = ({ config, status }) => {
  const endpoints = useMemo(() => buildServiceEndpoints(config), [config]);
  if (!endpoints) return null;

  const isReady = status === 'ready';
  const isRunning = status === 'running' || status === 'starting';
  const statusText = isReady ? 'Ready' : isRunning ? 'Waiting' : 'Configured';
  const statusColor = isReady ? '#22c55e' : isRunning ? '#fbbf24' : 'var(--text-muted)';

  const [copyBaseSuccess, setCopyBaseSuccess] = useState(false);
  const [openBaseSuccess, setOpenBaseSuccess] = useState(false);
  const [copyApiSuccess, setCopyApiSuccess] = useState(false);
  const [copyDirectSuccess, setCopyDirectSuccess] = useState(false);
  const [openDirectSuccess, setOpenDirectSuccess] = useState(false);
  const [copyHealthSuccess, setCopyHealthSuccess] = useState(false);
  const [openHealthSuccess, setOpenHealthSuccess] = useState(false);

    const copyWithFeedback = (setter: React.Dispatch<React.SetStateAction<boolean>>, text: string | undefined) => {
    if (!text) return;
    const doCopy = async () => {
      try {
        if (navigator.clipboard?.writeText) {
          await navigator.clipboard.writeText(text);
          setter(true);
          setTimeout(() => setter(false), 1400);
          return;
        }
      } catch { /* fall through */ }
      try {
        const ta = document.createElement('textarea');
        ta.value = text;
        ta.style.position = 'fixed';
        ta.style.left = '-9999px';
        document.body.appendChild(ta);
        ta.select();
        if (document.execCommand('copy')) {
          setter(true);
          setTimeout(() => setter(false), 1400);
        }
        document.body.removeChild(ta);
      } catch { /* silently fail */ }
    };
    doCopy();
  };

    const openWithFeedback = (setter: React.Dispatch<React.SetStateAction<boolean>>, url: string | undefined) => {
    if (!url) return;
    openUrl(url).then(() => {
      setter(true);
      setTimeout(() => setter(false), 1400);
    }).catch(() => { /* silently fail */ });
  };

  return (
    <div style={styles.endpointsSection}>
      <div style={styles.endpointsHeader}>
        <span style={styles.endpointsLabel}>Endpoints</span>
        <StatusBadge text={statusText} color={statusColor} />
      </div>

      <EndpointRow
        label='Base URL'
        url={endpoints.baseUrl}
        onCopy={() => copyWithFeedback(setCopyBaseSuccess, endpoints.baseUrl)}
        onOpen={() => openWithFeedback(setOpenBaseSuccess, endpoints.baseUrl)}
        showOpen
        copySuccess={copyBaseSuccess}
        openSuccess={openBaseSuccess}
      />

      {endpoints.apiBaseUrl && (
        <EndpointRow
          label='API Base URL'
          url={endpoints.apiBaseUrl}
          onCopy={() => copyWithFeedback(setCopyApiSuccess, endpoints.apiBaseUrl)}
          copySuccess={copyApiSuccess}
        />
      )}

      {endpoints.directUrl && (
        <EndpointRow
          label='Direct Link'
          url={endpoints.directUrl}
          onCopy={() => copyWithFeedback(setCopyDirectSuccess, endpoints.directUrl)}
          onOpen={() => openWithFeedback(setOpenDirectSuccess, endpoints.directUrl)}
          showOpen
          copySuccess={copyDirectSuccess}
          openSuccess={openDirectSuccess}
        />
      )}

      {endpoints.healthUrl && (
        <EndpointRow
          label='Health URL'
          url={endpoints.healthUrl}
          onCopy={() => copyWithFeedback(setCopyHealthSuccess, endpoints.healthUrl)}
          onOpen={() => openWithFeedback(setOpenHealthSuccess, endpoints.healthUrl)}
          showOpen
          copySuccess={copyHealthSuccess}
          openSuccess={openHealthSuccess}
        />
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  endpointsSection: {
    border: '1px solid var(--border-subtle)',
    borderRadius: 'var(--radius-sm)',
    backgroundColor: 'rgba(255, 255, 255, 0.02)',
    padding: '12px 16px',
    marginTop: '4px',
  },
  endpointsHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    marginBottom: '10px',
  },
  endpointsLabel: {
    fontFamily: 'var(--font-mono)',
    fontSize: '0.6875rem',
    fontWeight: 600,
    color: 'var(--text-muted)',
    letterSpacing: '0.08em',
    textTransform: 'uppercase',
  },
  endpointRow: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    gap: '12px',
    padding: '8px 0',
  },
  endpointLabel: {
    fontSize: '0.75rem',
    color: 'var(--text-muted)',
    display: 'block',
    marginBottom: '2px',
  },
  endpointUrl: {
    fontFamily: 'var(--font-mono)',
    fontSize: '0.8125rem',
    color: 'var(--text-primary)',
  },
  endpointActions: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    flexShrink: 0,
  },
  endpointButton: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '4px',
    fontSize: '0.75rem',
    color: 'var(--text-secondary)',
    background: 'rgba(255, 255, 255, 0.05)',
    border: '1px solid var(--border-subtle)',
    borderRadius: 'var(--radius-sm)',
    padding: '4px 8px',
    cursor: 'pointer',
  },
  endpointButtonSuccess: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '4px',
    fontSize: '0.75rem',
    color: '#22c55e',
    background: 'rgba(34, 197, 94, 0.08)',
    border: '1px solid rgba(34, 197, 94, 0.25)',
    borderRadius: 'var(--radius-sm)',
    padding: '4px 8px',
    cursor: 'default',
  },
  endpointButtonOpen: {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '4px',
    fontSize: '0.75rem',
    color: 'var(--text-secondary)',
    background: 'transparent',
    border: '1px solid var(--border-subtle)',
    borderRadius: 'var(--radius-sm)',
    padding: '4px 8px',
    cursor: 'pointer',
  },
};
