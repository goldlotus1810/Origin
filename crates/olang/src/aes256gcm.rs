//! AES-256-GCM authenticated encryption (NIST SP 800-38D)
//!
//! HomeOS native — zero external dependencies.
//! Provides authenticated encryption with associated data (AEAD).

extern crate alloc;
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// AES-256 Core
// ─────────────────────────────────────────────────────────────────────────────

/// AES S-box (SubBytes lookup table).
const SBOX: [u8; 256] = [
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];

/// Round constants for key expansion.
const RCON: [u8; 10] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36];

/// Multiply by 2 in GF(2^8) with irreducible polynomial x^8 + x^4 + x^3 + x + 1.
fn xtime(a: u8) -> u8 {
    if a & 0x80 != 0 {
        (a << 1) ^ 0x1b
    } else {
        a << 1
    }
}

/// Multiply two bytes in GF(2^8).
#[cfg(test)]
fn gmul(mut a: u8, mut b: u8) -> u8 {
    let mut p = 0u8;
    for _ in 0..8 {
        if b & 1 != 0 {
            p ^= a;
        }
        let hi = a & 0x80;
        a <<= 1;
        if hi != 0 {
            a ^= 0x1b;
        }
        b >>= 1;
    }
    p
}

/// AES-256 expanded key: 15 round keys × 16 bytes = 240 bytes.
struct Aes256 {
    round_keys: [[u8; 16]; 15],
}

impl Aes256 {
    /// Create AES-256 cipher from 32-byte key.
    fn new(key: &[u8; 32]) -> Self {
        let round_keys = key_expansion(key);
        Self { round_keys }
    }

    /// Encrypt one 16-byte block.
    fn encrypt_block(&self, block: &mut [u8; 16]) {
        // Initial round key addition
        xor_block(block, &self.round_keys[0]);

        // Rounds 1-13
        for round in 1..14 {
            sub_bytes(block);
            shift_rows(block);
            mix_columns(block);
            xor_block(block, &self.round_keys[round]);
        }

        // Final round (no MixColumns)
        sub_bytes(block);
        shift_rows(block);
        xor_block(block, &self.round_keys[14]);
    }
}

/// AES-256 key expansion: 32-byte key → 15 × 16-byte round keys.
fn key_expansion(key: &[u8; 32]) -> [[u8; 16]; 15] {
    // Expand to 60 × 4-byte words
    let mut w = [[0u8; 4]; 60];

    // First 8 words from the key
    for i in 0..8 {
        w[i] = [key[4*i], key[4*i+1], key[4*i+2], key[4*i+3]];
    }

    for i in 8..60 {
        let mut temp = w[i - 1];
        if i % 8 == 0 {
            // RotWord + SubWord + Rcon
            temp = [
                SBOX[temp[1] as usize] ^ RCON[i / 8 - 1],
                SBOX[temp[2] as usize],
                SBOX[temp[3] as usize],
                SBOX[temp[0] as usize],
            ];
        } else if i % 8 == 4 {
            // SubWord only
            temp = [
                SBOX[temp[0] as usize],
                SBOX[temp[1] as usize],
                SBOX[temp[2] as usize],
                SBOX[temp[3] as usize],
            ];
        }
        for j in 0..4 {
            w[i][j] = w[i - 8][j] ^ temp[j];
        }
    }

    // Pack into 15 round keys
    let mut rk = [[0u8; 16]; 15];
    for i in 0..15 {
        for j in 0..4 {
            rk[i][4*j..4*j+4].copy_from_slice(&w[4*i + j]);
        }
    }
    rk
}

/// SubBytes: apply S-box to each byte.
fn sub_bytes(state: &mut [u8; 16]) {
    for b in state.iter_mut() {
        *b = SBOX[*b as usize];
    }
}

/// ShiftRows: cyclic left shift of rows.
/// State is column-major: state[row + 4*col]
fn shift_rows(state: &mut [u8; 16]) {
    // Row 1: shift left by 1
    let t = state[1];
    state[1] = state[5];
    state[5] = state[9];
    state[9] = state[13];
    state[13] = t;

    // Row 2: shift left by 2
    let t0 = state[2];
    let t1 = state[6];
    state[2] = state[10];
    state[6] = state[14];
    state[10] = t0;
    state[14] = t1;

    // Row 3: shift left by 3 (= right by 1)
    let t = state[15];
    state[15] = state[11];
    state[11] = state[7];
    state[7] = state[3];
    state[3] = t;
}

