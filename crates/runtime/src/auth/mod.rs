//! Authentication: master key, first-run setup, ISL signing.
//!
//! Provides identity and authorization for origin.olang:
//! - First-run wizard: terms → create master key → lock ISL chain
//! - Login: password → derive key → verify match
//! - ISL signing: tier-0 commands signed by master key
//! - key.ol export/import: backup & recovery

extern crate alloc;

pub mod export;
pub mod key;
pub mod setup;
pub mod sign;
pub mod terms;
pub mod verify;

use olang::ed25519::SigningKey;

/// Authentication state machine.
#[allow(missing_debug_implementations)]
pub enum AuthState {
    /// No master key yet — need first-run wizard.
    Virgin,
    /// Master key exists but session not unlocked — need password.
    Locked,
    /// Session unlocked — signing key available.
    Unlocked { signing_key: SigningKey },
}

/// Header extension for auth data (appended after origin.olang header).
///
/// Total: 113 bytes.
#[derive(Clone, Debug)]
pub struct AuthHeader {
    /// Ed25519 public key (master identity).
    pub master_pubkey: [u8; 32],
    /// Salt for key derivation (random, generated at setup).
    pub salt: [u8; 16],
    /// AES-256-GCM encrypted seed (32 seed + 16 tag) — for biometric unlock.
    pub bio_encrypted_seed: [u8; 48],
    /// Biometric method: 0=none, 1=fingerprint, 2=face, 3=voice.
    pub bio_method: u8,
    /// Timestamp of first setup.
    pub setup_ts: i64,
    /// Hash of terms text that user accepted.
    pub terms_hash: u64,
}

impl AuthHeader {
    /// Check if this origin.olang has never been set up.
    pub fn is_virgin(&self) -> bool {
        self.master_pubkey == [0u8; 32]
    }

    /// Serialize to 113 bytes.
    pub fn to_bytes(&self) -> [u8; 113] {
        let mut out = [0u8; 113];
        out[0..32].copy_from_slice(&self.master_pubkey);
        out[32..48].copy_from_slice(&self.salt);
        out[48..96].copy_from_slice(&self.bio_encrypted_seed);
        out[96] = self.bio_method;
        out[97..105].copy_from_slice(&self.setup_ts.to_le_bytes());
        out[105..113].copy_from_slice(&self.terms_hash.to_le_bytes());
        out
    }

    /// Deserialize from 113 bytes.
    pub fn from_bytes(data: &[u8; 113]) -> Self {
        let mut master_pubkey = [0u8; 32];
        master_pubkey.copy_from_slice(&data[0..32]);
        let mut salt = [0u8; 16];
        salt.copy_from_slice(&data[32..48]);
        let mut bio_encrypted_seed = [0u8; 48];
        bio_encrypted_seed.copy_from_slice(&data[48..96]);
        let bio_method = data[96];
        let setup_ts = i64::from_le_bytes(data[97..105].try_into().unwrap());
        let terms_hash = u64::from_le_bytes(data[105..113].try_into().unwrap());
        Self {
            master_pubkey,
            salt,
            bio_encrypted_seed,
            bio_method,
            setup_ts,
            terms_hash,
        }
    }

    /// Create an empty (virgin) header.
    pub fn virgin() -> Self {
        Self {
            master_pubkey: [0u8; 32],
            salt: [0u8; 16],
            bio_encrypted_seed: [0u8; 48],
            bio_method: 0,
            setup_ts: 0,
            terms_hash: 0,
        }
    }
}

/// Auth errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    /// User rejected terms of use.
    TermsRejected,
    /// Password and confirmation don't match.
    PasswordMismatch,
    /// Password too short (minimum 8 characters).
    PasswordTooShort,
    /// Wrong password during login or import.
    WrongPassword,
    /// key.ol file is invalid format.
    InvalidKeyFile,
    /// Imported key doesn't match expected pubkey.
    KeyMismatch,
    /// I/O error (setup wizard, std only).
    Io(alloc::string::String),
}

impl core::fmt::Display for AuthError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TermsRejected => write!(f, "Terms of use rejected"),
            Self::PasswordMismatch => write!(f, "Passwords do not match"),
            Self::PasswordTooShort => write!(f, "Password must be at least 8 characters"),
            Self::WrongPassword => write!(f, "Wrong password"),
            Self::InvalidKeyFile => write!(f, "Invalid key.ol file"),
            Self::KeyMismatch => write!(f, "Imported key does not match"),
            Self::Io(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_header_virgin() {
        let h = AuthHeader::virgin();
        assert!(h.is_virgin());
    }

    #[test]
    fn test_auth_header_roundtrip() {
        let h = AuthHeader {
            master_pubkey: [0xAB; 32],
            salt: [0xCD; 16],
            bio_encrypted_seed: [0xEF; 48],
            bio_method: 1,
            setup_ts: 1710720000,
            terms_hash: 0xDEADBEEF,
        };
        assert!(!h.is_virgin());
        let bytes = h.to_bytes();
        let h2 = AuthHeader::from_bytes(&bytes);
        assert_eq!(h2.master_pubkey, h.master_pubkey);
        assert_eq!(h2.salt, h.salt);
        assert_eq!(h2.bio_encrypted_seed, h.bio_encrypted_seed);
        assert_eq!(h2.bio_method, 1);
        assert_eq!(h2.setup_ts, 1710720000);
        assert_eq!(h2.terms_hash, 0xDEADBEEF);
    }

    #[test]
    fn test_auth_header_not_virgin() {
        let mut h = AuthHeader::virgin();
        h.master_pubkey[0] = 1;
        assert!(!h.is_virgin());
    }
}
