// stdlib/homeos/reproduce.ol — Worker clone reproduction
// PLAN 6.3: origin.olang sinh bản sao nhỏ cho IoT Worker devices.
// Worker = VM (target arch) + minimal skills + config. No knowledge.
// Size target: 50-100 KB per worker clone.

// ── Worker kinds ────────────────────────────────────────────────────

pub fn kind_camera()  { return 0x01; }
pub fn kind_light()   { return 0x02; }
pub fn kind_door()    { return 0x03; }
pub fn kind_sensor()  { return 0x04; }
pub fn kind_network() { return 0x05; }

pub fn kind_name(kind) {
  if kind == 0x01 { return "camera"; }
  if kind == 0x02 { return "light"; }
  if kind == 0x03 { return "door"; }
  if kind == 0x04 { return "sensor"; }
  if kind == 0x05 { return "network"; }
  return "unknown";
}

// ── Skill packs per worker kind ─────────────────────────────────────

pub fn select_skills(kind) {
  if kind == 0x01 { return ["worker", "silk_ops", "gate"]; }       // camera
  if kind == 0x02 { return ["worker"]; }                            // light
  if kind == 0x03 { return ["worker", "gate"]; }                    // door
  if kind == 0x04 { return ["worker"]; }                            // sensor
  if kind == 0x05 { return ["worker", "gate"]; }                    // network
  return ["worker"];  // minimal
}

// ── Spawn: create worker binary ─────────────────────────────────────

pub fn spawn(config) {
  // config = { kind, arch, chief_addr, output, permissions }
  emit "Spawning worker: ";
  emit kind_name(config.kind);
  emit " (";
  emit config.arch;
  emit ")\n";

  // 1. Select skills for worker kind
  let skills = select_skills(config.kind);

  // 2. Compile skill bytecode
  let bytecode = compile_skills(skills);
  if len(bytecode) == 0 {
    emit "Error: no skills compiled\n";
    return false;
  }

  // 3. Generate worker config section
  let worker_cfg = make_worker_config(config);

  // 4. Build origin header
  // Layout: [header 32B][bytecode][config]
  let bc_offset = 32;
  let bc_size = len(bytecode);
  let kn_offset = bc_offset + bc_size;
  let kn_size = len(worker_cfg);

  let header = make_header(bc_offset, bc_size, kn_offset, kn_size,
                           config.arch, 0x0001);  // flag: worker clone

  // 5. Assemble binary
  let binary = [];
  append_bytes(binary, header);
  append_bytes(binary, bytecode);
  append_bytes(binary, worker_cfg);

  // 6. Write output
  __file_write_bytes(config.output, binary);

  emit "Worker spawned: ";
  emit config.output;
  emit " (";
  emit len(binary);
  emit " bytes, ";
  emit len(skills);
  emit " skills)\n";
  return true;
}

fn compile_skills(skills) {
  // Compile each skill .ol file to bytecode
  let bc = [];
  let i = 0;
  while i < len(skills) {
    let path = "stdlib/homeos/" + skills[i] + ".ol";
    let src = __file_read(path);
    if len(src) > 0 {
      let compiled = __compile(src);
      append_bytes(bc, compiled);
    }
    i = i + 1;
  }
  // Append Halt
  push(bc, 0x0F);
  return bc;
}

// ── Worker config format ────────────────────────────────────────────
// [WKPK:4][version:1][isl_addr:4][chief_addr:4][kind:1]
// [permissions:2][created_ts:8] = 24 bytes

fn make_worker_config(config) {
  let buf = [];
  // Magic: "WKPK"
  push(buf, 87); push(buf, 75); push(buf, 80); push(buf, 75);
  // Version
  push(buf, 0x01);
  // ISL address (4 bytes)
  push_u32(buf, config.isl_addr);
  // Chief address (4 bytes)
  push_u32(buf, config.chief_addr);
  // Worker kind (1 byte)
  push(buf, config.kind);
  // Permissions (2 bytes)
  push_u16(buf, config.permissions);
  // Created timestamp (8 bytes)
  push_u64(buf, time());
  return buf;
}

fn make_header(bc_offset, bc_size, kn_offset, kn_size, arch, flags) {
  let h = [];
  // Magic: ○LNG = [0xE2, 0x97, 0x8B, 0x4C]
  push(h, 226); push(h, 151); push(h, 139); push(h, 76);
  // Version
  push(h, 0x10);
  // Arch
  let arch_byte = 0x01;  // default x86_64
  if arch == "arm64" { arch_byte = 0x02; }
  if arch == "wasm" { arch_byte = 0x03; }
  push(h, arch_byte);
  // VM offset + size (0,0 for worker clones — no embedded VM)
  push_u32(h, 0);
  push_u32(h, 0);
  // Bytecode offset + size
  push_u32(h, bc_offset);
  push_u32(h, bc_size);
  // Knowledge offset + size
  push_u32(h, kn_offset);
  push_u32(h, kn_size);
  // Flags
  push_u16(h, flags);
  return h;
}

// ── ISL address allocation ──────────────────────────────────────────

pub fn allocate_isl_address(layer, group, subgroup) {
  // Workers = layer 2
  // Auto-assign: simple counter-based (real impl scans network)
  // Use time-based seed for uniqueness
  let t = time();
  let index = t - floor(t / 255) * 255 + 1;  // 1-255
  return layer * 16777216 + group * 65536 + subgroup * 256 + index;
}

// ── Byte helpers ────────────────────────────────────────────────────

fn append_bytes(dest, src) {
  let i = 0;
  while i < len(src) {
    push(dest, src[i]);
    i = i + 1;
  }
}

fn push_u16(buf, val) {
  push(buf, val - floor(val / 256) * 256);
  push(buf, floor(val / 256));
}

fn push_u32(buf, val) {
  let i = 0;
  while i < 4 {
    push(buf, val - floor(val / 256) * 256);
    val = floor(val / 256);
    i = i + 1;
  }
}

fn push_u64(buf, val) {
  let i = 0;
  while i < 8 {
    push(buf, val - floor(val / 256) * 256);
    val = floor(val / 256);
    i = i + 1;
  }
}
