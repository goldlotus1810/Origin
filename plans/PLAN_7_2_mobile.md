# PLAN 7.2 — Mobile: HomeOS chạy trên điện thoại

**Phụ thuộc:** 7.1 (wiring done), 4.1 (ARM64), 4.3 (WASM)
**Mục tiêu:** origin.olang chạy native trên Android (ARM64) + iOS (WASM fallback)

---

## Bối cảnh

```
Từ CLAUDE.md — Bài toán 16GB:
  500 triệu concepts = 16.5 GB → VỪA 1 CHIẾC ĐIỆN THOẠI
  HomeOS không cần GPU. Không cần cloud. Chạy local.

HIỆN TẠI:
  origin.olang = 1.35 MB ELF x86_64 (Linux only)
  ARM64 VM: 7KB binary, assembles OK, chưa test trên thiết bị thật
  WASM VM: 3KB .wasm, browser host works

SAU PLAN 7.2:
  Android: origin.olang chạy native ARM64 (Termux hoặc NDK)
  iOS: origin.olang.wasm chạy trong WKWebView
  Cả hai: < 2 MB binary, < 20 MB runtime memory
```

---

## Tasks

### 7.2.1 — Android native (~100 LOC build script)
```
Target: Android ARM64 (aarch64-linux-android)
  1. Cross-compile vm_arm64.S → ELF ARM64
  2. Builder pack → origin.olang (ARM64)
  3. Push to device: adb push origin.olang /data/local/tmp/
  4. Run: adb shell /data/local/tmp/origin.olang

Hoặc: Termux (đã có Linux userspace)
  1. Install Termux từ F-Droid
  2. pkg install binutils  (as, ld)
  3. make vm (ARM64 native trên device)
  4. make build
  5. ./origin.olang
```

### 7.2.2 — iOS WASM wrapper (~200 LOC Swift)
```
iOS app wrapper:
  1. WKWebView load origin.html + origin.olang.wasm
  2. JavaScript bridge: host_read/host_write ↔ Swift UI
  3. File I/O: WKScriptMessageHandler → sandbox documents
  4. No App Store restrictions (WASM = interpreted, not JIT)
```

### 7.2.3 — Storage layer (~150 LOC Olang)
```
Mobile storage:
  Android: /data/data/com.homeos/files/origin.olang
  iOS: Documents/origin.olang

  Knowledge file: append-only, mmap'd
  STM: in-memory, Dream cycle persists to knowledge
  Config: origin.olang.config (JSON, human-readable)
```

### 7.2.4 — Power management (~80 LOC Olang)
```
Mobile-aware behavior:
  Battery low → reduce Dream frequency
  Screen off → sleep mode (ISL only, no processing)
  Background → minimal heartbeat
  Charging → full Dream cycle + compaction
```

---

## Definition of Done

- [ ] origin.olang runs on Android (ARM64 native or Termux)
- [ ] origin.olang.wasm runs on iOS (WKWebView)
- [ ] < 2 MB binary size
- [ ] < 20 MB runtime memory
- [ ] Knowledge persists across app restarts
- [ ] Power-aware behavior (battery, screen state)

## Ước tính: 2-3 tuần
