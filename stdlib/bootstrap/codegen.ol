// ── Olang Bootstrap Code Generator ──────────────────────────────
// Self-hosting preparation: bytecode encoder written in Olang.
// Reads IR ops (from semantic.ol) → emits binary bytecode.
//
// Phase 4 / A9 — compiler self-hosting foundation.
// Depends on: stdlib/bootstrap/semantic.ol (Op type)
//
// Reference: plans/PLAN_0_5_codegen.md

use olang.bootstrap.lexer;
use olang.bootstrap.parser;
use olang.bootstrap.semantic;

// ── Opcode tags ────────────────────────────────────────────────
// Each opcode has a unique 1-byte tag.
// (decimal values — hex equivalents in comments)

let TAG_PUSH       = 1;   // 0x01
let TAG_LOAD       = 2;   // 0x02
let TAG_LCA        = 3;   // 0x03
let TAG_EDGE       = 4;   // 0x04
let TAG_QUERY      = 5;   // 0x05
let TAG_EMIT       = 6;   // 0x06
let TAG_CALL       = 7;   // 0x07
let TAG_RET        = 8;   // 0x08
let TAG_JMP        = 9;   // 0x09
let TAG_JZ         = 10;  // 0x0A
let TAG_DUP        = 11;  // 0x0B
let TAG_POP        = 12;  // 0x0C
let TAG_SWAP       = 13;  // 0x0D
let TAG_LOOP       = 14;  // 0x0E
let TAG_HALT       = 15;  // 0x0F
let TAG_DREAM      = 16;  // 0x10
let TAG_STATS      = 17;  // 0x11
let TAG_NOP        = 18;  // 0x12
let TAG_STORE      = 19;  // 0x13
let TAG_LOADLOCAL  = 20;  // 0x14
let TAG_PUSHNUM    = 21;  // 0x15
let TAG_FUSE       = 22;  // 0x16
let TAG_SCOPEBEGIN = 23;  // 0x17
let TAG_SCOPEEND   = 24;  // 0x18
let TAG_PUSHMOL    = 25;  // 0x19
let TAG_TRYBEGIN   = 26;  // 0x1A
let TAG_CATCHEND   = 27;  // 0x1B
let TAG_STOREUPDATE = 28; // 0x1C
let TAG_TRACE      = 29;  // 0x1D
let TAG_INSPECT    = 30;  // 0x1E
let TAG_ASSERT     = 31;  // 0x1F
let TAG_TYPEOF     = 32;  // 0x20
let TAG_WHY        = 33;  // 0x21
let TAG_EXPLAIN    = 34;  // 0x22
let TAG_FFI        = 35;  // 0x23
let TAG_CALLCLOSURE = 36; // 0x24

// ── Byte encoding helpers ──────────────────────────────────────

fn emit_byte(_eb, b) {
    push(_eb, b);
}

fn emit_u16_le(_eb, n) {
    push(_eb, n % 256);
    push(_eb, (n / 256) % 256);
}

fn emit_u32_le(_eb, n) {
    push(_eb, n % 256);
    push(_eb, (n / 256) % 256);
    push(_eb, (n / 65536) % 256);
    push(_eb, (n / 16777216) % 256);
}

fn emit_f64_le(_eb, n) {
    // Use VM builtin to get IEEE 754 LE bytes
    let _ef_bytes = f64_to_le_bytes(n);
    let fi = 0;
    while fi < 8 {
        push(_eb, _ef_bytes[fi]);
        let fi = fi + 1;
    };
}

fn emit_str(_eb, s) {
    // Encode string as [len:1][utf8_bytes:N]
    let _es_bytes = str_bytes(s);
    let _es_len = len(_es_bytes);
    push(_eb, _es_len);
    let si = 0;
    while si < _es_len {
        push(_eb, _es_bytes[si]);
        let si = si + 1;
    };
}

