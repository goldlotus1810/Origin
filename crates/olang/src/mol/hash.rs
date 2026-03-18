//! # hash — FNV-1a hash utilities
//!
//! Shared FNV-1a hashing — dùng xuyên suốt HomeOS.
//! Tất cả các crate import từ đây — không duplicate.

/// FNV-1a offset basis.
pub const FNV_OFFSET: u64 = 0xcbf29ce484222325;
/// FNV-1a prime.
pub const FNV_PRIME: u64 = 0x100000001b3;

/// FNV-1a hash cho bytes.
#[inline]
pub fn fnv1a(data: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// FNV-1a hash cho string (lowercase bytes).
#[inline]
pub fn fnv1a_str(s: &str) -> u64 {
    let mut h = FNV_OFFSET;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// FNV-1a hash với namespace prefix — tránh collision giữa domains.
///
/// Namespace byte được hash trước data bytes.
#[inline]
pub fn fnv1a_namespaced(namespace: u8, data: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    h ^= namespace as u64;
    h = h.wrapping_mul(FNV_PRIME);
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_deterministic() {
        let h1 = fnv1a(b"hello");
        let h2 = fnv1a(b"hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn fnv1a_different_inputs() {
        assert_ne!(fnv1a(b"hello"), fnv1a(b"world"));
    }

    #[test]
    fn fnv1a_str_matches_bytes() {
        assert_eq!(fnv1a_str("hello"), fnv1a(b"hello"));
    }

    #[test]
    fn fnv1a_namespaced_different() {
        let h1 = fnv1a_namespaced(0xAA, &[1]);
        let h2 = fnv1a_namespaced(0xBB, &[1]);
        assert_ne!(h1, h2, "Different namespaces → different hashes");
    }

    #[test]
    fn fnv1a_empty() {
        assert_eq!(fnv1a(b""), FNV_OFFSET);
    }
}
