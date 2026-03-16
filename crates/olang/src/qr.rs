//! # qr — QR Signing + Supersession
//!
//! QR (Quang Trọng) = node đã được chứng minh, bất biến mãi mãi.
//!
//! ## ED25519 Signing:
//!   Mỗi QR node được ký bằng ED25519.
//!   Signature = bằng chứng không thể giả mạo.
//!   Tamper QR → verify fail → reject.
//!
//! ## QR Supersession (không xóa — thêm):
//!   Khoa học cũng sai. QR cũ có thể được supersede.
//!   QR_A (cũ) vẫn tồn tại trong sổ cái (QT8).
//!   QR_B (mới) được tạo với SupersedesQR edge đến QR_A.
//!   Query QR_A → nhận QR_B + ghi chú "deprecated".
//!
//! ## Wire format QR record:
//!   [chain_bytes: N×5][chain_hash: 8][timestamp: 8][signature: 64] = N×5 + 80 bytes

extern crate alloc;
use alloc::vec::Vec;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

use crate::molecular::MolecularChain;

// ─────────────────────────────────────────────────────────────────────────────
// QR Signing
// ─────────────────────────────────────────────────────────────────────────────

/// Dữ liệu được ký trong một QR node.
///
/// message = SHA256(chain_bytes || chain_hash_le || timestamp_le)
#[derive(Debug, Clone)]
pub struct QRRecord {
    /// Chain của node
    pub chain: MolecularChain,
    /// FNV-1a hash của chain
    pub hash: u64,
    /// Timestamp khi QR được approve (nanoseconds)
    pub timestamp: i64,
    /// ED25519 signature của message
    pub signature: [u8; 64],
}

impl QRRecord {
    /// Tạo message bytes để sign/verify.
    ///
    /// message = SHA256(chain_bytes || hash_le8 || ts_le8)
    pub fn message_bytes(chain: &MolecularChain, hash: u64, timestamp: i64) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(chain.to_bytes());
        hasher.update(hash.to_le_bytes());
        hasher.update(timestamp.to_le_bytes());
        let result = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result);
        out
    }

    /// Serialize → bytes để ghi vào file.
    ///
    /// Format: [chain_len: u8][chain: N×5][hash: 8][ts: 8][sig: 64]
    pub fn to_bytes(&self) -> Vec<u8> {
        let chain_bytes = self.chain.to_bytes();
        let mut out = Vec::with_capacity(1 + chain_bytes.len() + 8 + 8 + 64);
        out.push((chain_bytes.len() / 5) as u8);
        out.extend_from_slice(&chain_bytes);
        out.extend_from_slice(&self.hash.to_le_bytes());
        out.extend_from_slice(&self.timestamp.to_le_bytes());
        out.extend_from_slice(&self.signature);
        out
    }

    /// Deserialize từ bytes.
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 1 + 5 + 8 + 8 + 64 {
            return None;
        }

        let chain_len = data[0] as usize;
        let chain_bytes_len = chain_len * 5;
        let needed = 1 + chain_bytes_len + 8 + 8 + 64;
        if data.len() < needed {
            return None;
        }

        let chain = MolecularChain::from_bytes(&data[1..1 + chain_bytes_len])?;
        let mut pos = 1 + chain_bytes_len;

        let hash = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
        pos += 8;
        let ts = i64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
        pos += 8;
        let mut sig = [0u8; 64];
        sig.copy_from_slice(&data[pos..pos + 64]);

        Some(Self {
            chain,
            hash,
            timestamp: ts,
            signature: sig,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// QRSigner — AAM dùng để sign QR
// ─────────────────────────────────────────────────────────────────────────────

/// AAM's signing key — dùng để approve QR nodes.
///
/// Trong production: key được load từ secure storage.
/// Trong test: key được generate từ seed.
pub struct QRSigner {
    signing_key: SigningKey,
}

impl QRSigner {
    /// Tạo từ 32-byte seed.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        Self {
            signing_key: SigningKey::from_bytes(seed),
        }
    }

    /// Verifying key (public) — dùng để verify signature.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Sign một chain → QRRecord.
    ///
    /// Đây là bước AAM approve một node thành QR.
    pub fn sign_qr(&self, chain: &MolecularChain, timestamp: i64) -> QRRecord {
        let hash = chain.chain_hash();
        let message = QRRecord::message_bytes(chain, hash, timestamp);
        let sig: Signature = self.signing_key.sign(&message);
        QRRecord {
            chain: chain.clone(),
            hash,
            timestamp,
            signature: sig.to_bytes(),
        }
    }

    /// Verify một QRRecord.
    pub fn verify(&self, record: &QRRecord) -> bool {
        verify_qr(record, &self.verifying_key())
    }
}

