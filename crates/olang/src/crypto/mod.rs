//! Cryptography: native implementations, zero external dependencies.
//!
//! SHA-256, SHA-512, Ed25519 signatures, AES-256-GCM, QR signing.

pub mod aes256gcm;
pub mod ed25519;
pub mod qr;
pub mod sha256;
pub mod sha512;
