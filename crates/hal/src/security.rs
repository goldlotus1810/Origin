//! # security — Process & Network Security Scanner
//!
//! Quét process đang chạy, kết nối mạng, phát hiện:
//!   - Process đáng ngờ (unknown origin, high resource)
//!   - Kết nối mạng trái phép (unauthorized external)
//!   - Port mở không cần thiết
//!
//! SecurityScanner → VulnerabilityReport → Chief → AAM
//! Khi phát hiện → CapabilityGate.check() → xin phép user để khắc phục

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::probe::{VulnerabilityReport, VulnerabilitySeverity, VulnerabilityCategory};

// ─────────────────────────────────────────────────────────────────────────────
// ProcessInfo — thông tin process
// ─────────────────────────────────────────────────────────────────────────────

/// Thông tin 1 process đang chạy.
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Tên process
    pub name: String,
    /// CPU usage (0.0 .. 1.0)
    pub cpu_usage: f32,
    /// Memory usage (bytes)
    pub memory_bytes: u64,
    /// Có phải system process không
    pub is_system: bool,
    /// Có known signature không (trusted)
    pub is_trusted: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// NetworkConnection — kết nối mạng
// ─────────────────────────────────────────────────────────────────────────────

/// Thông tin 1 kết nối mạng.
#[derive(Debug, Clone)]
pub struct NetworkConnection {
    /// Local port
    pub local_port: u16,
    /// Remote address (IP string)
    pub remote_addr: String,
    /// Remote port
    pub remote_port: u16,
    /// Protocol
    pub protocol: NetProtocol,
    /// Trạng thái
    pub status: ConnectionStatus,
    /// Process ID sở hữu kết nối
    pub pid: Option<u32>,
}

/// Network protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetProtocol {
    /// TCP
    Tcp,
    /// UDP
    Udp,
    /// Khác
    Other,
}

/// Trạng thái kết nối.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Đang kết nối
    Established,
    /// Đang chờ kết nối
    Listening,
    /// Đang đóng
    Closing,
    /// Khác
    Other,
}

// ─────────────────────────────────────────────────────────────────────────────
// ThreatLevel — mức độ đe dọa
// ─────────────────────────────────────────────────────────────────────────────

/// Mức độ đe dọa tổng hợp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreatLevel {
    /// An toàn
    Safe    = 0,
    /// Cần theo dõi
    Monitor = 1,
    /// Đáng ngờ — cần kiểm tra
    Suspect = 2,
    /// Nguy hiểm — cần hành động
    Danger  = 3,
}

// ─────────────────────────────────────────────────────────────────────────────
// SecurityScanner
// ─────────────────────────────────────────────────────────────────────────────

/// Security scanner — phân tích process và network connections.
///
/// Input: danh sách processes + connections (từ platform HAL).
/// Output: threat level + vulnerability reports.
pub struct SecurityScanner;

impl SecurityScanner {
    /// Quét processes → phát hiện đáng ngờ.
    pub fn scan_processes(processes: &[ProcessInfo]) -> (ThreatLevel, Vec<VulnerabilityReport>) {
        let mut reports = Vec::new();
        let mut max_threat = ThreatLevel::Safe;

        for proc in processes {
            // High CPU untrusted process
            if !proc.is_trusted && proc.cpu_usage > 0.80 {
                reports.push(VulnerabilityReport {
                    severity: VulnerabilitySeverity::High,
                    category: VulnerabilityCategory::SuspiciousProcess,
                    description: alloc::format!(
                        "Process '{}' (PID {}) — CPU {:.0}%, không trusted",
                        proc.name, proc.pid, proc.cpu_usage * 100.0
                    ),
                    recommendation: String::from("Kiểm tra và dừng nếu không cần thiết"),
                    component: alloc::format!("process:{}", proc.pid),
                });
                if max_threat < ThreatLevel::Suspect {
                    max_threat = ThreatLevel::Suspect;
                }
            }

            // High memory untrusted process
            if !proc.is_trusted && proc.memory_bytes > 500 * 1024 * 1024 {
                reports.push(VulnerabilityReport {
                    severity: VulnerabilitySeverity::Medium,
                    category: VulnerabilityCategory::SuspiciousProcess,
                    description: alloc::format!(
                        "Process '{}' (PID {}) — {}MB RAM, không trusted",
                        proc.name, proc.pid, proc.memory_bytes / (1024 * 1024)
                    ),
                    recommendation: String::from("Theo dõi mức sử dụng bộ nhớ"),
                    component: alloc::format!("process:{}", proc.pid),
                });
            }

            // Unknown non-system, non-trusted process
            if !proc.is_system && !proc.is_trusted && max_threat < ThreatLevel::Monitor {
                max_threat = ThreatLevel::Monitor;
            }
        }

        (max_threat, reports)
    }

