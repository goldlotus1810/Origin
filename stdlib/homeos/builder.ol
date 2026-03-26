// homeos/builder.ol — Self-sufficient builder
// Replaces Rust builder: compile .ol → bytecode, pack with VM → origin.olang
//
// Usage: origin.olang runs this to build a new origin.olang
//   o run builder.ol --stdlib stdlib/ --output origin_new.olang
//   o run builder.ol --stdlib stdlib/ --arch arm64 --output origin_arm64.olang

pub fn build(config) {
  let arch = config.arch;
  emit("Builder — origin.olang packer (Olang) [" + arch + "]\n");

  // 1. Compile all .ol → bytecode (arch-independent)
  let bytecode = [];
  if config.stdlib_path != "" {
    emit("  Compiling stdlib: " + config.stdlib_path + "\n");
    bytecode = compile_all(config.stdlib_path);
  }
  emit("  Bytecode: " + to_string(len(bytecode)) + " bytes\n");

  // WASM/WASI arch: embed bytecode into WASM binary
  if arch == "wasm" || arch == "wasi" {
    return build_wasm(config, bytecode);
  }

  // 2. Read VM code (pre-assembled binary for target arch)
  let vm_code = [];
  if config.vm_path != "" {
    emit("  Reading VM: " + config.vm_path + "\n");
    vm_code = file_read_bytes(config.vm_path);
  }
  emit("  VM code: " + to_string(len(vm_code)) + " bytes\n");

  // 3. Read knowledge
  let knowledge = [];
  if config.kn_path != "" {
    knowledge = file_read_bytes(config.kn_path);
  }
  emit("  Knowledge: " + to_string(len(knowledge)) + " bytes\n");

  // 4. Pack
  let origin_hdr = make_origin_header_arch(
    152,                          // vm_offset (after ELF 120 + origin 32)
    len(vm_code),
    152 + len(vm_code),           // bc_offset
    len(bytecode),
    152 + len(vm_code) + len(bytecode),  // kn_offset
    len(knowledge),
    0,                            // flags
    arch
  );

  // Concat all sections
  let payload = [];
  concat_bytes(payload, origin_hdr);
  concat_bytes(payload, vm_code);
  concat_bytes(payload, bytecode);
  concat_bytes(payload, knowledge);

  // Wrap in ELF for target arch
  let binary = make_elf_arch(payload, 32, arch);

  // 5. Write output
  file_write_bytes(config.output, binary);
  emit("  Output: " + config.output + " (" + to_string(len(binary)) + " bytes)\n");
  emit("Done!\n");
}

fn build_wasm(config, bytecode) {
  // Read pre-compiled WASM VM binary
  let vm_wasm = [];
  if config.vm_path != "" {
    emit("  Reading WASM VM: " + config.vm_path + "\n");
    vm_wasm = file_read_bytes(config.vm_path);
  }
  emit("  WASM VM: " + to_string(len(vm_wasm)) + " bytes\n");

  // Embed bytecode into WASM
  let binary = make_wasm_with_bytecode(vm_wasm, bytecode);
  emit("  WASM + bytecode: " + to_string(len(binary)) + " bytes\n");

  // Write output
  file_write_bytes(config.output, binary);
  emit("  Output: " + config.output + " (" + to_string(len(binary)) + " bytes)\n");
  emit("Done!\n");
}

fn compile_all(stdlib_path) {
  // Compile each .ol file in directory
  // Uses Olang compiler pipeline: parse → lower → encode_bytecode
  let all_bc = [];

  // Bootstrap first
  let bootstrap = stdlib_path + "/bootstrap";
  compile_dir(bootstrap, all_bc);

  // Then stdlib root
  compile_dir(stdlib_path, all_bc);

  // Then homeos
  let homeos = stdlib_path + "/homeos";
  compile_dir(homeos, all_bc);

  return all_bc;
}

fn compile_dir(dir, output) {
  // List .ol files, compile each
  let files = list_ol_files(dir);
  let i = 0;
  while i < len(files) {
    emit("    " + files[i] + "\n");
    let src = file_read_string(files[i]);
    let bc = compile_source(src);
    concat_bytes(output, bc);
    i = i + 1;
  }
}

fn compile_source(src) {
  // parse → lower → encode_bytecode
  // These are builtin calls to the Olang compiler pipeline
  let stmts = __parse(src);
  let program = __lower(stmts);
  return __encode_bytecode(program.ops);
}

fn list_ol_files(dir) {
  // VM builtin: list files in directory matching *.ol
  return __list_files(dir, ".ol");
}

fn file_read_bytes(path) {
  return __file_read(path);
}

fn file_read_string(path) {
  return __bytes_to_str(__file_read(path));
}

fn file_write_bytes(path, data) {
  __file_write(path, data);
}

fn concat_bytes(dst, src) {
  let i = 0;
  while i < len(src) {
    push(dst, src[i]);
    i = i + 1;
  }
}

// ── Default config ──

