# PLAN 6.3 — Reproduce (Worker Clones)

**Phụ thuộc:** 4.1 (cross-compile), 6.1 (self-update)
**Mục tiêu:** origin.olang sinh bản sao nhỏ cho Worker devices
**Tham chiếu:** `olang/src/clone.rs`, `SPEC_ORIGIN_REPRODUCTION.md`

---

## Bối cảnh

```
HIỆN TẠI:
  clone.rs (Rust) tạo WorkerPackage binary
  WorkerPackage = [magic][isl_addr][chief_addr][worker_kind][olang_bytes]
  NHƯNG: dùng Rust, chưa self-contained

SAU PLAN 6.3:
  origin.olang sinh bản sao nhỏ hơn cho IoT devices
  Worker clone = VM (target arch) + minimal bytecode + device skills
  Không cần knowledge (Worker không học, chỉ thực thi)
  Size: ~50-100 KB (vs ~600 KB full origin.olang)
```

---

## Thiết kế

### Worker clone anatomy

```
worker_clone.olang:

┌───────────────────────────────────────────────┐
│ HEADER (32 bytes)                              │
│  [○LNG][0x10][arch][vm_off][vm_sz][bc_off]... │
├───────────────────────────────────────────────┤
│ VM section (~50-80 KB)                         │
│  Same VM as parent, target architecture        │
├───────────────────────────────────────────────┤
│ BYTECODE section (~10-30 KB)                   │
│  Minimal subset:                               │
│  - ISL protocol (send/receive/address)         │
│  - Device skill (sensor/actuator/camera)       │
│  - Security gate (minimal)                     │
│  - Worker behavior (report to Chief)           │
│  NO: compiler, stdlib full, emotion, dream     │
├───────────────────────────────────────────────┤
│ CONFIG section (~100 bytes)                    │
│  - ISL address (4 bytes)                       │
│  - Chief address (4 bytes)                     │
│  - Worker kind (1 byte)                        │
│  - Master pubkey (32 bytes)                    │
│  - Permissions bitmask (2 bytes)               │
│  - Created timestamp (8 bytes)                 │
└───────────────────────────────────────────────┘

Total: ~60-110 KB depending on skills
```

### Worker kinds & skill packs

```
Kind         Skills                           Bytecode
──────────────────────────────────────────────────────
camera       isl + inverse_render + sensor     ~30 KB
light        isl + actuator                    ~15 KB
door         isl + actuator + security         ~20 KB
sensor       isl + sensor                      ~15 KB
network      isl + network + immunity          ~25 KB
```

### Reproduction workflow

```
o spawn camera --arch arm64 --chief 01.01.01.00 --output worker_cam.olang

1. Detect target arch → select VM binary (vm_arm64.bin hoặc asm_emit_arm64.ol)
2. Select skill pack cho worker kind
3. Compile skill bytecode (minimal subset)
4. Generate ISL address (unique in network)
5. Copy master_pubkey từ parent (auth chain)
6. Pack: header + VM + bytecode + config
7. Sign with master key
8. Write output file
9. (Optional) scp to target device
```

---

## Tasks

### 6.3.1 — reproduce.ol (~200 LOC)

```
reproduce.ol:

pub fn spawn(config) {
  emit("Spawning worker: " + config.kind + " for " + config.arch + "\n");

  // 1. Get VM for target arch
  let vm_code = get_vm_binary(config.arch);

  // 2. Select + compile skills
  let skills = select_skills(config.kind);
  let bytecode = compile_skills(skills);

  // 3. Generate config section
  let worker_config = make_worker_config(
    config.isl_address,
    config.chief_address,
    config.kind,
    config.permissions
  );

  // 4. Pack
  let origin_hdr = make_origin_header(
    152, len(vm_code),
    152 + len(vm_code), len(bytecode),
    152 + len(vm_code) + len(bytecode), len(worker_config),
    0x0001  // flag: worker clone
  );

  let payload = [];
  concat_bytes(payload, origin_hdr);
  concat_bytes(payload, vm_code);
  concat_bytes(payload, bytecode);
  concat_bytes(payload, worker_config);

  // 5. Wrap in ELF
  let binary = make_elf(payload, 32, config.arch);

  // 6. Sign
  let signed = sign_binary(binary, __master_key());

  // 7. Write
  file_write_bytes(config.output, signed);
  emit("Worker spawned: " + config.output + " (" + to_string(len(signed)) + " bytes)\n");
}

fn select_skills(kind) {
  if kind == "camera"  { return ["isl", "sensor", "inverse_render"]; }
  if kind == "light"   { return ["isl", "actuator"]; }
  if kind == "door"    { return ["isl", "actuator", "security"]; }
  if kind == "sensor"  { return ["isl", "sensor"]; }
  if kind == "network" { return ["isl", "network", "immunity"]; }
  return ["isl"];  // minimal
}

fn compile_skills(skills) {
  let bc = [];
  for skill in skills {
    let src = file_read_string("stdlib/homeos/" + skill + ".ol");
    concat_bytes(bc, compile_source(src));
  }
  return bc;
}
```

