//! Tauri IPC commands for optional email integration checks.
//!
//! The current implementation is intentionally lightweight and offline-friendly:
//! it validates reachability to configured IMAP/SMTP endpoints via TCP without
//! introducing always-on background sync or heavy protocol stacks.

use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const MAX_TIMEOUT_MS: u64 = 5_000;
const MIN_TIMEOUT_MS: u64 = 500;
const PROBE_INTERVAL: Duration = Duration::from_secs(60);

static LAST_EMAIL_PROBE: Mutex<Option<Instant>> = Mutex::new(None);

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
    let protocol_name = protocol_name(&request.protocol);
    log::info!(
        "Email connectivity probe requested (protocol={}, port={})",
        protocol_name,
        request.port
    );
    if host.is_empty() {
        return Err("Host is required".to_string());
    }
    if host.len() > 253 || host.chars().any(char::is_control) {
        return Err("Host is invalid or too long".to_string());
    }
    if !is_allowed_port(&request.protocol, request.port) {
        return Err("Port is not allowed for this email protocol".to_string());
    }
    reserve_email_probe_slot()?;

    let timeout = Duration::from_millis(
        request
            .timeout_ms
            .unwrap_or(2_500)
            .clamp(MIN_TIMEOUT_MS, MAX_TIMEOUT_MS),
    );
    let protocol = request.protocol;
    let port = request.port;

    let probe = async move {
        let started = Instant::now();
        let addr_string = format!("{}:{}", host, port);

        let addrs = match tokio::net::lookup_host(&addr_string).await {
            Ok(addrs) => addrs.collect::<Vec<_>>(),
            Err(err) => {
                return failure_result(
                    protocol,
                    host,
                    port,
                    started,
                    format!("DNS resolution failed: {}", err),
                );
            }
        };

        let addr = match addrs.into_iter().find(|addr| !is_disallowed_address(addr.ip())) {
            Some(addr) => addr,
            None => {
                return failure_result(
                    protocol,
                    host,
                    port,
                    started,
                    "Host resolves to a disallowed (private/loopback/metadata) address".to_string(),
                );
            }
        };

        let mut stream = match tokio::net::TcpStream::connect(addr).await {
            Ok(stream) => stream,
            Err(err) => {
                return failure_result(
                    protocol,
                    host,
                    port,
                    started,
                    format!("Connection failed: {}", err),
                );
            }
        };

        let banner = probe_banner(&mut stream, &protocol, port).await;
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
            host,
            port,
            success: true,
            latency_ms: started.elapsed().as_millis(),
            details,
            banner,
        }
    };
    let result = match tokio::time::timeout(timeout, probe).await {
        Ok(result) => result,
        Err(_) => {
            log::warn!(
                "Email connectivity probe timed out (protocol={}, port={})",
                protocol_name,
                request.port
            );
            return Err("Email connectivity test timed out (maximum 5 seconds)".to_string());
        }
    };
    log::info!(
        "Email connectivity probe completed (protocol={}, port={}, success={})",
        protocol_name,
        request.port,
        result.success
    );

    Ok(result)
}

fn failure_result(
    protocol: EmailProtocol,
    host: String,
    port: u16,
    started: Instant,
    details: String,
) -> EmailConnectionTestResult {
    EmailConnectionTestResult {
        protocol,
        host,
        port,
        success: false,
        latency_ms: started.elapsed().as_millis(),
        details,
        banner: None,
    }
}

fn protocol_name(protocol: &EmailProtocol) -> &'static str {
    match protocol {
        EmailProtocol::Smtp => "smtp",
        EmailProtocol::Imap => "imap",
    }
}

fn is_allowed_port(protocol: &EmailProtocol, port: u16) -> bool {
    match protocol {
        EmailProtocol::Smtp => matches!(port, 25 | 465 | 587),
        EmailProtocol::Imap => matches!(port, 143 | 993),
    }
}

fn reserve_email_probe_slot() -> Result<(), String> {
    let now = Instant::now();
    let mut last_probe = LAST_EMAIL_PROBE
        .lock()
        .map_err(|_| "Email connection test is unavailable".to_string())?;
    if !reserve_probe_slot(&mut last_probe, now) {
        return Err("Wait one minute before testing another email server".to_string());
    }
    Ok(())
}

fn reserve_probe_slot(last_probe: &mut Option<Instant>, now: Instant) -> bool {
    if last_probe.is_some_and(|last| now.duration_since(last) < PROBE_INTERVAL) {
        return false;
    }
    *last_probe = Some(now);
    true
}

async fn probe_banner(
    stream: &mut tokio::net::TcpStream,
    protocol: &EmailProtocol,
    port: u16,
) -> Option<String> {
    if matches!(protocol, EmailProtocol::Smtp) && port != 465 {
        let _ = tokio::time::timeout(
            Duration::from_millis(500),
            stream.write_all(b"EHLO 900crm.local\r\nQUIT\r\n"),
        )
        .await;
    }
    if matches!(protocol, EmailProtocol::Imap) && port != 993 {
        let _ = tokio::time::timeout(
            Duration::from_millis(500),
            stream.write_all(b"a1 CAPABILITY\r\na2 LOGOUT\r\n"),
        )
        .await;
    }

    let mut buf = [0u8; 256];
    match tokio::time::timeout(Duration::from_millis(500), stream.read(&mut buf)).await {
        Ok(Ok(n)) if n > 0 => {
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
    use std::time::{Duration, Instant};

    use super::{
        is_allowed_port, is_disallowed_address, reserve_probe_slot, EmailProtocol, PROBE_INTERVAL,
    };
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

    #[test]
    fn allows_only_email_service_ports_for_each_protocol() {
        assert!(is_allowed_port(&EmailProtocol::Smtp, 25));
        assert!(is_allowed_port(&EmailProtocol::Smtp, 465));
        assert!(is_allowed_port(&EmailProtocol::Smtp, 587));
        assert!(!is_allowed_port(&EmailProtocol::Smtp, 143));
        assert!(!is_allowed_port(&EmailProtocol::Smtp, 22));

        assert!(is_allowed_port(&EmailProtocol::Imap, 143));
        assert!(is_allowed_port(&EmailProtocol::Imap, 993));
        assert!(!is_allowed_port(&EmailProtocol::Imap, 587));
        assert!(!is_allowed_port(&EmailProtocol::Imap, 22));
    }

    #[test]
    fn limits_probe_frequency_to_one_per_minute() {
        let start = Instant::now();
        let mut last_probe = None;
        assert!(reserve_probe_slot(&mut last_probe, start));
        assert!(!reserve_probe_slot(
            &mut last_probe,
            start + Duration::from_secs(59)
        ));
        assert!(reserve_probe_slot(
            &mut last_probe,
            start + PROBE_INTERVAL
        ));
    }
}
