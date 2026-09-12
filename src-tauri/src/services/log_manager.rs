use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};

use crate::models::LogEntry;

/// Default maximum number of in-memory log entries preserved per service.
pub const DEFAULT_MAX_LOG_ENTRIES: usize = 1000;

/// Event name emitted to frontend subscribers when a new log entry is captured.
pub const SERVICE_LOG_APPENDED_EVENT: &str = "service-log-appended";

/// Internal bounded in-memory buffer state per service.
struct LogManagerState {
    buffers: HashMap<String, VecDeque<LogEntry>>,
    max_entries: usize,
}

/// Central in-memory log management service for KAIRO.
///
/// Principles:
/// - In-memory only (no persistent log files in Phase 9)
/// - Strict per-service isolation: Service A logs never mix with Service B
/// - Bounded capacity (FIFO eviction of oldest entries when limit reached)
/// - Asynchronous live event dispatching via event sink
/// - Safe concurrent access via `RwLock`
#[derive(Clone)]
pub struct LogManager {
    state: Arc<RwLock<LogManagerState>>,
    event_sink: Arc<RwLock<Option<Arc<dyn Fn(&LogEntry) + Send + Sync>>>>,
}

impl Default for LogManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LogManager {
    /// Creates a `LogManager` instance with standard bounded capacity (1,000 entries per service).
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_MAX_LOG_ENTRIES)
    }

    /// Creates a `LogManager` instance with a customized per-service capacity.
    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            state: Arc::new(RwLock::new(LogManagerState {
                buffers: HashMap::new(),
                max_entries,
            })),
            event_sink: Arc::new(RwLock::new(None)),
        }
    }

    /// Registers a custom event listener hook (useful for headless testing).
    pub fn set_event_sink<F>(&self, sink: F)
    where
        F: Fn(&LogEntry) + Send + Sync + 'static,
    {
        if let Ok(mut sink_guard) = self.event_sink.write() {
            *sink_guard = Some(Arc::new(sink));
        }
    }

    /// Appends a new log entry to the target service's bounded in-memory buffer,
    /// evicting the oldest entry if capacity is reached, and broadcasts the event.
    pub fn append(&self, entry: LogEntry) {
        // 1. In-memory bounded buffer storage
        if let Ok(mut state) = self.state.write() {
            let max = state.max_entries;
            let buffer = state.buffers.entry(entry.service_id.clone()).or_default();
            while buffer.len() >= max && max > 0 {
                buffer.pop_front();
            }
            buffer.push_back(entry.clone());
        }

        // 2. Dispatch to registered event sink (Tauri app handler or test callback)
        if let Ok(sink_guard) = self.event_sink.read() {
            if let Some(ref sink) = *sink_guard {
                sink(&entry);
            }
        }
    }

    /// Retrieves all buffered log entries for a given service in chronological order.
    pub fn get_logs(&self, service_id: &str) -> Vec<LogEntry> {
        if let Ok(state) = self.state.read() {
            if let Some(buffer) = state.buffers.get(service_id) {
                return buffer.iter().cloned().collect();
            }
        }
        Vec::new()
    }

    /// Clears the in-memory log buffer for a single service.
    pub fn clear_logs(&self, service_id: &str) {
        if let Ok(mut state) = self.state.write() {
            if let Some(buffer) = state.buffers.get_mut(service_id) {
                buffer.clear();
            }
        }
    }

    /// Clears the in-memory log buffer across all tracked services.
    pub fn clear_all_logs(&self) {
        if let Ok(mut state) = self.state.write() {
            state.buffers.clear();
        }
    }
}
