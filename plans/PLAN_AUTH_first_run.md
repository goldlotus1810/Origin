# PLAN AUTH — First-run setup: Terms + Master Key + Biometric

**Phụ thuộc:** Không — có thể làm song song với Phase 0
**Mục tiêu:** Khi `./origin.olang` chạy lần đầu → hiện Terms → tạo Master Key → lock ISL chain.
**Yêu cầu:** Biết Rust. Hiểu Ed25519, Argon2id, AES-256-GCM.

---

## Bối cảnh

Khi origin.olang chạy lần đầu (chưa có master key trong header):
1. Hiện Quy tắc sử dụng → người dùng Đồng ý
2. Nhập username + password → derive Ed25519 keypair
3. Ghi public key vào origin.olang header → ISL chain locked
4. (Tuỳ chọn) Thêm sinh trắc

Sau đó mỗi lần chạy: nhập password (hoặc biometric) → derive key → verify match.

---

## Crate: `crates/runtime/src/auth/`

### File structure

```
crates/runtime/src/auth/
├── mod.rs           ← pub mod, AuthState enum
├── setup.rs         ← First-run wizard
├── key.rs           ← Argon2id + Ed25519 key derivation
├── verify.rs        ← Login verification
├── biometric.rs     ← Biometric layer (encrypt/decrypt seed)
├── export.rs        ← key.ol export/import
└── terms.rs         ← Terms text + hash
```

### Thêm dependency vào Cargo.toml

```toml
# crates/runtime/Cargo.toml
[dependencies]
argon2 = "0.5"           # Argon2id key derivation
ed25519-dalek = "2"       # Ed25519 signing
aes-gcm = "0.10"          # AES-256-GCM encryption
rand = "0.8"              # Random salt generation
```

**Lưu ý:** Sau này (Phase 3) sẽ thay bằng Olang implementation. Giờ dùng Rust crate.

---

## Việc cần làm

### Task 1: AuthState + Header extension (mod.rs, ~80 LOC)

```rust
/// Trạng thái xác thực
pub enum AuthState {
    /// Chưa setup — cần first-run wizard
    Virgin,
    /// Đã setup, chưa unlock — cần password/biometric
    Locked,
    /// Đã unlock — sẵn sàng dùng
    Unlocked {
        signing_key: ed25519_dalek::SigningKey,
    },
}

/// Header extension (append sau 32 bytes gốc)
#[repr(C, packed)]
pub struct AuthHeader {
    pub master_pubkey: [u8; 32],     // Ed25519 public key
    pub salt: [u8; 16],              // Argon2id salt
    pub bio_encrypted_seed: [u8; 48],// AES-256-GCM(seed) — 32 seed + 16 tag
    pub bio_method: u8,              // 0=none, 1=fingerprint, 2=face, 3=voice
    pub setup_ts: i64,               // timestamp
    pub terms_hash: u64,             // hash of accepted terms
}
// Total: 32 + 16 + 48 + 1 + 8 + 8 = 113 bytes

impl AuthHeader {
    pub fn is_virgin(&self) -> bool {
        self.master_pubkey == [0u8; 32]
    }
}
```

### Task 2: Key derivation (key.rs, ~100 LOC)

```rust
use argon2::{Argon2, Algorithm, Version, Params};
use ed25519_dalek::{SigningKey, VerifyingKey};

/// Derive Ed25519 keypair từ username + password
pub fn derive_keypair(username: &str, password: &str) -> (SigningKey, VerifyingKey) {
    // 1. Salt = SHA-256(username)
    let salt = sha256(username.as_bytes());

    // 2. Argon2id: password → 32 bytes seed
    let params = Params::new(
        65536,    // 64 MB memory
        3,        // 3 iterations
        1,        // 1 thread (deterministic)
        Some(32), // 32 bytes output
    ).unwrap();

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut seed = [0u8; 32];
    argon2.hash_password_into(password.as_bytes(), &salt[..16], &mut seed).unwrap();

    // 3. Seed → Ed25519 keypair
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();

    (signing_key, verifying_key)
}

/// Verify password → same public key
pub fn verify_password(username: &str, password: &str, expected_pubkey: &[u8; 32]) -> bool {
    let (_, verifying_key) = derive_keypair(username, password);
    verifying_key.as_bytes() == expected_pubkey
}
```