/// MixColumns: matrix multiplication in GF(2^8).
fn mix_columns(state: &mut [u8; 16]) {
    for col in 0..4 {
        let i = col * 4;
        let a0 = state[i];
        let a1 = state[i + 1];
        let a2 = state[i + 2];
        let a3 = state[i + 3];

        let x0 = xtime(a0);
        let x1 = xtime(a1);
        let x2 = xtime(a2);
        let x3 = xtime(a3);

        state[i]     = x0 ^ x1 ^ a1 ^ a2 ^ a3;
        state[i + 1] = a0 ^ x1 ^ x2 ^ a2 ^ a3;
        state[i + 2] = a0 ^ a1 ^ x2 ^ x3 ^ a3;
        state[i + 3] = x0 ^ a0 ^ a1 ^ a2 ^ x3;
    }
}

/// XOR a 16-byte block with a round key.
fn xor_block(block: &mut [u8; 16], key: &[u8; 16]) {
    for i in 0..16 {
        block[i] ^= key[i];
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GCM Mode (NIST SP 800-38D)
// ─────────────────────────────────────────────────────────────────────────────

/// Multiply two 128-bit blocks in GF(2^128) with reduction polynomial
/// x^128 + x^7 + x^2 + x + 1.
fn ghash_mul(x: &[u8; 16], y: &[u8; 16]) -> [u8; 16] {
    let mut z = [0u8; 16];
    let mut v = *y;

    for i in 0..128 {
        let byte_idx = i / 8;
        let bit_idx = 7 - (i % 8); // MSB first
        if (x[byte_idx] >> bit_idx) & 1 == 1 {
            for j in 0..16 {
                z[j] ^= v[j];
            }
        }

        // v = v >> 1 in GF(2^128) with reduction
        let lsb = v[15] & 1;
        for j in (1..16).rev() {
            v[j] = (v[j] >> 1) | (v[j - 1] << 7);
        }
        v[0] >>= 1;
        if lsb != 0 {
            v[0] ^= 0xe1; // reduction: x^128 ≡ x^7 + x^2 + x + 1
        }
    }
    z
}

/// GHASH: hash function for GCM authentication.
/// Processes data in 16-byte blocks.
fn ghash(h: &[u8; 16], data: &[u8]) -> [u8; 16] {
    let mut y = [0u8; 16];
    let mut i = 0;

    while i + 16 <= data.len() {
        for j in 0..16 {
            y[j] ^= data[i + j];
        }
        y = ghash_mul(&y, h);
        i += 16;
    }

    // Handle partial last block (pad with zeros)
    if i < data.len() {
        let remaining = data.len() - i;
        for j in 0..remaining {
            y[j] ^= data[i + j];
        }
        y = ghash_mul(&y, h);
    }

    y
}

/// Increment the counter (last 4 bytes, big-endian).
fn inc32(counter: &mut [u8; 16]) {
    for i in (12..16).rev() {
        counter[i] = counter[i].wrapping_add(1);
        if counter[i] != 0 {
            break;
        }
    }
}

/// AES-256-GCM authenticated encryption.
///
/// - `key`: 32-byte AES-256 key
/// - `nonce`: 12-byte nonce (IV)
/// - `plaintext`: data to encrypt
/// - `aad`: additional authenticated data (not encrypted, but authenticated)
///
/// Returns `ciphertext || tag` (plaintext.len() + 16 bytes).
pub fn aes256gcm_encrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    plaintext: &[u8],
    aad: &[u8],
) -> Vec<u8> {
    let cipher = Aes256::new(key);

    // H = AES(K, 0^128)
    let mut h = [0u8; 16];
    cipher.encrypt_block(&mut h);

    // J0 = nonce || 0x00000001 (for 12-byte nonce)
    let mut j0 = [0u8; 16];
    j0[..12].copy_from_slice(nonce);
    j0[15] = 1;

    // Encrypt: CTR mode starting from J0 + 1
    let mut counter = j0;
    inc32(&mut counter);

    let mut ciphertext = Vec::with_capacity(plaintext.len() + 16);
    let mut i = 0;
    while i < plaintext.len() {
        let mut block = counter;
        cipher.encrypt_block(&mut block);
        inc32(&mut counter);

        let chunk_len = core::cmp::min(16, plaintext.len() - i);
        for j in 0..chunk_len {
            ciphertext.push(plaintext[i + j] ^ block[j]);
        }
        i += chunk_len;
    }

    // Compute GHASH over AAD and ciphertext
    // S = GHASH(H, A || pad(A) || C || pad(C) || len(A)64 || len(C)64)
    let mut ghash_input = Vec::new();

    // AAD padded to 16-byte boundary
    ghash_input.extend_from_slice(aad);
    let aad_pad = (16 - (aad.len() % 16)) % 16;
    ghash_input.extend(core::iter::repeat_n(0u8,aad_pad));

    // Ciphertext padded to 16-byte boundary
    ghash_input.extend_from_slice(&ciphertext);
    let ct_pad = (16 - (ciphertext.len() % 16)) % 16;
    ghash_input.extend(core::iter::repeat_n(0u8,ct_pad));

    // Lengths in bits, big-endian u64
    let aad_bits = (aad.len() as u64) * 8;
    let ct_bits = (ciphertext.len() as u64) * 8;
    ghash_input.extend_from_slice(&aad_bits.to_be_bytes());
    ghash_input.extend_from_slice(&ct_bits.to_be_bytes());

    let s = ghash(&h, &ghash_input);

    // Tag = AES(K, J0) XOR S
    let mut tag_mask = j0;
    cipher.encrypt_block(&mut tag_mask);
    let mut tag = [0u8; 16];
    for j in 0..16 {
        tag[j] = s[j] ^ tag_mask[j];
    }

    ciphertext.extend_from_slice(&tag);
    ciphertext
}

/// AES-256-GCM authenticated decryption.
///
/// - `key`: 32-byte AES-256 key
/// - `nonce`: 12-byte nonce (IV)
/// - `ciphertext_and_tag`: ciphertext || tag (at least 16 bytes for tag)
/// - `aad`: additional authenticated data
///
/// Returns plaintext on success, `None` if authentication fails.
pub fn aes256gcm_decrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    ciphertext_and_tag: &[u8],
    aad: &[u8],
) -> Option<Vec<u8>> {
    if ciphertext_and_tag.len() < 16 {
        return None;
    }

    let ct_len = ciphertext_and_tag.len() - 16;
    let ciphertext = &ciphertext_and_tag[..ct_len];
    let received_tag = &ciphertext_and_tag[ct_len..];

    let cipher = Aes256::new(key);

    // H = AES(K, 0^128)
    let mut h = [0u8; 16];
    cipher.encrypt_block(&mut h);

    // J0 = nonce || 0x00000001
    let mut j0 = [0u8; 16];
    j0[..12].copy_from_slice(nonce);
    j0[15] = 1;

    // Compute expected tag BEFORE decryption (verify first)
    let mut ghash_input = Vec::new();

    // AAD padded
    ghash_input.extend_from_slice(aad);
    let aad_pad = (16 - (aad.len() % 16)) % 16;
    ghash_input.extend(core::iter::repeat_n(0u8,aad_pad));

    // Ciphertext padded
    ghash_input.extend_from_slice(ciphertext);
    let ct_pad = (16 - (ciphertext.len() % 16)) % 16;
    ghash_input.extend(core::iter::repeat_n(0u8,ct_pad));

    // Lengths
    let aad_bits = (aad.len() as u64) * 8;
    let ct_bits = (ciphertext.len() as u64) * 8;
    ghash_input.extend_from_slice(&aad_bits.to_be_bytes());
    ghash_input.extend_from_slice(&ct_bits.to_be_bytes());

    let s = ghash(&h, &ghash_input);

    let mut tag_mask = j0;
    cipher.encrypt_block(&mut tag_mask);
    let mut expected_tag = [0u8; 16];
    for j in 0..16 {
        expected_tag[j] = s[j] ^ tag_mask[j];
    }

    // Constant-time tag comparison
    let mut diff = 0u8;
    for j in 0..16 {
        diff |= expected_tag[j] ^ received_tag[j];
    }
    if diff != 0 {
        return None;
    }

    // Decrypt: CTR mode starting from J0 + 1
    let mut counter = j0;
    inc32(&mut counter);

    let mut plaintext = Vec::with_capacity(ct_len);
    let mut i = 0;
    while i < ct_len {
        let mut block = counter;
        cipher.encrypt_block(&mut block);
        inc32(&mut counter);

        let chunk_len = core::cmp::min(16, ct_len - i);
        for j in 0..chunk_len {
            plaintext.push(ciphertext[i + j] ^ block[j]);
        }
        i += chunk_len;
    }

    Some(plaintext)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> alloc::string::String {
        let mut s = alloc::string::String::new();
        for b in bytes {
            s.push_str(&alloc::format!("{:02x}", b));
        }
        s
    }

    fn from_hex(h: &str) -> Vec<u8> {
        let mut out = Vec::new();
        for i in (0..h.len()).step_by(2) {
            out.push(u8::from_str_radix(&h[i..i+2], 16).unwrap());
        }
        out
    }

    // NIST AES-256 test vector (FIPS 197 Appendix C.3)
    #[test]
    fn test_aes256_block() {
        let key: [u8; 32] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
            0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
            0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
        ];
        let mut block: [u8; 16] = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
        ];
        let cipher = Aes256::new(&key);
        cipher.encrypt_block(&mut block);
        assert_eq!(
            hex(&block),
            "8ea2b7ca516745bfeafc49904b496089",
            "NIST AES-256 test vector"
        );
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [0x42u8; 32];
        let nonce = [0x01u8; 12];
        let plaintext = b"Hello, HomeOS! This is a test message for AES-256-GCM.";
        let aad = b"";

        let encrypted = aes256gcm_encrypt(&key, &nonce, plaintext, aad);
        assert_eq!(encrypted.len(), plaintext.len() + 16);

        let decrypted = aes256gcm_decrypt(&key, &nonce, &encrypted, aad).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_with_aad() {
        let key = [0x42u8; 32];
        let nonce = [0x01u8; 12];
        let plaintext = b"secret data";
        let aad = b"authenticated but not encrypted";

        let encrypted = aes256gcm_encrypt(&key, &nonce, plaintext, aad);
        let decrypted = aes256gcm_decrypt(&key, &nonce, &encrypted, aad).unwrap();
        assert_eq!(decrypted, plaintext);

        // Wrong AAD fails
        let result = aes256gcm_decrypt(&key, &nonce, &encrypted, b"wrong aad");
        assert!(result.is_none(), "wrong AAD must fail");
    }

    #[test]
    fn test_tamper_detection() {
        let key = [0x42u8; 32];
        let nonce = [0x01u8; 12];
        let plaintext = b"tamper test";

        let mut encrypted = aes256gcm_encrypt(&key, &nonce, plaintext, b"");

        // Corrupt one byte
        encrypted[3] ^= 0xFF;

        let result = aes256gcm_decrypt(&key, &nonce, &encrypted, b"");
        assert!(result.is_none(), "tampered ciphertext must fail");
    }

    #[test]
    fn test_wrong_key() {
        let key1 = [0x42u8; 32];
        let key2 = [0x43u8; 32];
        let nonce = [0x01u8; 12];
        let plaintext = b"key test";

        let encrypted = aes256gcm_encrypt(&key1, &nonce, plaintext, b"");
        let result = aes256gcm_decrypt(&key2, &nonce, &encrypted, b"");
        assert!(result.is_none(), "wrong key must fail");
    }

    #[test]
    fn test_wrong_nonce() {
        let key = [0x42u8; 32];
        let nonce1 = [0x01u8; 12];
        let nonce2 = [0x02u8; 12];
        let plaintext = b"nonce test";

        let encrypted = aes256gcm_encrypt(&key, &nonce1, plaintext, b"");
        let result = aes256gcm_decrypt(&key, &nonce2, &encrypted, b"");
        assert!(result.is_none(), "wrong nonce must fail");
    }

    #[test]
    fn test_empty_plaintext() {
        let key = [0x42u8; 32];
        let nonce = [0x01u8; 12];

        let encrypted = aes256gcm_encrypt(&key, &nonce, b"", b"");
        assert_eq!(encrypted.len(), 16); // tag only

        let decrypted = aes256gcm_decrypt(&key, &nonce, &encrypted, b"").unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_deterministic() {
        let key = [0x42u8; 32];
        let nonce = [0x01u8; 12];
        let plaintext = b"deterministic";

        let enc1 = aes256gcm_encrypt(&key, &nonce, plaintext, b"");
        let enc2 = aes256gcm_encrypt(&key, &nonce, plaintext, b"");
        assert_eq!(enc1, enc2, "same key+nonce+plaintext must give same output");
    }

    #[test]
    fn test_different_nonce_different_output() {
        let key = [0x42u8; 32];
        let plaintext = b"nonce uniqueness";

        let enc1 = aes256gcm_encrypt(&key, &[0x01; 12], plaintext, b"");
        let enc2 = aes256gcm_encrypt(&key, &[0x02; 12], plaintext, b"");
        assert_ne!(enc1, enc2, "different nonces must give different ciphertext");
    }

    #[test]
    fn test_large_plaintext() {
        let key = [0x42u8; 32];
        let nonce = [0x01u8; 12];
        let plaintext = alloc::vec![0xABu8; 1000]; // multi-block

        let encrypted = aes256gcm_encrypt(&key, &nonce, &plaintext, b"");
        assert_eq!(encrypted.len(), 1016); // 1000 + 16

        let decrypted = aes256gcm_decrypt(&key, &nonce, &encrypted, b"").unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_too_short_decrypt() {
        let key = [0x42u8; 32];
        let nonce = [0x01u8; 12];

        let result = aes256gcm_decrypt(&key, &nonce, &[0u8; 10], b"");
        assert!(result.is_none(), "too short must fail");
    }

    // NIST GCM test vector (SP 800-38D, Test Case 16: AES-256, 96-bit IV)
    #[test]
    fn test_nist_gcm_vector() {
        let key = from_hex("feffe9928665731c6d6a8f9467308308feffe9928665731c6d6a8f9467308308");
        let nonce = from_hex("cafebabefacedbaddecaf888");
        let plaintext = from_hex(
            "d9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a721c3c0c95956809532fcf0e2449a6b525b16aedf5aa0de657ba637b39"
        );
        let aad = from_hex("feedfacedeadbeeffeedfacedeadbeefabaddad2");

        let mut key_arr = [0u8; 32];
        key_arr.copy_from_slice(&key);
        let mut nonce_arr = [0u8; 12];
        nonce_arr.copy_from_slice(&nonce);

        let encrypted = aes256gcm_encrypt(&key_arr, &nonce_arr, &plaintext, &aad);
        let ct_len = encrypted.len() - 16;
        let ciphertext = &encrypted[..ct_len];
        let tag = &encrypted[ct_len..];

        let expected_ct = "522dc1f099567d07f47f37a32a84427d643a8cdcbfe5c0c97598a2bd2555d1aa8cb08e48590dbb3da7b08b1056828838c5f61e6393ba7a0abcc9f662";
        let expected_tag = "76fc6ece0f4e1768cddf8853bb2d551b";

        assert_eq!(hex(ciphertext), expected_ct, "NIST GCM ciphertext");
        assert_eq!(hex(tag), expected_tag, "NIST GCM tag");

        // Verify decryption
        let decrypted = aes256gcm_decrypt(&key_arr, &nonce_arr, &encrypted, &aad).unwrap();
        assert_eq!(decrypted, plaintext, "NIST GCM decryption");
    }

    #[test]
    fn test_gmul_basic() {
        // GF(2^8) multiplication sanity checks
        assert_eq!(gmul(0x57, 0x83), 0xc1, "NIST GF(2^8) example");
    }

    #[test]
    fn test_tag_tamper() {
        let key = [0x42u8; 32];
        let nonce = [0x01u8; 12];
        let plaintext = b"tag test";

        let mut encrypted = aes256gcm_encrypt(&key, &nonce, plaintext, b"");

        // Tamper with tag (last 16 bytes)
        let len = encrypted.len();
        encrypted[len - 1] ^= 1;

        let result = aes256gcm_decrypt(&key, &nonce, &encrypted, b"");
        assert!(result.is_none(), "tampered tag must fail");
    }
}