pub fn default_config() {
  return {
    vm_path: "vm/x86_64/vm_x86_64.bin",
    stdlib_path: "stdlib",
    kn_path: "origin.olang",
    output: "origin_new.olang",
    arch: "x86_64"
  };
}

pub fn arm64_config() {
  return {
    vm_path: "vm/arm64/vm_arm64.bin",
    stdlib_path: "stdlib",
    kn_path: "origin.olang",
    output: "origin_arm64.olang",
    arch: "arm64"
  };
}

pub fn wasm_config() {
  return {
    vm_path: "vm/wasm/vm_wasm.wasm",
    stdlib_path: "stdlib",
    kn_path: "",
    output: "origin.wasm",
    arch: "wasm"
  };
}

pub fn wasi_config() {
  return {
    vm_path: "vm/wasm/vm_wasi.wasm",
    stdlib_path: "stdlib",
    kn_path: "",
    output: "origin_wasi.wasm",
    arch: "wasi"
  };
}

// ── Fat binary config ──

pub fn fat_config() {
  return {
    archs: [
      { name: "x86_64", vm_path: "vm/x86_64/vm_x86_64.bin", arch_id: 1, entry_off: 0 },
      { name: "arm64",  vm_path: "vm/arm64/vm_arm64.bin",    arch_id: 2, entry_off: 0 }
    ],
    stdlib_path: "stdlib",
    kn_path: "origin.olang",
    output: "origin.fat",
    stub_x86: "o_x86",
    stub_arm: "o_arm"
  };
}

// ── Fat binary builder ──
// Packs multiple arch VMs + shared bytecode + knowledge into 1 file

pub fn build_fat(config) {
  emit("Builder — fat binary packer (multi-arch)\n");

  // 1. Compile bytecode (shared, arch-independent)
  let bytecode = [];
  if config.stdlib_path != "" {
    emit("  Compiling stdlib: " + config.stdlib_path + "\n");
    bytecode = compile_all(config.stdlib_path);
  }
  emit("  Bytecode: " + to_string(len(bytecode)) + " bytes (shared)\n");

  // 2. Read VM binaries for each arch
  let vm_codes = [];
  let i = 0;
  while i < len(config.archs) {
    let arch = config.archs[i];
    emit("  Reading VM [" + arch.name + "]: " + arch.vm_path + "\n");
    let vm = file_read_bytes(arch.vm_path);
    push(vm_codes, vm);
    emit("    VM size: " + to_string(len(vm)) + " bytes\n");
    i = i + 1;
  }

  // 3. Read knowledge
  let knowledge = [];
  if config.kn_path != "" {
    knowledge = file_read_bytes(config.kn_path);
  }
  emit("  Knowledge: " + to_string(len(knowledge)) + " bytes (shared)\n");

  // 4. Calculate offsets
  // Layout: [Fat Header 64B][VM 0][VM 1]...[Bytecode][Knowledge]
  let fat_hdr_size = 64;
  let arch_entries = [];

  let offset = fat_hdr_size;
  i = 0;
  while i < len(config.archs) {
    let arch = config.archs[i];
    push(arch_entries, {
      arch_id: arch.arch_id,
      vm_off: offset,
      vm_size: len(vm_codes[i]),
      entry_off: arch.entry_off
    });
    offset = offset + len(vm_codes[i]);
    i = i + 1;
  }

  let bc_off = offset;
  let kn_off = bc_off + len(bytecode);

  // 5. Build fat header
  let hdr = make_fat_header(arch_entries, bc_off, len(bytecode), kn_off, len(knowledge));

  // 6. Assemble fat binary
  let fat = [];
  concat_bytes(fat, hdr);
  i = 0;
  while i < len(vm_codes) {
    concat_bytes(fat, vm_codes[i]);
    i = i + 1;
  }
  concat_bytes(fat, bytecode);
  concat_bytes(fat, knowledge);

  // 7. Write fat binary
  file_write_bytes(config.output, fat);
  emit("  Fat binary: " + config.output + " (" + to_string(len(fat)) + " bytes)\n");

  // 8. Generate ELF loader stubs
  let fat_path = to_bytes(config.output);
  if config.stub_x86 != "" {
    let x86_stub_code = make_x86_64_stub(fat_path);
    let x86_elf = make_elf_arch(x86_stub_code, 0, "x86_64");
    file_write_bytes(config.stub_x86, x86_elf);
    emit("  Stub [x86_64]: " + config.stub_x86 + " (" + to_string(len(x86_elf)) + " bytes)\n");
  }
  if config.stub_arm != "" {
    let arm_stub_code = make_arm64_stub(fat_path);
    let arm_elf = make_elf_arch(arm_stub_code, 0, "arm64");
    file_write_bytes(config.stub_arm, arm_elf);
    emit("  Stub [arm64]: " + config.stub_arm + " (" + to_string(len(arm_elf)) + " bytes)\n");
  }

  emit("Done! Fat binary with " + to_string(len(config.archs)) + " architectures.\n");
}

fn to_bytes(str) {
  return __str_bytes(str);
}