/// Verify QRRecord với public key.
pub fn verify_qr(record: &QRRecord, verifying_key: &VerifyingKey) -> bool {
    let message = QRRecord::message_bytes(&record.chain, record.hash, record.timestamp);
    let sig = Signature::from_bytes(&record.signature);
    verifying_key.verify(&message, &sig).is_ok()
}

// ─────────────────────────────────────────────────────────────────────────────
// QR Supersession
// ─────────────────────────────────────────────────────────────────────────────

/// Một QR supersession record.
///
/// QR_old bị supersede bởi QR_new.
/// QR_old vẫn tồn tại (QT8 — không xóa).
/// Supersession record cũng được ký bởi AAM.
#[derive(Debug, Clone)]
pub struct QRSupersessionRecord {
    /// Hash của QR cũ (deprecated)
    pub old_hash: u64,
    /// Hash của QR mới (đúng hơn)
    pub new_hash: u64,
    /// Lý do supersede (ngắn gọn)
    pub reason: [u8; 32],
    /// Timestamp
    pub timestamp: i64,
    /// ED25519 signature của (old_hash || new_hash || reason || timestamp)
    pub signature: [u8; 64],
}

impl QRSupersessionRecord {
    /// Message bytes để sign.
    pub fn message_bytes(old: u64, new: u64, reason: &[u8; 32], ts: i64) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(old.to_le_bytes());
        hasher.update(new.to_le_bytes());
        hasher.update(reason.as_slice());
        hasher.update(ts.to_le_bytes());
        let result = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result);
        out
    }

    /// Serialize → bytes.
    ///
    /// Format: [old: 8][new: 8][reason: 32][ts: 8][sig: 64] = 120 bytes
    pub fn to_bytes(&self) -> [u8; 120] {
        let mut out = [0u8; 120];
        out[0..8].copy_from_slice(&self.old_hash.to_le_bytes());
        out[8..16].copy_from_slice(&self.new_hash.to_le_bytes());
        out[16..48].copy_from_slice(&self.reason);
        out[48..56].copy_from_slice(&self.timestamp.to_le_bytes());
        out[56..120].copy_from_slice(&self.signature);
        out
    }

    /// Deserialize từ 120 bytes.
    pub fn from_bytes(data: &[u8; 120]) -> Self {
        let old = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let new = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let mut reason = [0u8; 32];
        reason.copy_from_slice(&data[16..48]);
        let ts = i64::from_le_bytes(data[48..56].try_into().unwrap());
        let mut sig = [0u8; 64];
        sig.copy_from_slice(&data[56..120]);
        Self {
            old_hash: old,
            new_hash: new,
            reason,
            timestamp: ts,
            signature: sig,
        }
    }
}

impl QRSigner {
    /// Supersede QR_old bằng QR_new.
    ///
    /// Tạo QRSupersessionRecord được ký bởi AAM.
    /// QR_old vẫn tồn tại — chỉ thêm record mới (QT8).
    pub fn supersede(
        &self,
        old_hash: u64,
        new_hash: u64,
        reason: &[u8; 32],
        timestamp: i64,
    ) -> QRSupersessionRecord {
        let message = QRSupersessionRecord::message_bytes(old_hash, new_hash, reason, timestamp);
        let sig: Signature = self.signing_key.sign(&message);
        QRSupersessionRecord {
            old_hash,
            new_hash,
            reason: *reason,
            timestamp,
            signature: sig.to_bytes(),
        }
    }
}

