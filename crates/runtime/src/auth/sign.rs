//! ISL message signing and verification.
//!
//! Tier-0 ISL commands (Approved, Emergency, Program) MUST be signed
//! by master key. Workers/Chiefs verify before executing.

use olang::ed25519::{SigningKey, VerifyingKey, Signature};

/// ISL message types that require signature.
const SIGNED_MSG_TYPES: &[u8] = &[
    0x09, // Approved
    0x08, // Emergency
    0x0E, // Program
];

/// Check if a message type requires signature.
pub fn requires_signature(msg_type: u8) -> bool {
    SIGNED_MSG_TYPES.contains(&msg_type)
}

/// Sign an ISL message body.
///
/// Returns 64-byte Ed25519 signature.
pub fn sign_isl(key: &SigningKey, message: &[u8]) -> [u8; 64] {
    key.sign(message).to_bytes()
}

/// Verify ISL message signature.
///
/// - `pubkey`: 32-byte master public key (from AuthHeader)
/// - `message`: the ISL message bytes that were signed
/// - `signature`: 64-byte Ed25519 signature
pub fn verify_isl(pubkey: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool {
    let vk = VerifyingKey::from_bytes(pubkey);
    let sig = Signature::from_bytes(signature);
    vk.verify(message, &sig).is_ok()
}

/// Build the signable payload from ISL frame components.
///
/// Format: [from:4][to:4][msg_type:1][payload:3][body...]
pub fn build_signable(
    from: &[u8; 4],
    to: &[u8; 4],
    msg_type: u8,
    payload: &[u8; 3],
    body: &[u8],
) -> alloc::vec::Vec<u8> {
    let mut buf = alloc::vec::Vec::with_capacity(12 + body.len());
    buf.extend_from_slice(from);
    buf.extend_from_slice(to);
    buf.push(msg_type);
    buf.extend_from_slice(payload);
    buf.extend_from_slice(body);
    buf
}

extern crate alloc;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::key::derive_keypair;

    #[test]
    fn test_sign_verify_isl() {
        let (sk, vk) = derive_keypair("user", "pass_for_isl_test");
        let msg = b"AAM approve worker_light_0";
        let sig = sign_isl(&sk, msg);
        assert!(verify_isl(vk.as_bytes(), msg, &sig));
    }

    #[test]
    fn test_verify_wrong_key() {
        let (sk, _) = derive_keypair("user", "pass1");
        let (_, vk2) = derive_keypair("user", "pass2");
        let msg = b"AAM approve";
        let sig = sign_isl(&sk, msg);
        assert!(!verify_isl(vk2.as_bytes(), msg, &sig));
    }

    #[test]
    fn test_verify_tampered_message() {
        let (sk, vk) = derive_keypair("user", "pass_tamper_test");
        let msg = b"AAM approve worker_light_0";
        let sig = sign_isl(&sk, msg);
        let tampered = b"AAM approve worker_door_0";
        assert!(!verify_isl(vk.as_bytes(), tampered, &sig));
    }

    #[test]
    fn test_requires_signature() {
        assert!(requires_signature(0x09)); // Approved
        assert!(requires_signature(0x08)); // Emergency
        assert!(requires_signature(0x0E)); // Program
        assert!(!requires_signature(0x01)); // Text
        assert!(!requires_signature(0x02)); // Query
        assert!(!requires_signature(0x06)); // Tick
    }

    #[test]
    fn test_build_signable() {
        let from = [0x00, 0x00, 0x00, 0x00]; // AAM
        let to = [0x01, 0x00, 0x00, 0x01];   // LeoAI
        let msg_type = 0x09;                    // Approved
        let payload = [0x00, 0x00, 0x00];
        let body = b"approve_worker";
        let signable = build_signable(&from, &to, msg_type, &payload, body);
        assert_eq!(signable.len(), 12 + body.len());
        assert_eq!(signable[8], 0x09);
    }

    #[test]
    fn test_sign_verify_with_build_signable() {
        let (sk, vk) = derive_keypair("admin", "strong_password_42");
        let from = [0x00, 0x00, 0x00, 0x00];
        let to = [0x01, 0x00, 0x00, 0x01];
        let signable = build_signable(&from, &to, 0x09, &[0; 3], b"payload_data");
        let sig = sign_isl(&sk, &signable);
        assert!(verify_isl(vk.as_bytes(), &signable, &sig));
    }
}
