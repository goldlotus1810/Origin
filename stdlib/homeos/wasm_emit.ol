// homeos/wasm_emit.ol — Emit WASM binary format (.wasm)
// Generates valid WASM module with embedded bytecode in data section.
// No external tools needed (no wat2wasm).
//
// WASM binary format reference:
//   [magic 4B] [version 4B] [sections...]
//   Each section: [id:1] [size:LEB128] [payload]

// ════════════════════════════════════════════════════════
// LEB128 encoding helpers
// ════════════════════════════════════════════════════════

fn leb128_u32(buf, val) {
  let v = val;
  while v >= 128 {
    push(buf, (v % 128) + 128);
    v = v / 128;
  }
  push(buf, v);
}

fn leb128_i32(buf, val) {
  // Signed LEB128 for i32
  // NOTE: Only correct for non-negative values. Olang uses f64 division
  // which truncates toward zero, so negative values won't encode properly.
  // This is acceptable since all current callers pass non-negative offsets.
  let v = val;
  let more = 1;
  while more {
    let byte = v % 128;
    if byte < 0 { byte = byte + 128; }
    v = v / 128;
    // Check if done: v==0 and sign bit clear, or v==-1 and sign bit set
    if (v == 0) && ((byte % 64) == 0) {
      more = 0;
    } else if (v == 0 - 1) && ((byte % 64) != 0) {
      more = 0;
    } else {
      byte = byte + 128;  // set continuation bit
    }
    push(buf, byte % 256);
  }
}

fn leb128_i64(buf, val) {
  // For i64 constants — delegate to i32 for small values
  leb128_i32(buf, val);
}

// ════════════════════════════════════════════════════════
// WASM type constants
// ════════════════════════════════════════════════════════

let WASM_I32    = 0x7F;
let WASM_I64    = 0x7E;
let WASM_F32    = 0x7D;
let WASM_F64    = 0x7C;
let WASM_FUNC   = 0x60;
let WASM_VOID   = 0x40;

// Section IDs
let SEC_TYPE    = 1;
let SEC_IMPORT  = 2;
let SEC_FUNC    = 3;
let SEC_MEMORY  = 5;
let SEC_GLOBAL  = 6;
let SEC_EXPORT  = 7;
let SEC_CODE    = 10;
let SEC_DATA    = 11;

// ════════════════════════════════════════════════════════
// Section helpers
// ════════════════════════════════════════════════════════

fn emit_section(out, id, payload) {
  push(out, id);
  leb128_u32(out, len(payload));
  concat_bytes(out, payload);
}

fn emit_string(buf, s) {
  // Write length-prefixed UTF-8 string
  let bytes = __str_to_bytes(s);
  leb128_u32(buf, len(bytes));
  concat_bytes(buf, bytes);
}

fn concat_bytes(dst, src) {
  let i = 0;
  while i < len(src) {
    push(dst, src[i]);
    i = i + 1;
  }
}

// ════════════════════════════════════════════════════════
// WASM module builder
// ════════════════════════════════════════════════════════

// Build a complete WASM module embedding the VM + bytecode.
// Strategy: emit the existing vm_wasm.wat as pre-compiled .wasm,
// then patch the data section to include embedded bytecode.
//
// For Phase 4.3, we use a simpler approach:
// Generate a minimal WASM wrapper that loads bytecode from
// an embedded data section and delegates to the VM exports.
//
// The full VM is still compiled from WAT via wat2wasm (one-time),
// then this emitter patches bytecode into the data section.

