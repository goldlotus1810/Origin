//! # log — EventLog append-only
//!
//! Ghi lại mọi sự kiện trong HomeOS.
//! Append-only — không xóa, không sửa (QT8).
//! Dùng để crash recovery: replay log → restore state.

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

// ─────────────────────────────────────────────────────────────────────────────
// LogEvent
// ─────────────────────────────────────────────────────────────────────────────

/// Một sự kiện trong EventLog.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum LogEvent {
    /// Node mới được tạo.
    NodeCreated {
        /// FNV-1a hash của chain
        chain_hash:  u64,
        /// Tầng
        layer:       u8,
        /// Offset trong file
        file_offset: u64,
        /// Timestamp (ns)
        timestamp:   i64,
    },
    /// Node được promote lên QR.
    NodePromotedQR {
        chain_hash: u64,
        timestamp:  i64,
    },
    /// Silk edge được tạo.
    EdgeCreated {
        from_hash:  u64,
        to_hash:    u64,
        edge_type:  u8,
        timestamp:  i64,
    },
    /// Hebbian weight cập nhật.
    WeightUpdated {
        from_hash:  u64,
        to_hash:    u64,
        weight:     f32,
        timestamp:  i64,
    },
    /// Alias được đăng ký.
    AliasRegistered {
        name:       String,
        chain_hash: u64,
        timestamp:  i64,
    },
    /// QR supersession.
    QRSuperseded {
        old_hash:  u64,
        new_hash:  u64,
        timestamp: i64,
    },
    /// Dream cycle chạy.
    DreamCycle {
        proposals: u32,
        timestamp: i64,
    },
    /// Lỗi hệ thống.
    SystemError {
        code:      u32,
        timestamp: i64,
    },
}

impl LogEvent {
    /// Timestamp của event.
    pub fn timestamp(&self) -> i64 {
        match self {
            Self::NodeCreated    { timestamp, .. } => *timestamp,
            Self::NodePromotedQR { timestamp, .. } => *timestamp,
            Self::EdgeCreated    { timestamp, .. } => *timestamp,
            Self::WeightUpdated  { timestamp, .. } => *timestamp,
            Self::AliasRegistered{ timestamp, .. } => *timestamp,
            Self::QRSuperseded   { timestamp, .. } => *timestamp,
            Self::DreamCycle     { timestamp, .. } => *timestamp,
            Self::SystemError    { timestamp, .. } => *timestamp,
        }
    }