### Task 3: First-run wizard (setup.rs, ~150 LOC)

```rust
use std::io::{self, Write};

pub fn run_first_setup() -> Result<AuthHeader, AuthError> {
    // 1. Hiện Terms
    println!("{}", terms::TERMS_TEXT);
    print!("\n[Đồng ý & Tiếp tục] (y/n): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() != "y" {
        return Err(AuthError::TermsRejected);
    }

    // 2. Nhập credentials
    print!("Tên người dùng: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim();

    let password = prompt_password("Mật khẩu: ")?;
    let confirm = prompt_password("Xác nhận: ")?;

    if password != confirm {
        return Err(AuthError::PasswordMismatch);
    }

    // 3. Derive keypair
    let (signing_key, verifying_key) = key::derive_keypair(username, &password);

    // 4. Build AuthHeader
    let mut salt = [0u8; 16];
    rand::fill(&mut salt);

    let header = AuthHeader {
        master_pubkey: *verifying_key.as_bytes(),
        salt,
        bio_encrypted_seed: [0u8; 48], // no biometric yet
        bio_method: 0,
        setup_ts: timestamp_now(),
        terms_hash: hash64(terms::TERMS_TEXT.as_bytes()),
    };

    // 5. (Optional) Biometric
    println!("\nThêm xác thực sinh trắc? (y/n): ");
    // ... (skip for v1)

    println!("\n○ Master Key created.");
    println!("○ ISL chain locked: AAM → {:?}", &header.master_pubkey[..8]);

    Ok(header)
}
```

### Task 4: ISL message signing (verify.rs, ~80 LOC)

```rust
use ed25519_dalek::{Signature, Signer, Verifier};

/// Ký ISL message (AAM tier-0 commands)
pub fn sign_isl_message(key: &SigningKey, message: &[u8]) -> Signature {
    key.sign(message)
}

/// Verify ISL message signature
pub fn verify_isl_signature(
    pubkey: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) -> bool {
    let Ok(verifying_key) = VerifyingKey::from_bytes(pubkey) else {
        return false;
    };
    let Ok(sig) = Signature::from_bytes(signature) else {
        return false;
    };
    verifying_key.verify(message, &sig).is_ok()
}
```

### Task 5: key.ol export/import (export.rs, ~120 LOC)

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::Aead;

/// Export master key → encrypted key.ol file
pub fn export_key(
    username: &str,
    password: &str,
    pubkey: &[u8; 32],
    origin_id: u64,
) -> Vec<u8> {
    let (signing_key, _) = key::derive_keypair(username, password);
    let seed = signing_key.to_bytes();

    // Encrypt seed with password-derived AES key
    let aes_key = derive_aes_key(password);
    let cipher = Aes256Gcm::new(&aes_key);
    let nonce = Nonce::from_slice(&[0u8; 12]); // deterministic for same password
    let encrypted = cipher.encrypt(nonce, seed.as_ref()).unwrap();

    // key.ol format:
    let mut output = Vec::new();
    output.extend_from_slice(&encrypted);        // 48 bytes (32 + 16 tag)
    output.extend_from_slice(pubkey);             // 32 bytes
    output.extend_from_slice(&hash64(username.as_bytes()).to_le_bytes()); // 8 bytes
    output.extend_from_slice(&timestamp_now().to_le_bytes());            // 8 bytes
    output.extend_from_slice(&origin_id.to_le_bytes());                  // 8 bytes
    // Total: 104 bytes
    output
}