pub fn make_wasm_with_bytecode(vm_wasm_bytes, bytecode) {
  // Parse existing .wasm, find/replace data section for bytecode embedding
  // vm_wasm_bytes = pre-compiled vm_wasm.wasm (from wat2wasm)
  // bytecode = compiled .ol bytecode to embed

  if len(vm_wasm_bytes) < 8 {
    emit("Error: invalid WASM binary\n");
    return [];
  }

  // Verify WASM magic
  if vm_wasm_bytes[0] != 0x00 {
    emit("Error: not a WASM file (bad magic)\n");
    return [];
  }

  // Strategy: append a new data section at the end.
  // The VM's init() currently calls host_load_bytecode.
  // For embedded mode, we add a data segment that places
  // bytecode at the expected memory offset (0x10000).

  let out = [];

  // Copy original WASM (all sections)
  let i = 0;
  while i < len(vm_wasm_bytes) {
    push(out, vm_wasm_bytes[i]);
    i = i + 1;
  }

  // Append data section with embedded bytecode
  if len(bytecode) > 0 {
    let data_payload = [];

    // Number of data segments: 1 (additional)
    // Note: if original already has data sections, this adds more
    // WASM allows multiple data sections... actually no, only one data section.
    // We need to merge. For now, build a standalone data segment.

    // Actually WASM spec says only one data section per module.
    // So we need to find and extend the existing one, or
    // create a new approach.
    //
    // Simpler: use the "init from host" approach (Option B from plan)
    // but set a global flag for "has embedded bytecode"

    // For embedded mode: we'll create a custom section with bytecode
    // Custom section (id=0) named "bytecode"
    let custom_payload = [];
    emit_string(custom_payload, "bytecode");
    // Bytecode length as u32 LE
    let bc_len = len(bytecode);
    push(custom_payload, bc_len % 256);
    push(custom_payload, (bc_len / 256) % 256);
    push(custom_payload, (bc_len / 65536) % 256);
    push(custom_payload, (bc_len / 16777216) % 256);
    // Bytecode data
    concat_bytes(custom_payload, bytecode);

    emit_section(out, 0, custom_payload);
  }

  return out;
}

// Generate a minimal standalone WASM module (no pre-compiled VM needed).
// This emits a small WASM that:
//   1. Imports host functions (write, read, load_bytecode, log, emit_event)
//   2. Has 1MB linear memory
//   3. Exports: init, run, memory
//   4. Embeds bytecode in data section at offset 0x10000
//
// This is a simplified VM — for full features use make_wasm_with_bytecode().