    /// Type byte cho serialization.
    pub fn type_byte(&self) -> u8 {
        match self {
            Self::NodeCreated    { .. } => 0x01,
            Self::NodePromotedQR { .. } => 0x02,
            Self::EdgeCreated    { .. } => 0x03,
            Self::WeightUpdated  { .. } => 0x04,
            Self::AliasRegistered{ .. } => 0x05,
            Self::QRSuperseded   { .. } => 0x06,
            Self::DreamCycle     { .. } => 0x07,
            Self::SystemError    { .. } => 0xFF,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EventLog
// ─────────────────────────────────────────────────────────────────────────────

/// Log sự kiện — append-only (QT8).
///
/// In-memory trong session. Persist vào log.olang.
/// Crash recovery: replay log → rebuild Registry.
pub struct EventLog {
    events: Vec<LogEvent>,
    path:   String,
}

impl EventLog {
    /// Tạo EventLog mới.
    pub fn new(path: String) -> Self {
        Self { events: Vec::new(), path }
    }

    /// Append event — không bao giờ xóa hay sửa (QT8).
    pub fn append(&mut self, event: LogEvent) {
        self.events.push(event);
    }

    /// Số events.
    pub fn len(&self) -> usize { self.events.len() }

    /// Log có rỗng không.
    pub fn is_empty(&self) -> bool { self.events.is_empty() }

    /// Path của log file.
    pub fn path(&self) -> &str { &self.path }

    /// Tất cả events.
    pub fn events(&self) -> &[LogEvent] { &self.events }

    /// Events liên quan đến một chain_hash.
    pub fn events_for_hash(&self, hash: u64) -> Vec<&LogEvent> {
        self.events.iter().filter(|e| match e {
            LogEvent::NodeCreated    { chain_hash, .. } => *chain_hash == hash,
            LogEvent::NodePromotedQR { chain_hash, .. } => *chain_hash == hash,
            LogEvent::EdgeCreated    { from_hash, to_hash, .. } =>
                *from_hash == hash || *to_hash == hash,
            LogEvent::WeightUpdated  { from_hash, to_hash, .. } =>
                *from_hash == hash || *to_hash == hash,
            LogEvent::AliasRegistered{ chain_hash, .. } => *chain_hash == hash,
            LogEvent::QRSuperseded   { old_hash, new_hash, .. } =>
                *old_hash == hash || *new_hash == hash,
            _ => false,
        }).collect()
    }

    /// Events từ timestamp.
    pub fn events_since(&self, since_ns: i64) -> Vec<&LogEvent> {
        self.events.iter()
            .filter(|e| e.timestamp() >= since_ns)
            .collect()
    }

    /// NodeCreated events — dùng khi rebuild Registry từ log.
    pub fn node_created_events(&self) -> Vec<&LogEvent> {
        self.events.iter()
            .filter(|e| matches!(e, LogEvent::NodeCreated { .. }))
            .collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_log() -> EventLog {
        EventLog::new(String::from("test.log"))
    }

    #[test]
    fn log_empty() {
        let log = make_log();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn log_append() {
        let mut log = make_log();
        log.append(LogEvent::NodeCreated {
            chain_hash: 0xABCD, layer: 0,
            file_offset: 0, timestamp: 1000,
        });
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn log_append_only() {
        let mut log = make_log();
        // Append 5 events
        for i in 0..5u32 {
            log.append(LogEvent::SystemError { code: i, timestamp: i as i64 });
        }
        assert_eq!(log.len(), 5);
        // Không có cách xóa — chỉ append
    }

    #[test]
    fn log_events_for_hash() {
        let mut log = make_log();
        let hash = 0xFEED_BEEF_u64;

        log.append(LogEvent::NodeCreated {
            chain_hash: hash, layer: 2, file_offset: 0, timestamp: 1000,
        });
        log.append(LogEvent::NodeCreated {
            chain_hash: 0xDEAD, layer: 0, file_offset: 100, timestamp: 2000,
        });
        log.append(LogEvent::EdgeCreated {
            from_hash: hash, to_hash: 0xDEAD,
            edge_type: 0x01, timestamp: 3000,
        });

        let events = log.events_for_hash(hash);
        assert_eq!(events.len(), 2, "2 events liên quan đến hash");
    }

    #[test]
    fn log_events_since() {
        let mut log = make_log();
        log.append(LogEvent::SystemError { code: 1, timestamp: 1000 });
        log.append(LogEvent::SystemError { code: 2, timestamp: 2000 });
        log.append(LogEvent::SystemError { code: 3, timestamp: 3000 });

        let recent = log.events_since(2000);
        assert_eq!(recent.len(), 2, "Events từ t=2000 trở đi");
    }

    #[test]
    fn log_node_created_events() {
        let mut log = make_log();
        log.append(LogEvent::NodeCreated { chain_hash: 1, layer: 0, file_offset: 0, timestamp: 1 });
        log.append(LogEvent::EdgeCreated { from_hash: 1, to_hash: 2, edge_type: 0x01, timestamp: 2 });
        log.append(LogEvent::NodeCreated { chain_hash: 3, layer: 2, file_offset: 100, timestamp: 3 });

        let nc = log.node_created_events();
        assert_eq!(nc.len(), 2, "2 NodeCreated events");
    }

    #[test]
    fn log_qr_supersede_event() {
        let mut log = make_log();
        log.append(LogEvent::QRSuperseded {
            old_hash: 0xDEAD_0000_u64, new_hash: 0xBEEF_0000_u64, timestamp: 5000,
        });
        let events = log.events_for_hash(0xDEAD_0000_u64);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn log_dream_cycle() {
        let mut log = make_log();
        log.append(LogEvent::DreamCycle { proposals: 3, timestamp: 10000 });
        assert_eq!(log.len(), 1);
        assert!(matches!(log.events()[0], LogEvent::DreamCycle { proposals: 3, .. }));
    }
}