### 6.3.2 — Worker config format (~50 LOC)

```
worker_config.ol:

pub fn make_worker_config(isl_addr, chief_addr, kind, perms) {
  let buf = [];
  // Magic
  push_bytes(buf, [0x57, 0x4B, 0x50, 0x4B]);  // "WKPK"
  // Version
  push(buf, 0x01);
  // ISL address (4 bytes)
  push_u32(buf, isl_addr);
  // Chief address (4 bytes)
  push_u32(buf, chief_addr);
  // Worker kind (1 byte)
  push(buf, encode_kind(kind));
  // Master pubkey (32 bytes) — copied from parent
  let pubkey = __master_pubkey();
  concat_bytes(buf, pubkey);
  // Permissions (2 bytes)
  push_u16(buf, perms);
  // Created timestamp (8 bytes)
  push_u64(buf, __time_secs());
  return buf;
}

fn encode_kind(kind) {
  if kind == "camera"  { return 0x01; }
  if kind == "light"   { return 0x02; }
  if kind == "door"    { return 0x03; }
  if kind == "sensor"  { return 0x04; }
  if kind == "network" { return 0x05; }
  return 0x00;
}
```

### 6.3.3 — ISL address allocation (~50 LOC)

```
isl_address.ol:

// ISL address = [layer:1][group:1][subgroup:1][index:1]
// Workers = layer 2
// Auto-assign: scan existing workers → find free index

pub fn allocate_address(layer, group, subgroup) {
  let existing = __isl_list_addresses(layer, group, subgroup);
  let used = {};
  for addr in existing {
    used[addr.index] = true;
  }
  let i = 1;
  while i < 256 {
    if not used[i] {
      return make_address(layer, group, subgroup, i);
    }
    i = i + 1;
  }
  return null;  // full (255 workers per subgroup)
}
```

### 6.3.4 — Deploy helper (optional)

```
deploy.ol:

pub fn deploy(worker_path, target_host) {
  // SCP or ISL-based transfer
  // 1. Connect to target via SSH/ISL
  // 2. Transfer binary
  // 3. chmod +x
  // 4. Start worker
  // 5. Verify: ISL handshake with Chief
}
```

---

## Rào cản

```
1. Cross-arch VM binary availability
   → Cần: vm_arm64.bin pre-assembled
   → Hoặc: dùng asm_emit_arm64.ol (Phase 4.1) để generate at runtime
   → Store pre-assembled VMs trong origin.olang data section

2. Worker security
   → Worker clone PHẢI chứa master_pubkey
   → Mọi ISL message từ Chief phải verify signature
   → Worker KHÔNG có private key → không thể giả mạo Chief
   → Worker permissions: bitmask trong config

3. Minimal bytecode selection
   → Quá ít skills → Worker vô dụng
   → Quá nhiều skills → Worker quá lớn
   → Default: ISL + 1 primary skill per kind
   → Custom: "o spawn camera --skills isl,sensor,actuator"

4. ISL address conflict
   → allocate_address() cần scan network
   → Fallback: random address + collision detection
```

---

## Test Plan

```
Test 1: spawn light worker → verify binary < 100 KB
Test 2: spawn camera worker → verify all skills present in bytecode
Test 3: Worker config → parse → verify ISL addr + chief addr + kind
Test 4: Worker signature → verify with master pubkey
Test 5: Deploy (nếu có target) → ISL handshake → Chief receives report
Test 6: ISL address allocation → verify unique addresses
```

---

## Definition of Done

- [ ] `o spawn <kind>` → tạo worker binary
- [ ] Worker config format (ISL addr, chief, kind, pubkey)
- [ ] Skill selection per worker kind
- [ ] Cross-arch support (x86_64, ARM64)
- [ ] Signature (Ed25519) on worker binary
- [ ] ISL address allocation (auto-assign)
- [ ] Test: spawn + verify binary structure

## Ước tính: 1-2 tuần
