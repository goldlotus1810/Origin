//! Key derivation: password → PBKDF2-HMAC-SHA512 → Ed25519 keypair.
//!
//! Uses native SHA-512 (no external deps). PBKDF2 with 100_000 iterations.
//! Salt = SHA-256(username) to ensure different users get different keys.

use olang::sha256::Sha256;
use olang::sha512::Sha512;
use olang::ed25519::{SigningKey, VerifyingKey};

/// PBKDF2 iteration count — balance security vs. derivation time.
/// 100k iterations of HMAC-SHA512 ≈ 200ms on modern hardware.
const PBKDF2_ITERATIONS: u32 = 100_000;

/// Derive Ed25519 keypair from username + password.
///
/// Process: username → salt, password + salt → PBKDF2-HMAC-SHA512 → 32-byte seed → Ed25519.
/// Deterministic: same (username, password) → same keypair always.
pub fn derive_keypair(username: &str, password: &str) -> (SigningKey, VerifyingKey) {
    // Salt = SHA-256(username) — 32 bytes, truncate to 16 for PBKDF2
    let mut h = Sha256::new();
    h.update(username.as_bytes());
    let username_hash = h.finalize();
    let salt = &username_hash[..16];

    // PBKDF2-HMAC-SHA512 → 32-byte seed
    let seed = pbkdf2_hmac_sha512(password.as_bytes(), salt, PBKDF2_ITERATIONS);

    // Seed → Ed25519 keypair
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// Verify that password produces the expected public key.
pub fn verify_password(username: &str, password: &str, expected_pubkey: &[u8; 32]) -> bool {
    let (_, verifying_key) = derive_keypair(username, password);
    verifying_key.as_bytes() == expected_pubkey
}

/// PBKDF2-HMAC-SHA512 (RFC 2898 / RFC 8018).
///
/// Derives a 32-byte key from password + salt using iterated HMAC-SHA512.
fn pbkdf2_hmac_sha512(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    // We only need 32 bytes → 1 block (SHA-512 output = 64 bytes, take first 32)
    let block = pbkdf2_f(password, salt, iterations, 1);
    let mut out = [0u8; 32];
    out.copy_from_slice(&block[..32]);
    out
}

/// PBKDF2 F function: U_1 = HMAC(password, salt || INT(block_index))
/// U_i = HMAC(password, U_{i-1}), result = U_1 XOR U_2 XOR ... XOR U_c
fn pbkdf2_f(password: &[u8], salt: &[u8], iterations: u32, block_index: u32) -> [u8; 64] {
    // U_1 = HMAC-SHA512(password, salt || block_index_be)
    let mut msg = alloc::vec::Vec::with_capacity(salt.len() + 4);
    msg.extend_from_slice(salt);
    msg.extend_from_slice(&block_index.to_be_bytes());

    let mut u = hmac_sha512(password, &msg);
    let mut result = u;

    // U_2 .. U_c
    for _ in 1..iterations {
        u = hmac_sha512(password, &u);
        for j in 0..64 {
            result[j] ^= u[j];
        }
    }
    result
}

/// HMAC-SHA512 (RFC 2104).
fn hmac_sha512(key: &[u8], message: &[u8]) -> [u8; 64] {
    const BLOCK_SIZE: usize = 128; // SHA-512 block = 128 bytes

    // If key > block size, hash it
    let mut k = [0u8; BLOCK_SIZE];
    if key.len() > BLOCK_SIZE {
        let mut h = Sha512::new();
        h.update(key);
        let hashed = h.finalize();
        k[..64].copy_from_slice(&hashed);
    } else {
        k[..key.len()].copy_from_slice(key);
    }

    // ipad = k XOR 0x36, opad = k XOR 0x5c
    let mut ipad = [0x36u8; BLOCK_SIZE];
    let mut opad = [0x5cu8; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }

    // inner = SHA-512(ipad || message)
    let mut inner = Sha512::new();
    inner.update(&ipad);
    inner.update(message);
    let inner_hash = inner.finalize();

    // outer = SHA-512(opad || inner_hash)
    let mut outer = Sha512::new();
    outer.update(&opad);
    outer.update(&inner_hash);
    outer.finalize()
}

extern crate alloc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation_deterministic() {
        let (_, pk1) = derive_keypair("user", "password123");
        let (_, pk2) = derive_keypair("user", "password123");
        assert_eq!(pk1.as_bytes(), pk2.as_bytes(), "same input → same key");
    }

    #[test]
    fn test_key_derivation_different_users() {
        let (_, pk1) = derive_keypair("alice", "password123");
        let (_, pk2) = derive_keypair("bob", "password123");
        assert_ne!(pk1.as_bytes(), pk2.as_bytes(), "different username → different key");
    }

    #[test]
    fn test_key_derivation_different_passwords() {
        let (_, pk1) = derive_keypair("user", "password_a");
        let (_, pk2) = derive_keypair("user", "password_b");
        assert_ne!(pk1.as_bytes(), pk2.as_bytes(), "different password → different key");
    }

    #[test]
    fn test_verify_password_correct() {
        let (_, pk) = derive_keypair("user", "correct_pass");
        assert!(verify_password("user", "correct_pass", pk.as_bytes()));
    }

    #[test]
    fn test_verify_password_wrong() {
        let (_, pk) = derive_keypair("user", "correct_pass");
        assert!(!verify_password("user", "wrong_pass", pk.as_bytes()));
    }

    #[test]
    fn test_hmac_sha512_not_zero() {
        let result = hmac_sha512(b"key", b"message");
        assert_ne!(result, [0u8; 64]);
    }

    #[test]
    fn test_pbkdf2_not_zero() {
        let result = pbkdf2_hmac_sha512(b"pass", b"salt", 1);
        assert_ne!(result, [0u8; 32]);
    }

    #[test]
    fn test_pbkdf2_deterministic() {
        let a = pbkdf2_hmac_sha512(b"pass", b"salt", 10);
        let b = pbkdf2_hmac_sha512(b"pass", b"salt", 10);
        assert_eq!(a, b);
    }

    #[test]
    fn test_pbkdf2_iterations_matter() {
        let a = pbkdf2_hmac_sha512(b"pass", b"salt", 1);
        let b = pbkdf2_hmac_sha512(b"pass", b"salt", 2);
        assert_ne!(a, b, "different iterations → different output");
    }
}