fn emit_str_u16(_eb, s) {
    // Encode Push chain: [mol_count:2 LE][u16_mol_0:2 LE][u16_mol_1:2 LE]...
    // Each char → u16 molecule = 0x2100 | byte_value
    let _eu_bytes = str_bytes(s);
    let _eu_len = len(_eu_bytes);
    emit_u16_le(_eb, _eu_len);
    let su = 0;
    while su < _eu_len {
        let mol = _eu_bytes[su] + 8448;
        push(_eb, mol % 256);
        push(_eb, mol / 256);
        let su = su + 1;
    };
}

// ── Opcode tag lookup ──────────────────────────────────────────

fn tag_for(op_tag) {
    if op_tag == "Push"         { return TAG_PUSH; };
    if op_tag == "Load"         { return TAG_LOAD; };
    if op_tag == "Lca"          { return TAG_LCA; };
    if op_tag == "Edge"         { return TAG_EDGE; };
    if op_tag == "Query"        { return TAG_QUERY; };
    if op_tag == "Emit"         { return TAG_EMIT; };
    if op_tag == "Call"         { return TAG_CALL; };
    if op_tag == "Ret"          { return TAG_RET; };
    if op_tag == "Jmp"          { return TAG_JMP; };
    if op_tag == "Jz"           { return TAG_JZ; };
    if op_tag == "Dup"          { return TAG_DUP; };
    if op_tag == "Pop"          { return TAG_POP; };
    if op_tag == "Swap"         { return TAG_SWAP; };
    if op_tag == "Loop"         { return TAG_LOOP; };
    if op_tag == "Halt"         { return TAG_HALT; };
    if op_tag == "Dream"        { return TAG_DREAM; };
    if op_tag == "Stats"        { return TAG_STATS; };
    if op_tag == "Nop"          { return TAG_NOP; };
    if op_tag == "Store"        { return TAG_STORE; };
    if op_tag == "LoadLocal"    { return TAG_LOADLOCAL; };
    if op_tag == "PushNum"      { return TAG_PUSHNUM; };
    if op_tag == "Fuse"         { return TAG_FUSE; };
    if op_tag == "ScopeBegin"   { return TAG_SCOPEBEGIN; };
    if op_tag == "ScopeEnd"     { return TAG_SCOPEEND; };
    if op_tag == "PushMol"      { return TAG_PUSHMOL; };
    if op_tag == "TryBegin"     { return TAG_TRYBEGIN; };
    if op_tag == "CatchEnd"     { return TAG_CATCHEND; };
    if op_tag == "StoreUpdate"  { return TAG_STOREUPDATE; };
    if op_tag == "Trace"        { return TAG_TRACE; };
    if op_tag == "Inspect"      { return TAG_INSPECT; };
    if op_tag == "Assert"       { return TAG_ASSERT; };
    if op_tag == "TypeOf"       { return TAG_TYPEOF; };
    if op_tag == "Why"          { return TAG_WHY; };
    if op_tag == "Explain"      { return TAG_EXPLAIN; };
    if op_tag == "Ffi"          { return TAG_FFI; };
    if op_tag == "CallClosure"  { return TAG_CALLCLOSURE; };
    return 0;
}

// ── Main encoder ───────────────────────────────────────────────