    /// Quét connections → phát hiện kết nối trái phép.
    pub fn scan_connections(connections: &[NetworkConnection]) -> (ThreatLevel, Vec<VulnerabilityReport>) {
        let mut reports = Vec::new();
        let mut max_threat = ThreatLevel::Safe;

        for conn in connections {
            // Well-known dangerous ports
            if is_suspicious_port(conn.remote_port) && conn.status == ConnectionStatus::Established {
                reports.push(VulnerabilityReport {
                    severity: VulnerabilitySeverity::High,
                    category: VulnerabilityCategory::NetworkInsecure,
                    description: alloc::format!(
                        "Kết nối tới {}:{} (port đáng ngờ) — PID {:?}",
                        conn.remote_addr, conn.remote_port, conn.pid
                    ),
                    recommendation: String::from("Kiểm tra kết nối — có thể là malware C2"),
                    component: String::from("network"),
                });
                if max_threat < ThreatLevel::Danger {
                    max_threat = ThreatLevel::Danger;
                }
            }

            // Outbound to external on uncommon port
            if conn.status == ConnectionStatus::Established
                && conn.remote_port != 80
                && conn.remote_port != 443
                && conn.remote_port != 53
                && conn.remote_port != 22
                && conn.remote_port > 1024
                && !conn.remote_addr.starts_with("127.")
                && !conn.remote_addr.starts_with("192.168.")
                && !conn.remote_addr.starts_with("10.")
            {
                reports.push(VulnerabilityReport {
                    severity: VulnerabilitySeverity::Medium,
                    category: VulnerabilityCategory::NetworkInsecure,
                    description: alloc::format!(
                        "Kết nối external không thường: {}:{} — PID {:?}",
                        conn.remote_addr, conn.remote_port, conn.pid
                    ),
                    recommendation: String::from("Xác nhận kết nối này hợp lệ"),
                    component: String::from("network"),
                });
                if max_threat < ThreatLevel::Suspect {
                    max_threat = ThreatLevel::Suspect;
                }
            }

            // Listening on public port without PID
            if conn.status == ConnectionStatus::Listening && conn.pid.is_none() {
                reports.push(VulnerabilityReport {
                    severity: VulnerabilitySeverity::Medium,
                    category: VulnerabilityCategory::NetworkInsecure,
                    description: alloc::format!(
                        "Port {} listening — không rõ process",
                        conn.local_port,
                    ),
                    recommendation: String::from("Xác định process đang mở port này"),
                    component: String::from("network"),
                });
                if max_threat < ThreatLevel::Monitor {
                    max_threat = ThreatLevel::Monitor;
                }
            }
        }

        (max_threat, reports)
    }

    /// Quét toàn bộ (processes + connections).
    pub fn full_scan(
        processes: &[ProcessInfo],
        connections: &[NetworkConnection],
    ) -> (ThreatLevel, Vec<VulnerabilityReport>) {
        let (t1, mut r1) = Self::scan_processes(processes);
        let (t2, r2) = Self::scan_connections(connections);
        r1.extend(r2);
        let max_threat = if t1 > t2 { t1 } else { t2 };
        (max_threat, r1)
    }
}

