//! # error — Error handling strategy cho HomeOS
//!
//! Chiến lược error handling:
//!   1. Disk full    → flush pending_writes, warn, continue in-memory
//!   2. Corrupt file → parse_recoverable, backup, continue
//!   3. Network fail → retry with exponential backoff (Fib-based)
//!   4. Encode fail  → return ProcessResult::Empty, log
//!
//! Không panic — luôn degrade gracefully.

extern crate alloc;
use alloc::string::String;
use alloc::format;

/// Lỗi runtime — degrade gracefully, không panic.
#[derive(Debug, Clone)]
pub enum HomeError {
    /// Disk full hoặc write failure.
    DiskFull {
        pending_bytes: usize,
        message: String,
    },
    /// File bị corrupt — recoverable.
    CorruptFile {
        recovered_records: usize,
        lost_bytes: usize,
    },
    /// Network failure — có thể retry.
    NetworkFailure {
        attempt: u8,
        max_attempts: u8,
        message: String,
    },
    /// Encode thất bại — input không hợp lệ.
    EncodeFailed {
        reason: String,
    },
}

impl HomeError {
    /// Có thể retry không?
    pub fn is_retryable(&self) -> bool {
        matches!(self, HomeError::NetworkFailure { attempt, max_attempts, .. } if attempt < max_attempts)
    }

    /// Mô tả lỗi.
    pub fn describe(&self) -> String {
        match self {
            HomeError::DiskFull { pending_bytes, message } =>
                format!("Disk full: {} bytes pending — {}", pending_bytes, message),
            HomeError::CorruptFile { recovered_records, lost_bytes } =>
                format!("Corrupt file: recovered {} records, lost {} bytes", recovered_records, lost_bytes),
            HomeError::NetworkFailure { attempt, max_attempts, message } =>
                format!("Network failure (attempt {}/{}): {}", attempt, max_attempts, message),
            HomeError::EncodeFailed { reason } =>
                format!("Encode failed: {}", reason),
        }
    }
}

/// Fibonacci-based retry delay (ms): 1, 1, 2, 3, 5, 8, 13, 21...
///
/// attempt 0 → 1000ms, attempt 1 → 1000ms, attempt 2 → 2000ms, etc.
pub fn fib_retry_delay_ms(attempt: u8) -> u64 {
    let mut a = 1u64;
    let mut b = 1u64;
    for _ in 0..attempt {
        let c = a + b;
        a = b;
        b = c;
    }
    a * 1000 // seconds → milliseconds
}

/// Max retries trước khi give up.
pub const MAX_RETRIES: u8 = 5;

/// Kết quả persist — thành công hoặc degraded.
#[derive(Debug, Clone)]
pub enum PersistResult {
    /// Ghi thành công.
    Ok { bytes_written: usize },
    /// Ghi thất bại — data vẫn trong pending_writes.
    DiskFull { pending_bytes: usize },
    /// Không có gì để ghi.
    NoPending,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fib_retry_delays() {
        assert_eq!(fib_retry_delay_ms(0), 1000);
        assert_eq!(fib_retry_delay_ms(1), 1000);
        assert_eq!(fib_retry_delay_ms(2), 2000);
        assert_eq!(fib_retry_delay_ms(3), 3000);
        assert_eq!(fib_retry_delay_ms(4), 5000);
    }

    #[test]
    fn error_retryable() {
        let e = HomeError::NetworkFailure {
            attempt: 2, max_attempts: 5,
            message: String::from("timeout"),
        };
        assert!(e.is_retryable());

        let e2 = HomeError::NetworkFailure {
            attempt: 5, max_attempts: 5,
            message: String::from("timeout"),
        };
        assert!(!e2.is_retryable());
    }

    #[test]
    fn error_describe() {
        let e = HomeError::DiskFull {
            pending_bytes: 1024,
            message: String::from("no space"),
        };
        assert!(e.describe().contains("1024"));
    }
}