fn encode_op(_eo_out, op) {
    let t = op[0];
    if t == "PushNum" {
        emit_byte(_eo_out, 21);
        emit_f64_le(_eo_out, op[2]);
        return;
    };
    if t == "Emit" { emit_byte(_eo_out, 6); return; };
    if t == "Halt" { emit_byte(_eo_out, 15); return; };
    if t == "Ret" { emit_byte(_eo_out, 8); return; };
    if t == "Pop" { emit_byte(_eo_out, 12); return; };
    if t == "Dup" { emit_byte(_eo_out, 11); return; };
    if t == "ScopeBegin" { emit_byte(_eo_out, 23); return; };
    if t == "ScopeEnd" { emit_byte(_eo_out, 24); return; };
    if t == "Push" {
        emit_byte(_eo_out, 1);
        emit_str_u16(_eo_out, op[1]);
        return;
    };
    if t == "Load" {
        emit_byte(_eo_out, 2);
        emit_str(_eo_out, op[1]);
        return;
    };
    if t == "Store" {
        emit_byte(_eo_out, 19);
        emit_str(_eo_out, op[1]);
        return;
    };
    if t == "LoadLocal" {
        emit_byte(_eo_out, 20);
        emit_str(_eo_out, op[1]);
        return;
    };
    if t == "StoreUpdate" {
        emit_byte(_eo_out, 28);
        emit_str(_eo_out, op[1]);
        return;
    };
    if t == "Call" {
        emit_byte(_eo_out, 7);
        emit_str(_eo_out, op[1]);
        return;
    };
    if t == "Jmp" {
        emit_byte(_eo_out, 9);
        emit_u32_le(_eo_out, op[2]);
        return;
    };
    if t == "Jz" {
        emit_byte(_eo_out, 10);
        emit_u32_le(_eo_out, op[2]);
        return;
    };
    if t == "Swap" { emit_byte(_eo_out, 13); return; };
    if t == "Closure" {
        // Closure: [0x25][param_count:1][body_len:4]
        emit_byte(_eo_out, 37);
        emit_byte(_eo_out, op[2]);
        emit_u32_le(_eo_out, op[1]);
        return;
    };
}

// ── Op byte size (for jump target resolution) ─────────────────

fn op_size(_os_op) {
    let _os_t = _os_op[0];
    if _os_t == "PushNum" { return 9; };
    if _os_t == "Push" {
        let _os_name = _os_op[1];
        let _os_b = str_bytes(_os_name);
        let _os_len = len(_os_b);
        let _os_result = 3 + _os_len * 2;
        return _os_result;
    };
    if _os_t == "Load" || _os_t == "Store" || _os_t == "LoadLocal"
        || _os_t == "StoreUpdate" || _os_t == "Call" {
        let _os_name2 = _os_op[1];
        let _os_b2 = str_bytes(_os_name2);
        let _os_len2 = len(_os_b2);
        let _os_result2 = 2 + _os_len2;
        return _os_result2;
    };
    if _os_t == "Jmp" || _os_t == "Jz" || _os_t == "Loop" || _os_t == "TryBegin" { return 5; };
    if _os_t == "PushMol" { return 3; };
    if _os_t == "Edge" || _os_t == "Query" { return 2; };
    return 1;
}

// ── Entry point ────────────────────────────────────────────────

pub fn generate(ops) {
    // Pass 1: measure actual encoded size by encoding to fresh temp arrays.
    let offsets = [];
    let _gpos = 0;
    let _gi = 0;
    while _gi < len(ops) {
        push(offsets, _gpos);
        let _gt1 = [];
        encode_op(_gt1, ops[_gi]);
        let _gpos = _gpos + len(_gt1);
        let _gi = _gi + 1;
    };
    push(offsets, _gpos);

    // Pass 2: encode for real, resolving Jmp/Jz targets.
    let _gout = [];
    let _gi2 = 0;
    while _gi2 < len(ops) {
        let _gop = ops[_gi2];
        let _gt = _gop[0];
        if _gt == "Jmp" {
            let _gtarget = _gop[2];
            if _gtarget < len(offsets) {
                emit_byte(_gout, 9);
                emit_u32_le(_gout, offsets[_gtarget]);
            } else {
                encode_op(_gout, _gop);
            };
        } else {
            if _gt == "Jz" {
                let _gtarget = _gop[2];
                if _gtarget < len(offsets) {
                    emit_byte(_gout, 10);
                    emit_u32_le(_gout, offsets[_gtarget]);
                } else {
                    encode_op(_gout, _gop);
                };
            } else {
                if _gt == "Closure" {
                    let _gbody_ops = _gop[1];
                    let _gbody_start = _gi2 + 1;
                    let _gbody_end = _gbody_start + _gbody_ops;
                    let _gbyte_len = offsets[_gbody_end] - offsets[_gbody_start];
                    emit_byte(_gout, 37);
                    emit_byte(_gout, _gop[2]);
                    emit_u32_le(_gout, _gbyte_len);
                } else {
                    encode_op(_gout, _gop);
                };
            };
        };
        let _gi2 = _gi2 + 1;
    };
    return _gout;
}