/// Ports thường gắn với malware / C2 / backdoor.
fn is_suspicious_port(port: u16) -> bool {
    matches!(port,
        4444 |  // Metasploit default
        5555 |  // Android ADB (remote exploit)
        6666 |  // IRC botnet
        6667 |  // IRC botnet
        31337 | // Back Orifice
        12345 | // NetBus
        65535   // Uncommon max port
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn trusted_process() -> ProcessInfo {
        ProcessInfo {
            pid: 1,
            name: String::from("systemd"),
            cpu_usage: 0.01,
            memory_bytes: 10 * 1024 * 1024,
            is_system: true,
            is_trusted: true,
        }
    }

    fn suspicious_process() -> ProcessInfo {
        ProcessInfo {
            pid: 9999,
            name: String::from("unknown_miner"),
            cpu_usage: 0.95,
            memory_bytes: 800 * 1024 * 1024,
            is_system: false,
            is_trusted: false,
        }
    }

    fn normal_connection() -> NetworkConnection {
        NetworkConnection {
            local_port: 54321,
            remote_addr: String::from("93.184.216.34"),
            remote_port: 443,
            protocol: NetProtocol::Tcp,
            status: ConnectionStatus::Established,
            pid: Some(100),
        }
    }

    fn suspicious_connection() -> NetworkConnection {
        NetworkConnection {
            local_port: 54321,
            remote_addr: String::from("45.33.32.156"),
            remote_port: 4444,
            protocol: NetProtocol::Tcp,
            status: ConnectionStatus::Established,
            pid: Some(9999),
        }
    }

    #[test]
    fn scan_trusted_processes_safe() {
        let procs = [trusted_process()];
        let (threat, reports) = SecurityScanner::scan_processes(&procs);
        assert_eq!(threat, ThreatLevel::Safe);
        assert!(reports.is_empty(), "Trusted process → no reports");
    }

    #[test]
    fn scan_suspicious_process_detected() {
        let procs = [trusted_process(), suspicious_process()];
        let (threat, reports) = SecurityScanner::scan_processes(&procs);
        assert!(threat >= ThreatLevel::Suspect, "High CPU untrusted → Suspect");
        assert!(!reports.is_empty(), "Phải có report");
        assert!(reports[0].description.contains("unknown_miner"));
    }

    #[test]
    fn scan_normal_connection_safe() {
        let conns = [normal_connection()];
        let (threat, reports) = SecurityScanner::scan_connections(&conns);
        assert_eq!(threat, ThreatLevel::Safe);
        assert!(reports.is_empty(), "HTTPS connection → safe");
    }

    #[test]
    fn scan_suspicious_connection_danger() {
        let conns = [suspicious_connection()];
        let (threat, reports) = SecurityScanner::scan_connections(&conns);
        assert_eq!(threat, ThreatLevel::Danger, "Port 4444 → Danger");
        assert!(!reports.is_empty());
        assert!(reports[0].description.contains("4444"));
    }

    #[test]
    fn scan_listening_no_pid() {
        let conn = NetworkConnection {
            local_port: 8080,
            remote_addr: String::new(),
            remote_port: 0,
            protocol: NetProtocol::Tcp,
            status: ConnectionStatus::Listening,
            pid: None,
        };
        let (threat, reports) = SecurityScanner::scan_connections(&[conn]);
        assert!(threat >= ThreatLevel::Monitor);
        assert!(!reports.is_empty());
    }

    #[test]
    fn scan_external_uncommon_port() {
        let conn = NetworkConnection {
            local_port: 54321,
            remote_addr: String::from("203.0.113.50"),
            remote_port: 8888,
            protocol: NetProtocol::Tcp,
            status: ConnectionStatus::Established,
            pid: Some(500),
        };
        let (threat, reports) = SecurityScanner::scan_connections(&[conn]);
        assert!(threat >= ThreatLevel::Suspect);
        assert!(!reports.is_empty());
    }

    #[test]
    fn scan_local_network_not_flagged() {
        let conn = NetworkConnection {
            local_port: 54321,
            remote_addr: String::from("192.168.1.100"),
            remote_port: 8888,
            protocol: NetProtocol::Tcp,
            status: ConnectionStatus::Established,
            pid: Some(500),
        };
        let (threat, reports) = SecurityScanner::scan_connections(&[conn]);
        // Local network → no external warning
        assert_eq!(threat, ThreatLevel::Safe);
        assert!(reports.is_empty(), "Local network → safe");
    }

    #[test]
    fn full_scan_combines() {
        let procs = [suspicious_process()];
        let conns = [suspicious_connection()];
        let (threat, reports) = SecurityScanner::full_scan(&procs, &conns);
        assert!(threat >= ThreatLevel::Danger);
        assert!(reports.len() >= 2, "Process + connection reports");
    }

    #[test]
    fn threat_level_ordering() {
        assert!(ThreatLevel::Danger > ThreatLevel::Suspect);
        assert!(ThreatLevel::Suspect > ThreatLevel::Monitor);
        assert!(ThreatLevel::Monitor > ThreatLevel::Safe);
    }

    #[test]
    fn suspicious_ports() {
        assert!(is_suspicious_port(4444));
        assert!(is_suspicious_port(31337));
        assert!(!is_suspicious_port(80));
        assert!(!is_suspicious_port(443));
        assert!(!is_suspicious_port(22));
    }
}
