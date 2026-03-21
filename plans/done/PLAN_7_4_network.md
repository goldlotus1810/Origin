# PLAN 7.4 — Network: ISL over real transport

**Phụ thuộc:** 7.1 (wiring), 6.3 (reproduce)
**Mục tiêu:** ISL messages flow giữa devices thật qua TCP/UDP/BLE

---

## Bối cảnh

```
HIỆN TẠI:
  ISL = in-memory message passing (ISLQueue, ISLFrame)
  Chief ↔ Worker = same process, same memory space
  Mọi thứ là local

SAU PLAN 7.4:
  ISL over TCP: Chief trên laptop ↔ Worker trên Raspberry Pi
  ISL over BLE: Chief trên phone ↔ Worker trên ESP32
  ISL over WebSocket: Chief trên server ↔ Worker trên browser (WASM)
  AES-256-GCM encryption (đã spec trong isl crate)
```

---

## Tasks

### 7.4.1 — ISL TCP transport (~200 LOC Olang)
```
isl_tcp.ol:
  listen(port) → accept connections → ISLFrame recv/send
  connect(host, port) → ISLFrame send/recv
  Frame format: [length:4][encrypted_payload:N]
  AES-256-GCM: key derived from master_key + ISL addresses
```

### 7.4.2 — ISL WebSocket transport (~150 LOC Olang)
```
isl_ws.ol:
  For WASM Workers in browser
  WebSocket → ISLFrame serialization
  origin.html: WebSocket client → connect to Chief
```

### 7.4.3 — ISL BLE transport (~100 LOC Olang)
```
isl_ble.ol:
  For IoT Workers (ESP32, nRF52)
  BLE GATT service: 1 characteristic per ISL channel
  MTU-aware fragmentation (BLE MTU = 23-512 bytes)
  ISLFrame fits in 1-2 BLE packets (12B header + body)
```

### 7.4.4 — Discovery protocol (~100 LOC Olang)
```
isl_discovery.ol:
  mDNS: _homeos._tcp.local → find Chief on LAN
  BLE scan: find Workers advertising HomeOS service UUID
  Auto-connect: Worker finds Chief → handshake → ISL ready
```

---

## Definition of Done

- [ ] ISL TCP: Chief ↔ Worker across network
- [ ] ISL WebSocket: browser Worker connects to Chief
- [ ] Encrypted transport (AES-256-GCM)
- [ ] Discovery: auto-find Chief on LAN
- [ ] Test: 2 processes, different ports, ISL message roundtrip

## Ước tính: 1-2 tuần
