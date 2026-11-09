//! Tauri IPC commands for optional email integration checks.
//!
//! The current implementation is intentionally lightweight and offline-friendly:
//! it validates reachability to configured IMAP/SMTP endpoints via TCP without
//! introducing always-on background sync or heavy protocol stacks.

use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/// Supported email protocols for connection checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmailProtocol {
    Smtp,
    Imap,
}

/// Request payload for testing an email server endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConnectionTestRequest {
    pub protocol: EmailProtocol,
    pub host: String,
    pub port: u16,
    pub timeout_ms: Option<u64>,
}

/// Response payload for endpoint test results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConnectionTestResult {
    pub protocol: EmailProtocol,
    pub host: String,
    pub port: u16,
    pub success: bool,
    pub latency_ms: u128,
    pub details: String,
    pub banner: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Command
// ─────────────────────────────────────────────────────────────────────────────

/// Tests TCP reachability for an SMTP/IMAP endpoint.
///
/// This is intentionally a lightweight integration check:
/// - verifies DNS resolution + TCP connect
/// - attempts a best-effort protocol probe on plaintext ports
/// - does not perform credential login or full TLS negotiation
#[tauri::command(rename_all = "snake_case")]
pub async fn test_email_server_connection(
    request: EmailConnectionTestRequest,
) -> Result<EmailConnectionTestResult, String> {
    let host = request.host.trim().to_string();
    if host.is_empty() {
        return Err("Host is required".to_string());
    }
    if request.port == 0 {
        return Err("Port must be greater than 0".to_string());
    }

    let timeout_ms = request.timeout_ms.unwrap_or(2_500).clamp(500, 10_000);
    let protocol = request.protocol.clone();
    let host_for_task = host.clone();
    let port = request.port;

    let result = tokio::task::spawn_blocking(move || {
        let started = Instant::now();
        let addr_string = format!("{}:{}", host_for_task, port);

        let addrs: Vec<SocketAddr> = match addr_string.to_socket_addrs() {
            Ok(it) => it.collect(),
            Err(err) => {
                return EmailConnectionTestResult {
                    protocol,
                    host: host_for_task,
                    port,
                    success: false,
                    latency_ms: started.elapsed().as_millis(),
                    details: format!("DNS resolution failed: {}", err),
                    banner: None,
                };
            }
        };

        let addr = match addrs.into_iter().find(|a| !is_disallowed_address(a.ip())) {
            Some(a) => a,
            None => {
                return EmailConnectionTestResult {
                    protocol,
                    host: host_for_task,
                    port,
                    success: false,
                    latency_ms: started.elapsed().as_millis(),
                    details: "Host resolves to a disallowed (private/loopback/metadata) address"
                        .to_string(),
                    banner: None,
                };
            }
        };

        let timeout = Duration::from_millis(timeout_ms);
        let mut stream = match TcpStream::connect_timeout(&addr, timeout) {
            Ok(s) => s,
            Err(err) => {
                return EmailConnectionTestResult {
                    protocol,
                    host: host_for_task,
                    port,
                    success: false,
                    latency_ms: started.elapsed().as_millis(),
                    details: format!("Connection failed: {}", err),
                    banner: None,
                };
            }
        };

        let _ = stream.set_read_timeout(Some(Duration::from_millis(700)));
        let _ = stream.set_write_timeout(Some(Duration::from_millis(700)));

        let banner = probe_banner(&mut stream, &protocol, port);
        let details = match protocol {
            EmailProtocol::Smtp => {
                if port == 465 {
                    "SMTP endpoint reachable (implicit TLS port; TCP probe only)".to_string()
                } else {
                    "SMTP endpoint reachable".to_string()
                }
            }
            EmailProtocol::Imap => {
                if port == 993 {
                    "IMAP endpoint reachable (implicit TLS port; TCP probe only)".to_string()
                } else {
                    "IMAP endpoint reachable".to_string()
                }
            }
        };

        EmailConnectionTestResult {
            protocol,
            host: host_for_task,
            port,
            success: true,
            latency_ms: started.elapsed().as_millis(),
            details,
            banner,
        }
    })
    .await
    .map_err(|err| format!("Email test task failed: {}", err))?;

    Ok(result)
}

