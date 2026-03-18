//! First-run setup wizard (std-only, interactive).
//!
//! Shows terms → accepts username/password → creates AuthHeader.
//! Only runs when AuthHeader.is_virgin().

extern crate alloc;

use super::{AuthError, AuthHeader};
use super::key;
use super::terms;

/// Minimum password length.
const MIN_PASSWORD_LEN: usize = 8;

/// Create AuthHeader from validated credentials.
///
/// This is the core logic — no I/O, no stdin. The interactive wizard
/// (std-only) calls this after collecting input.
pub fn create_auth_header(
    username: &str,
    password: &str,
    ts: i64,
) -> Result<AuthHeader, AuthError> {
    if password.len() < MIN_PASSWORD_LEN {
        return Err(AuthError::PasswordTooShort);
    }

    let (_, verifying_key) = key::derive_keypair(username, password);

    // Generate salt (deterministic from username for reproducibility)
    let mut salt = [0u8; 16];
    let username_bytes = username.as_bytes();
    for (i, s) in salt.iter_mut().enumerate() {
        *s = if i < username_bytes.len() {
            username_bytes[i]
        } else {
            (i as u8).wrapping_mul(0x9E).wrapping_add(0x37)
        };
    }

    Ok(AuthHeader {
        master_pubkey: *verifying_key.as_bytes(),
        salt,
        bio_encrypted_seed: [0u8; 48], // no biometric at setup
        bio_method: 0,
        setup_ts: ts,
        terms_hash: terms::terms_hash(),
    })
}

/// Validate that password meets requirements.
pub fn validate_password(password: &str) -> Result<(), AuthError> {
    if password.len() < MIN_PASSWORD_LEN {
        Err(AuthError::PasswordTooShort)
    } else {
        Ok(())
    }
}

/// Validate that password and confirmation match.
pub fn validate_password_match(password: &str, confirm: &str) -> Result<(), AuthError> {
    if password != confirm {
        Err(AuthError::PasswordMismatch)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_auth_header() {
        let h = create_auth_header("alice", "strong_password_42", 1710720000).unwrap();
        assert!(!h.is_virgin());
        assert_eq!(h.setup_ts, 1710720000);
        assert_eq!(h.bio_method, 0);
        assert_ne!(h.terms_hash, 0);
        assert_ne!(h.master_pubkey, [0u8; 32]);
    }

    #[test]
    fn test_create_auth_header_deterministic() {
        let h1 = create_auth_header("alice", "strong_password_42", 100).unwrap();
        let h2 = create_auth_header("alice", "strong_password_42", 100).unwrap();
        assert_eq!(h1.master_pubkey, h2.master_pubkey);
    }

    #[test]
    fn test_create_auth_header_password_too_short() {
        let result = create_auth_header("alice", "short", 100);
        assert!(matches!(result, Err(AuthError::PasswordTooShort)));
    }

    #[test]
    fn test_validate_password() {
        assert!(validate_password("12345678").is_ok());
        assert!(validate_password("1234567").is_err());
        assert!(validate_password("").is_err());
    }

    #[test]
    fn test_validate_password_match() {
        assert!(validate_password_match("abc", "abc").is_ok());
        assert!(validate_password_match("abc", "def").is_err());
    }

    #[test]
    fn test_setup_then_verify() {
        let h = create_auth_header("bob", "my_secure_pw_99", 200).unwrap();
        assert!(key::verify_password("bob", "my_secure_pw_99", &h.master_pubkey));
        assert!(!key::verify_password("bob", "wrong_pw", &h.master_pubkey));
    }
}
