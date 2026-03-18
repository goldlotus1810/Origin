//! Login verification: password → derive key → check against stored pubkey.

use super::{AuthError, AuthHeader, AuthState};
use super::key;
use olang::ed25519::SigningKey;

/// Attempt to unlock with username + password.
///
/// Returns `Unlocked` state with signing key on success.
pub fn unlock(
    header: &AuthHeader,
    username: &str,
    password: &str,
) -> Result<AuthState, AuthError> {
    if header.is_virgin() {
        return Err(AuthError::WrongPassword);
    }
    if key::verify_password(username, password, &header.master_pubkey) {
        let (signing_key, _) = key::derive_keypair(username, password);
        Ok(AuthState::Unlocked { signing_key })
    } else {
        Err(AuthError::WrongPassword)
    }
}

/// Verify that a signing key matches the stored pubkey.
pub fn verify_key_matches(header: &AuthHeader, signing_key: &SigningKey) -> bool {
    signing_key.verifying_key().as_bytes() == &header.master_pubkey
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::key::derive_keypair;

    fn make_header(username: &str, password: &str) -> AuthHeader {
        let (_, vk) = derive_keypair(username, password);
        AuthHeader {
            master_pubkey: *vk.as_bytes(),
            salt: [0u8; 16],
            bio_encrypted_seed: [0u8; 48],
            bio_method: 0,
            setup_ts: 1710720000,
            terms_hash: 0,
        }
    }

    #[test]
    fn test_unlock_correct_password() {
        let header = make_header("alice", "secure_pass_123");
        let result = unlock(&header, "alice", "secure_pass_123");
        assert!(result.is_ok());
        match result.unwrap() {
            AuthState::Unlocked { signing_key } => {
                assert!(verify_key_matches(&header, &signing_key));
            }
            _ => panic!("expected Unlocked"),
        }
    }

    #[test]
    fn test_unlock_wrong_password() {
        let header = make_header("alice", "secure_pass_123");
        let result = unlock(&header, "alice", "wrong_password");
        assert!(matches!(result, Err(AuthError::WrongPassword)));
    }

    #[test]
    fn test_unlock_virgin_header() {
        let header = AuthHeader::virgin();
        let result = unlock(&header, "alice", "any_pass");
        assert!(matches!(result, Err(AuthError::WrongPassword)));
    }

    #[test]
    fn test_verify_key_matches() {
        let (sk, vk) = derive_keypair("bob", "my_password");
        let header = AuthHeader {
            master_pubkey: *vk.as_bytes(),
            salt: [0u8; 16],
            bio_encrypted_seed: [0u8; 48],
            bio_method: 0,
            setup_ts: 0,
            terms_hash: 0,
        };
        assert!(verify_key_matches(&header, &sk));
    }
}