fn probe_banner(stream: &mut TcpStream, protocol: &EmailProtocol, port: u16) -> Option<String> {
    if matches!(protocol, EmailProtocol::Smtp) && port != 465 {
        let _ = stream.write_all(b"EHLO 900crm.local\r\nQUIT\r\n");
    }
    if matches!(protocol, EmailProtocol::Imap) && port != 993 {
        let _ = stream.write_all(b"a1 CAPABILITY\r\na2 LOGOUT\r\n");
    }

    let mut buf = [0u8; 256];
    match stream.read(&mut buf) {
        Ok(n) if n > 0 => {
            let raw = String::from_utf8_lossy(&buf[..n]).trim().to_string();
            if raw.is_empty() {
                None
            } else {
                Some(raw.chars().take(120).collect())
            }
        }
        _ => None,
    }
}

/// Returns `true` for addresses a connection test must not contact: loopback,
/// unspecified, private, link-local (incl. cloud metadata `169.254.169.254`),
/// CGNAT, broadcast/multicast, IPv6 unique-local ranges, and IPv4-mapped IPv6
/// forms of those ranges (`::ffff:127.0.0.1`, `::ffff:169.254.169.254`).
fn is_disallowed_address(addr: IpAddr) -> bool {
    match canonicalize_ip(addr) {
        IpAddr::V4(v4) => is_disallowed_ipv4(v4),
        IpAddr::V6(v6) => is_disallowed_ipv6(v6),
    }
}

fn canonicalize_ip(addr: IpAddr) -> IpAddr {
    match addr {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(addr),
        other => other,
    }
}

fn is_disallowed_ipv4(addr: std::net::Ipv4Addr) -> bool {
    addr.is_loopback()
        || addr.is_unspecified()
        || addr.is_multicast()
        || addr.is_private()
        || addr.is_link_local()
        || addr.is_broadcast()
        || is_carrier_grade_nat(addr)
}

fn is_disallowed_ipv6(addr: std::net::Ipv6Addr) -> bool {
    if addr.is_loopback() || addr.is_unspecified() || addr.is_multicast() {
        return true;
    }
    if let Some(v4) = addr.to_ipv4_mapped() {
        return is_disallowed_ipv4(v4);
    }
    let octets = addr.octets();
    // Link-local fe80::/10
    if octets[0] == 0xFE && (octets[1] & 0xC0) == 0x80 {
        return true;
    }
    // Unique local fc00::/7
    (octets[0] & 0xFE) == 0xFC
}

fn is_carrier_grade_nat(addr: std::net::Ipv4Addr) -> bool {
    let octets = addr.octets();
    octets[0] == 100 && (octets[1] & 0xC0) == 64
}

#[cfg(test)]
mod tests {
    use super::is_disallowed_address;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn rejects_loopback_and_private_v4() {
        assert!(is_disallowed_address(IpAddr::V4(Ipv4Addr::LOCALHOST)));
        assert!(is_disallowed_address(IpAddr::V4(Ipv4Addr::new(
            10, 0, 0, 1
        ))));
        assert!(is_disallowed_address(IpAddr::V4(Ipv4Addr::new(
            192, 168, 1, 1
        ))));
        assert!(is_disallowed_address(IpAddr::V4(Ipv4Addr::new(
            172, 16, 0, 1
        ))));
    }

    #[test]
    fn rejects_link_local_metadata_and_cgnat() {
        assert!(is_disallowed_address(IpAddr::V4(Ipv4Addr::new(
            169, 254, 169, 254
        ))));
        assert!(is_disallowed_address(IpAddr::V4(Ipv4Addr::new(
            100, 64, 0, 1
        ))));
    }

    #[test]
    fn rejects_ipv4_mapped_loopback_and_metadata() {
        assert!(is_disallowed_address(IpAddr::V6(Ipv6Addr::new(
            0, 0, 0, 0, 0, 0xffff, 0x7f00, 1
        ))));
        assert!(is_disallowed_address(IpAddr::V6(
            Ipv4Addr::new(127, 0, 0, 1).to_ipv6_mapped()
        )));
        assert!(is_disallowed_address(IpAddr::V6(
            Ipv4Addr::new(169, 254, 169, 254).to_ipv6_mapped()
        )));
        assert!(is_disallowed_address(IpAddr::V6(
            Ipv4Addr::new(10, 1, 2, 3).to_ipv6_mapped()
        )));
    }

    #[test]
    fn allows_public_v4_and_v6() {
        assert!(!is_disallowed_address(IpAddr::V4(Ipv4Addr::new(
            8, 8, 8, 8
        ))));
        assert!(!is_disallowed_address(IpAddr::V6(Ipv6Addr::new(
            0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888
        ))));
    }
}