/// Import key.ol → verify + restore
pub fn import_key(key_data: &[u8], password: &str) -> Result<AuthHeader, AuthError> {
    if key_data.len() != 104 {
        return Err(AuthError::InvalidKeyFile);
    }

    let encrypted_seed = &key_data[0..48];
    let expected_pubkey: [u8; 32] = key_data[48..80].try_into().unwrap();

    // Decrypt
    let aes_key = derive_aes_key(password);
    let cipher = Aes256Gcm::new(&aes_key);
    let nonce = Nonce::from_slice(&[0u8; 12]);
    let seed = cipher.decrypt(nonce, encrypted_seed)
        .map_err(|_| AuthError::WrongPassword)?;

    // Verify pubkey matches
    let signing_key = SigningKey::from_bytes(&seed.try_into().unwrap());
    if signing_key.verifying_key().as_bytes() != &expected_pubkey {
        return Err(AuthError::KeyMismatch);
    }

    Ok(AuthHeader {
        master_pubkey: expected_pubkey,
        salt: [0u8; 16], // will be regenerated
        bio_encrypted_seed: [0u8; 48],
        bio_method: 0,
        setup_ts: timestamp_now(),
        terms_hash: 0,
    })
}
```

### Task 6: Wire into HomeRuntime (origin.rs)

```rust
// crates/runtime/src/core/origin.rs

impl HomeRuntime {
    pub fn new() -> Self {
        // ... existing init ...

        // Check auth state
        let auth_header = read_auth_header(&origin_path);
        let auth_state = if auth_header.is_virgin() {
            // First run → setup wizard
            let header = auth::setup::run_first_setup().unwrap();
            write_auth_header(&origin_path, &header);
            AuthState::Locked // require login after setup
        } else {
            AuthState::Locked
        };

        // ... rest of init ...
    }

    pub fn unlock(&mut self, password: &str) -> bool {
        let header = self.auth_header();
        // derive keypair, verify match
        if key::verify_password(&self.username, password, &header.master_pubkey) {
            let (signing_key, _) = key::derive_keypair(&self.username, password);
            self.auth_state = AuthState::Unlocked { signing_key };
            true
        } else {
            false
        }
    }
}
```

---

## Test plan

```rust
#[test]
fn test_key_derivation_deterministic() {
    let (_, pk1) = derive_keypair("user", "pass123");
    let (_, pk2) = derive_keypair("user", "pass123");
    assert_eq!(pk1, pk2, "same input → same key");
}

#[test]
fn test_key_derivation_different_users() {
    let (_, pk1) = derive_keypair("alice", "pass123");
    let (_, pk2) = derive_keypair("bob", "pass123");
    assert_ne!(pk1, pk2, "different username → different key");
}

#[test]
fn test_verify_password() {
    let (_, pk) = derive_keypair("user", "correct");
    assert!(verify_password("user", "correct", pk.as_bytes()));
    assert!(!verify_password("user", "wrong", pk.as_bytes()));
}

#[test]
fn test_export_import_roundtrip() {
    let (_, pk) = derive_keypair("user", "pass123");
    let exported = export_key("user", "pass123", pk.as_bytes(), 42);
    let imported = import_key(&exported, "pass123").unwrap();
    assert_eq!(imported.master_pubkey, *pk.as_bytes());
}

#[test]
fn test_import_wrong_password() {
    let (_, pk) = derive_keypair("user", "pass123");
    let exported = export_key("user", "pass123", pk.as_bytes(), 42);
    assert!(import_key(&exported, "wrong").is_err());
}

#[test]
fn test_isl_sign_verify() {
    let (sk, pk) = derive_keypair("user", "pass123");
    let msg = b"AAM approve worker_light_0";
    let sig = sign_isl_message(&sk, msg);
    assert!(verify_isl_signature(pk.as_bytes(), msg, &sig.to_bytes()));
}
```

---

## Definition of Done

- [ ] First-run wizard hiện terms → nhập username/password → tạo key
- [ ] Key derivation deterministic (same input → same key)
- [ ] Public key ghi vào origin.olang header
- [ ] Login verify: đúng password → Unlocked, sai → Locked
- [ ] ISL message signing + verification
- [ ] key.ol export/import round-trip
- [ ] Wrong password → error (không decrypt được)
- [ ] All tests pass

## Ước tính: 1 tuần

---

*Tham chiếu: PLAN_REWRITE.md § Cài đặt lần đầu*
