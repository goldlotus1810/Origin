// homeos/builder.ol — Self-sufficient builder
// Replaces Rust builder: compile .ol → bytecode, pack with VM → origin.olang
//
// Usage: origin.olang runs this to build a new origin.olang
//   o run builder.ol --stdlib stdlib/ --output origin_new.olang

pub fn build(config) {
  emit("Builder — origin.olang packer (Olang)\n");

  // 1. Compile all .ol → bytecode
  let bytecode = [];
  if config.stdlib_path != "" {
    emit("  Compiling stdlib: " + config.stdlib_path + "\n");
    bytecode = compile_all(config.stdlib_path);
  }
  emit("  Bytecode: " + to_string(len(bytecode)) + " bytes\n");

  // 2. Read VM code (pre-assembled binary)
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
  let origin_hdr = make_origin_header(
    152,                          // vm_offset (after ELF 120 + origin 32)
    len(vm_code),
    152 + len(vm_code),           // bc_offset
    len(bytecode),
    152 + len(vm_code) + len(bytecode),  // kn_offset
    len(knowledge),
    0                             // flags
  );

  // Concat all sections
  let payload = [];
  concat_bytes(payload, origin_hdr);
  concat_bytes(payload, vm_code);
  concat_bytes(payload, bytecode);
  concat_bytes(payload, knowledge);

  // Wrap in ELF
  let binary = make_elf(payload, 32);  // entry = 32 bytes past origin header = VM start

  // 5. Write output
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
    output: "origin_new.olang"
  };
}
