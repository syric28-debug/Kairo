use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6, TcpStream, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

pub const DEFAULT_PORT_CHECK_TIMEOUT_MS: u64 = 150;

/// Standalone port checker for local service endpoints.
/// Strictly limited to local loopback (127.0.0.1 and [::1]).
///
/// Principles:
/// - Zero external network scanning or discovery
/// - Non-blocking to caller (uses short connect_timeout)
/// - Clean socket termination (drops TcpStream immediately upon connection)
/// - Separate from ProcessManager lifecycle responsibilities
pub struct PortChecker;

impl PortChecker {
    /// Checks whether the specified port is actively listening on local loopback.
    /// Uses the default timeout (150ms).
    pub fn check_local_port(port: u16) -> bool {
        Self::is_port_listening(port, Duration::from_millis(DEFAULT_PORT_CHECK_TIMEOUT_MS))
    }

    /// Checks whether the specified port is actively listening on local loopback
    /// using a custom timeout duration.
    ///
    /// Probe Order:
    /// 1. IPv4 loopback (127.0.0.1:port)
    /// 2. IPv6 loopback ([::1]:port) if IPv4 is not listening or fails
    pub fn is_port_listening(port: u16, timeout: Duration) -> bool {
        if port == 0 {
            return false;
        }

        // 1. Probe IPv4 loopback (127.0.0.1)
        let ipv4_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port));
        if Self::probe_socket(&ipv4_addr, timeout) {
            return true;
        }

        // 2. Probe IPv6 loopback (::1) as fallback
        let ipv6_addr = SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), port, 0, 0));
        if Self::probe_socket(&ipv6_addr, timeout) {
            return true;
        }

        false
    }

    /// Attempts a non-blocking TCP handshake to a target local socket address.
    /// Returns true if a connection was successfully established.
    fn probe_socket(addr: &SocketAddr, timeout: Duration) -> bool {
        match TcpStream::connect_timeout(addr, timeout) {
            Ok(stream) => {
                // Connection succeeded. Drop stream immediately to close connection.
                drop(stream);
                true
            }
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn test_zero_port_returns_false() {
        assert!(!PortChecker::check_local_port(0));
    }

    #[test]
    fn test_active_listener_detected() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind ephemeral port");
        let port = listener.local_addr().unwrap().port();

        assert!(PortChecker::check_local_port(port));
    }

    #[test]
    fn test_unbound_port_returns_false() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind ephemeral port");
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        assert!(!PortChecker::check_local_port(port));
    }
}