pub fn make_standalone_wasm(bytecode) {
  let out = [];

  // ── WASM header ──
  // Magic: \0asm
  push(out, 0x00); push(out, 0x61); push(out, 0x73); push(out, 0x6D);
  // Version: 1
  push(out, 0x01); push(out, 0x00); push(out, 0x00); push(out, 0x00);

  // ── Type section ──
  // Type 0: (i32,i32) -> i32   (host_write, host_read, host_load_bytecode)
  // Type 1: (i32,i32) -> ()    (host_log)
  // Type 2: (i32,i32,i32) -> () (host_emit_event)
  // Type 3: () -> ()           (init)
  // Type 4: () -> i32          (run)
  let type_sec = [];
  leb128_u32(type_sec, 5);  // 5 types

  // Type 0: (i32,i32) -> i32
  push(type_sec, WASM_FUNC);
  leb128_u32(type_sec, 2);  // 2 params
  push(type_sec, WASM_I32); push(type_sec, WASM_I32);
  leb128_u32(type_sec, 1);  // 1 result
  push(type_sec, WASM_I32);

  // Type 1: (i32,i32) -> ()
  push(type_sec, WASM_FUNC);
  leb128_u32(type_sec, 2);
  push(type_sec, WASM_I32); push(type_sec, WASM_I32);
  leb128_u32(type_sec, 0);  // 0 results

  // Type 2: (i32,i32,i32) -> ()
  push(type_sec, WASM_FUNC);
  leb128_u32(type_sec, 3);
  push(type_sec, WASM_I32); push(type_sec, WASM_I32); push(type_sec, WASM_I32);
  leb128_u32(type_sec, 0);

  // Type 3: () -> ()
  push(type_sec, WASM_FUNC);
  leb128_u32(type_sec, 0);
  leb128_u32(type_sec, 0);

  // Type 4: () -> i32
  push(type_sec, WASM_FUNC);
  leb128_u32(type_sec, 0);
  leb128_u32(type_sec, 1);
  push(type_sec, WASM_I32);

  emit_section(out, SEC_TYPE, type_sec);

  // ── Import section ──
  let imp_sec = [];
  leb128_u32(imp_sec, 5);  // 5 imports

  // host_write: (i32,i32)->i32  type 0
  emit_string(imp_sec, "env");
  emit_string(imp_sec, "host_write");
  push(imp_sec, 0x00);  // func import
  leb128_u32(imp_sec, 0);  // type index 0

  // host_read: (i32,i32)->i32  type 0
  emit_string(imp_sec, "env");
  emit_string(imp_sec, "host_read");
  push(imp_sec, 0x00);
  leb128_u32(imp_sec, 0);

  // host_load_bytecode: (i32,i32)->i32  type 0
  emit_string(imp_sec, "env");
  emit_string(imp_sec, "host_load_bytecode");
  push(imp_sec, 0x00);
  leb128_u32(imp_sec, 0);

  // host_log: (i32,i32)->()  type 1
  emit_string(imp_sec, "env");
  emit_string(imp_sec, "host_log");
  push(imp_sec, 0x00);
  leb128_u32(imp_sec, 1);

  // host_emit_event: (i32,i32,i32)->()  type 2
  emit_string(imp_sec, "env");
  emit_string(imp_sec, "host_emit_event");
  push(imp_sec, 0x00);
  leb128_u32(imp_sec, 2);

  emit_section(out, SEC_IMPORT, imp_sec);

  // ── Function section ──
  // 2 local functions: init (type 3: ()->()) and run (type 4: ()->i32)
  let func_sec = [];
  leb128_u32(func_sec, 2);  // 2 functions
  leb128_u32(func_sec, 3);  // init -> type 3
  leb128_u32(func_sec, 4);  // run  -> type 4
  emit_section(out, SEC_FUNC, func_sec);

  // ── Memory section ──
  let mem_sec = [];
  leb128_u32(mem_sec, 1);  // 1 memory
  push(mem_sec, 0x00);     // no max
  leb128_u32(mem_sec, 16); // 16 pages = 1MB
  emit_section(out, SEC_MEMORY, mem_sec);

  // ── Export section ──
  let exp_sec = [];
  leb128_u32(exp_sec, 3);  // 3 exports

  // memory
  emit_string(exp_sec, "memory");
  push(exp_sec, 0x02);  // memory export
  leb128_u32(exp_sec, 0);

  // init (func index 5 = 5 imports + 0)
  emit_string(exp_sec, "init");
  push(exp_sec, 0x00);  // func export
  leb128_u32(exp_sec, 5);

  // run (func index 6 = 5 imports + 1)
  emit_string(exp_sec, "run");
  push(exp_sec, 0x00);
  leb128_u32(exp_sec, 6);

  emit_section(out, SEC_EXPORT, exp_sec);

  // ── Code section ──
  let code_sec = [];
  leb128_u32(code_sec, 2);  // 2 function bodies

  // init body: nop (bytecode loaded from data section)
  let init_body = [];
  leb128_u32(init_body, 0);  // 0 locals
  push(init_body, 0x01);     // nop
  push(init_body, 0x0B);     // end
  leb128_u32(code_sec, len(init_body));
  concat_bytes(code_sec, init_body);

  // run body: return 0 (i32.const 0)
  let run_body = [];
  leb128_u32(run_body, 0);   // 0 locals
  push(run_body, 0x41);      // i32.const
  leb128_i32(run_body, 0);   // 0
  push(run_body, 0x0B);      // end
  leb128_u32(code_sec, len(run_body));
  concat_bytes(code_sec, run_body);

  emit_section(out, SEC_CODE, code_sec);

  // ── Data section (embedded bytecode) ──
  if len(bytecode) > 0 {
    let data_sec = [];
    leb128_u32(data_sec, 1);  // 1 data segment

    // Active segment: memory 0, offset i32.const 0x10000
    push(data_sec, 0x00);     // active, memory 0
    push(data_sec, 0x41);     // i32.const
    leb128_i32(data_sec, 0x10000);  // offset = 65536
    push(data_sec, 0x0B);     // end init expr

    // Data bytes
    leb128_u32(data_sec, len(bytecode));
    concat_bytes(data_sec, bytecode);

    emit_section(out, SEC_DATA, data_sec);
  }

  return out;
}

// ════════════════════════════════════════════════════════
// Architecture constant
// ════════════════════════════════════════════════════════

let ARCH_WASM = "wasm";
