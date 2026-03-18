//! key.ol export/import — backup & recovery for master key.
//!
//! Format (104 bytes):
//!   [encrypted_seed: 48B]  AES-256-GCM(aes_key, seed) → 32 seed + 16 tag
//!   [master_pubkey: 32B]   to verify correct key
//!   [username_hash: 8B]    to verify correct user
//!   [created_ts: 8B]       timestamp
//!   [origin_id: 8B]        hash of original origin.olang

extern crate alloc;

use alloc::vec::Vec;
use olang::aes256gcm::{aes256gcm_decrypt, aes256gcm_encrypt};
use olang::sha256::Sha256;
use olang::ed25519::SigningKey;

use super::AuthError;
use super::key;

/// key.ol file size in bytes.
pub const KEY_FILE_SIZE: usize = 104;

/// Export master key to encrypted key.ol bytes.
///
/// The seed is encrypted with an AES key derived from the password.
/// Only someone with the password can decrypt and recover the signing key.
pub fn export_key(
    username: &str,
    password: &str,
    pubkey: &[u8; 32],
    origin_id: u64,
    ts: i64,
) -> Vec<u8> {
    let (signing_key, _) = key::derive_keypair(username, password);
    let seed = signing_key.seed();

    // Derive AES-256 key from password (different derivation than Ed25519)
    let aes_key = derive_aes_key(password);
    let nonce = [0u8; 12]; // deterministic for same password

    // Encrypt seed → 32 + 16 = 48 bytes
    let encrypted = aes256gcm_encrypt(&aes_key, &nonce, seed, b"key.ol");

    // Build output
    let mut output = Vec::with_capacity(KEY_FILE_SIZE);
    output.extend_from_slice(&encrypted);           // 48B
    output.extend_from_slice(pubkey);                // 32B
    output.extend_from_slice(&hash_username(username).to_le_bytes()); // 8B
    output.extend_from_slice(&ts.to_le_bytes());     // 8B
    output.extend_from_slice(&origin_id.to_le_bytes()); // 8B
    output
}

/// Import key.ol → verify + recover signing key.
///
/// Returns the SigningKey if password is correct and key matches.
pub fn import_key(
    key_data: &[u8],
    password: &str,
) -> Result<(SigningKey, [u8; 32]), AuthError> {
    if key_data.len() != KEY_FILE_SIZE {
        return Err(AuthError::InvalidKeyFile);
    }

    let encrypted_seed = &key_data[0..48];
    let expected_pubkey: [u8; 32] = key_data[48..80].try_into().unwrap();

    // Decrypt seed
    let aes_key = derive_aes_key(password);
    let nonce = [0u8; 12];
    let seed_vec = aes256gcm_decrypt(&aes_key, &nonce, encrypted_seed, b"key.ol")
        .ok_or(AuthError::WrongPassword)?;

    if seed_vec.len() != 32 {
        return Err(AuthError::InvalidKeyFile);
    }

    let mut seed = [0u8; 32];
    seed.copy_from_slice(&seed_vec);

    // Rebuild signing key and verify pubkey match
    let signing_key = SigningKey::from_bytes(&seed);
    if signing_key.verifying_key().as_bytes() != &expected_pubkey {
        return Err(AuthError::KeyMismatch);
    }

    Ok((signing_key, expected_pubkey))
}

/// Derive AES-256 key from password (separate from Ed25519 derivation).
///
/// Uses SHA-256(SHA-256(password) || "key.ol.aes") — simple but sufficient
/// since the underlying seed is already high-entropy Ed25519 material.
fn derive_aes_key(password: &str) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(password.as_bytes());
    let pass_hash = h.finalize();

    let mut h2 = Sha256::new();
    h2.update(&pass_hash);
    h2.update(b"key.ol.aes");
    h2.finalize()
}

/// Hash username for identity check in key.ol.
fn hash_username(username: &str) -> u64 {
    let mut h = Sha256::new();
    h.update(username.as_bytes());
    let digest = h.finalize();
    u64::from_le_bytes(digest[0..8].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_import_roundtrip() {
        let (_, pk) = key::derive_keypair("user", "password123");
        let exported = export_key("user", "password123", pk.as_bytes(), 42, 1710720000);
        assert_eq!(exported.len(), KEY_FILE_SIZE);

        let (imported_sk, imported_pk) = import_key(&exported, "password123").unwrap();
        assert_eq!(&imported_pk, pk.as_bytes());
        assert_eq!(imported_sk.verifying_key().as_bytes(), pk.as_bytes());
    }

    #[test]
    fn test_import_wrong_password() {
        let (_, pk) = key::derive_keypair("user", "password123");
        let exported = export_key("user", "password123", pk.as_bytes(), 42, 1710720000);
        let result = import_key(&exported, "wrong_password");
        assert!(result.is_err());
    }

    #[test]
    fn test_import_invalid_size() {
        let result = import_key(&[0u8; 50], "pass");
        assert!(matches!(result, Err(AuthError::InvalidKeyFile)));
    }

    #[test]
    fn test_import_empty() {
        let result = import_key(&[], "pass");
        assert!(matches!(result, Err(AuthError::InvalidKeyFile)));
    }

    #[test]
    fn test_export_deterministic() {
        let (_, pk) = key::derive_keypair("user", "pass456");
        let a = export_key("user", "pass456", pk.as_bytes(), 1, 100);
        let b = export_key("user", "pass456", pk.as_bytes(), 1, 100);
        assert_eq!(a, b);
    }

    #[test]
    fn test_derive_aes_key_deterministic() {
        let k1 = derive_aes_key("hello");
        let k2 = derive_aes_key("hello");
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_derive_aes_key_different() {
        let k1 = derive_aes_key("hello");
        let k2 = derive_aes_key("world");
        assert_ne!(k1, k2);
    }
}