/// Verify supersession record.
pub fn verify_supersession(record: &QRSupersessionRecord, vk: &VerifyingKey) -> bool {
    let message = QRSupersessionRecord::message_bytes(
        record.old_hash,
        record.new_hash,
        &record.reason,
        record.timestamp,
    );
    let sig = Signature::from_bytes(&record.signature);
    vk.verify(&message, &sig).is_ok()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::encode_codepoint;

    fn skip_if_empty() -> bool {
        ucd::table_len() == 0
    }

    /// Test signing key từ seed cố định.
    fn test_signer() -> QRSigner {
        let seed = [0x42u8; 32]; // seed cố định cho test
        QRSigner::from_seed(&seed)
    }

    // ── QR Signing ──────────────────────────────────────────────────────────

    #[test]
    fn sign_and_verify_fire() {
        if skip_if_empty() {
            return;
        }
        let signer = test_signer();
        let chain = encode_codepoint(0x1F525); // 🔥
        let record = signer.sign_qr(&chain, 1000);

        assert_eq!(record.chain, chain);
        assert_eq!(record.timestamp, 1000);
        assert!(signer.verify(&record), "Signature phải valid");
    }

    #[test]
    fn sign_and_verify_water() {
        if skip_if_empty() {
            return;
        }
        let signer = test_signer();
        let chain = encode_codepoint(0x1F4A7); // 💧
        let record = signer.sign_qr(&chain, 2000);
        assert!(signer.verify(&record));
    }

    #[test]
    fn tamper_chain_fails_verify() {
        if skip_if_empty() {
            return;
        }
        let signer = test_signer();
        let chain = encode_codepoint(0x1F525);
        let mut record = signer.sign_qr(&chain, 1000);

        // Tamper: thay chain bằng chain khác
        record.chain = encode_codepoint(0x1F4A7); // 💧 thay vì 🔥
        assert!(!signer.verify(&record), "Tampered chain phải fail verify");
    }

    #[test]
    fn tamper_timestamp_fails_verify() {
        if skip_if_empty() {
            return;
        }
        let signer = test_signer();
        let chain = encode_codepoint(0x1F525);
        let mut record = signer.sign_qr(&chain, 1000);

        // Tamper: thay timestamp
        record.timestamp = 9999;
        assert!(
            !signer.verify(&record),
            "Tampered timestamp phải fail verify"
        );
    }

    #[test]
    fn tamper_signature_fails_verify() {
        if skip_if_empty() {
            return;
        }
        let signer = test_signer();
        let chain = encode_codepoint(0x1F525);
        let mut record = signer.sign_qr(&chain, 1000);

        // Tamper: flip 1 bit trong signature
        record.signature[0] ^= 0x01;
        assert!(
            !signer.verify(&record),
            "Tampered signature phải fail verify"
        );
    }

    #[test]
    fn wrong_key_fails_verify() {
        if skip_if_empty() {
            return;
        }
        let signer1 = test_signer();
        let signer2 = QRSigner::from_seed(&[0xFF; 32]); // key khác

        let chain = encode_codepoint(0x1F525);
        let record = signer1.sign_qr(&chain, 1000);

        // Verify với key khác → fail
        assert!(
            !verify_qr(&record, &signer2.verifying_key()),
            "Sai key phải fail verify"
        );
    }

    #[test]
    fn signature_deterministic() {
        if skip_if_empty() {
            return;
        }
        // ED25519 deterministic: cùng input → cùng signature
        let signer = test_signer();
        let chain = encode_codepoint(0x1F525);
        let r1 = signer.sign_qr(&chain, 1000);
        let r2 = signer.sign_qr(&chain, 1000);
        assert_eq!(r1.signature, r2.signature, "ED25519 phải deterministic");
    }

    #[test]
    fn different_timestamps_different_signatures() {
        if skip_if_empty() {
            return;
        }
        let signer = test_signer();
        let chain = encode_codepoint(0x1F525);
        let r1 = signer.sign_qr(&chain, 1000);
        let r2 = signer.sign_qr(&chain, 2000); // khác timestamp
        assert_ne!(
            r1.signature, r2.signature,
            "Khác timestamp → khác signature"
        );
    }

    // ── QRRecord serialization ───────────────────────────────────────────────

    #[test]
    fn qr_record_roundtrip() {
        if skip_if_empty() {
            return;
        }
        let signer = test_signer();
        let chain = encode_codepoint(0x1F525);
        let record = signer.sign_qr(&chain, 1000);

        let bytes = record.to_bytes();
        let decoded = QRRecord::from_bytes(&bytes).expect("parse QRRecord");

        assert_eq!(decoded.chain, record.chain);
        assert_eq!(decoded.hash, record.hash);
        assert_eq!(decoded.timestamp, record.timestamp);
        assert_eq!(decoded.signature, record.signature);
    }

    #[test]
    fn qr_record_too_short() {
        assert!(
            QRRecord::from_bytes(&[0u8; 10]).is_none(),
            "Too short → None"
        );
    }

    // ── QR Supersession ─────────────────────────────────────────────────────

    #[test]
    fn supersede_and_verify() {
        if skip_if_empty() {
            return;
        }
        let signer = test_signer();
        let chain_old = encode_codepoint(0x25CB); // ○ (giả sử QR cũ)
        let chain_new = encode_codepoint(0x25CF); // ● (QR mới đúng hơn)

        let old_record = signer.sign_qr(&chain_old, 1000);
        let new_record = signer.sign_qr(&chain_new, 2000);

        let mut reason = [0u8; 32];
        reason[..7].copy_from_slice(b"updated");

        let supersession = signer.supersede(old_record.hash, new_record.hash, &reason, 3000);

        assert_eq!(supersession.old_hash, old_record.hash);
        assert_eq!(supersession.new_hash, new_record.hash);
        assert!(
            verify_supersession(&supersession, &signer.verifying_key()),
            "Supersession phải valid"
        );
    }

    #[test]
    fn supersession_old_still_valid() {
        if skip_if_empty() {
            return;
        }
        // QR_old vẫn valid sau khi bị supersede (QT8)
        let signer = test_signer();
        let chain_old = encode_codepoint(0x25CB);
        let chain_new = encode_codepoint(0x25CF);

        let old_record = signer.sign_qr(&chain_old, 1000);
        let new_record = signer.sign_qr(&chain_new, 2000);

        let reason = [0u8; 32];
        let _supersession = signer.supersede(old_record.hash, new_record.hash, &reason, 3000);

        // QR_old vẫn verify được (vẫn tồn tại trong sổ cái)
        assert!(
            signer.verify(&old_record),
            "QR_old vẫn valid sau supersession (QT8 — không xóa)"
        );
        assert!(signer.verify(&new_record), "QR_new phải valid");
    }

    #[test]
    fn supersession_tamper_fails() {
        if skip_if_empty() {
            return;
        }
        let signer = test_signer();
        let chain_old = encode_codepoint(0x25CB);
        let chain_new = encode_codepoint(0x25CF);

        let old_rec = signer.sign_qr(&chain_old, 1000);
        let new_rec = signer.sign_qr(&chain_new, 2000);
        let reason = [0u8; 32];

        let mut ss = signer.supersede(old_rec.hash, new_rec.hash, &reason, 3000);

        // Tamper: thay new_hash
        ss.new_hash = 0xDEAD_BEEF;
        assert!(
            !verify_supersession(&ss, &signer.verifying_key()),
            "Tampered supersession phải fail"
        );
    }

    #[test]
    fn supersession_roundtrip() {
        if skip_if_empty() {
            return;
        }
        let signer = test_signer();
        let chain_old = encode_codepoint(0x25CB);
        let chain_new = encode_codepoint(0x25CF);

        let old_rec = signer.sign_qr(&chain_old, 1000);
        let new_rec = signer.sign_qr(&chain_new, 2000);

        let mut reason = [0u8; 32];
        reason[..6].copy_from_slice(b"better");

        let ss = signer.supersede(old_rec.hash, new_rec.hash, &reason, 3000);
        let bytes = ss.to_bytes();
        let decoded = QRSupersessionRecord::from_bytes(&bytes);

        assert_eq!(decoded.old_hash, ss.old_hash);
        assert_eq!(decoded.new_hash, ss.new_hash);
        assert_eq!(decoded.reason, ss.reason);
        assert_eq!(decoded.timestamp, ss.timestamp);
        assert_eq!(decoded.signature, ss.signature);

        // Verify sau roundtrip vẫn pass
        assert!(
            verify_supersession(&decoded, &signer.verifying_key()),
            "Supersession verify sau roundtrip"
        );
    }

    // ── Chain hash integrity ─────────────────────────────────────────────────

    #[test]
    fn qr_hash_matches_chain_hash() {
        if skip_if_empty() {
            return;
        }
        let signer = test_signer();
        let chain = encode_codepoint(0x1F525);
        let record = signer.sign_qr(&chain, 1000);

        assert_eq!(
            record.hash,
            chain.chain_hash(),
            "QR hash phải khớp chain hash"
        );
    }

    #[test]
    fn sign_all_ucd_entries_sample() {
        if skip_if_empty() {
            return;
        }
        let signer = test_signer();
        let sample_cps = [
            0x1F525u32, // 🔥
            0x1F4A7,    // 💧
            0x2744,     // ❄
            0x1F9E0,    // 🧠
            0x25CF,     // ●
            0x2208,     // ∈
            0x2192,     // →
        ];

        for cp in sample_cps {
            let chain = encode_codepoint(cp);
            let record = signer.sign_qr(&chain, cp as i64);
            assert!(
                signer.verify(&record),
                "QR sign/verify phải pass cho cp=0x{:05X}",
                cp
            );
        }
    }
}
