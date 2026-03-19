//! # bytecode — Binary bytecode decoder
//!
//! Decodes binary bytecode produced by `codegen.ol` back into `Vec<Op>`.
//! Format: each opcode = 1-byte tag + optional payload.
//!
//! Tag assignments match `codegen.ol` (PLAN_0_5 format):
//!   0x01=Push  0x02=Load  0x03=Lca  0x04=Edge  0x05=Query  0x06=Emit
//!   0x07=Call  0x08=Ret   0x09=Jmp  0x0A=Jz    0x0B=Dup    0x0C=Pop
//!   0x0D=Swap  0x0E=Loop  0x0F=Halt 0x10=Dream 0x11=Stats  0x12=Nop
//!   0x13=Store 0x14=LoadLocal 0x15=PushNum 0x16=Fuse 0x17=ScopeBegin
//!   0x18=ScopeEnd 0x19=PushMol 0x1A=TryBegin 0x1B=CatchEnd
//!   0x1C=StoreUpdate 0x1D=Trace 0x1E=Inspect 0x1F=Assert 0x20=TypeOf
//!   0x21=Why 0x22=Explain 0x23=Ffi 0x24=CallClosure

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use super::ir::Op;

/// Error during bytecode decoding.
#[derive(Debug, Clone, PartialEq)]
pub enum DecodeError {
    /// Unexpected end of input while reading payload.
    UnexpectedEof,
    /// Unknown opcode tag.
    UnknownOpcode(u8),
    /// Invalid UTF-8 in string payload.
    InvalidUtf8,
    /// Invalid chain bytes.
    InvalidChain,
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "unexpected end of bytecode"),
            Self::UnknownOpcode(tag) => write!(f, "unknown opcode tag: 0x{:02X}", tag),
            Self::InvalidUtf8 => write!(f, "invalid UTF-8 in string payload"),
            Self::InvalidChain => write!(f, "invalid chain bytes"),
        }
    }
}

/// Bytecode decoder state.
struct Decoder<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    fn read_u8(&mut self) -> Result<u8, DecodeError> {
        if self.pos >= self.data.len() {
            return Err(DecodeError::UnexpectedEof);
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn read_u16_le(&mut self) -> Result<u16, DecodeError> {
        if self.remaining() < 2 {
            return Err(DecodeError::UnexpectedEof);
        }
        let lo = self.data[self.pos] as u16;
        let hi = self.data[self.pos + 1] as u16;
        self.pos += 2;
        Ok(lo | (hi << 8))
    }

    fn read_u32_le(&mut self) -> Result<u32, DecodeError> {
        if self.remaining() < 4 {
            return Err(DecodeError::UnexpectedEof);
        }
        let b0 = self.data[self.pos] as u32;
        let b1 = self.data[self.pos + 1] as u32;
        let b2 = self.data[self.pos + 2] as u32;
        let b3 = self.data[self.pos + 3] as u32;
        self.pos += 4;
        Ok(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
    }

    fn read_f64_le(&mut self) -> Result<f64, DecodeError> {
        if self.remaining() < 8 {
            return Err(DecodeError::UnexpectedEof);
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[self.pos..self.pos + 8]);
        self.pos += 8;
        Ok(f64::from_le_bytes(bytes))
    }

    /// Read string: [len:1][utf8:N]
    fn read_str(&mut self) -> Result<String, DecodeError> {
        let slen = self.read_u8()? as usize;
        if self.remaining() < slen {
            return Err(DecodeError::UnexpectedEof);
        }
        let bytes = &self.data[self.pos..self.pos + slen];
        self.pos += slen;
        String::from_utf8(bytes.to_vec()).map_err(|_| DecodeError::InvalidUtf8)
    }

    /// Read string: [len:2][utf8:N]
    fn read_str_u16(&mut self) -> Result<String, DecodeError> {
        let slen = self.read_u16_le()? as usize;
        if self.remaining() < slen {
            return Err(DecodeError::UnexpectedEof);
        }
        let bytes = &self.data[self.pos..self.pos + slen];
        self.pos += slen;
        String::from_utf8(bytes.to_vec()).map_err(|_| DecodeError::InvalidUtf8)
    }
}

/// Decode binary bytecode (produced by codegen.ol) into a list of IR ops.
///
/// The format uses 1-byte tags (0x01..0x24) with variable payloads.
/// This is the PLAN_0_5 bytecode format, distinct from `Op::to_bytes()`
/// which uses a different tag assignment for the existing Rust pipeline.
pub fn decode_bytecode(bytes: &[u8]) -> Result<Vec<Op>, DecodeError> {
    let mut dec = Decoder::new(bytes);
    let mut ops = Vec::new();

    while dec.pos < dec.data.len() {
        let tag = dec.read_u8()?;
        let op = match tag {
            0x01 => {
                // Push: [chain_len:2][chain_bytes:N]
                // In bootstrap codegen, Push carries a name string.
                // We decode it as a Load since Rust Op::Push needs MolecularChain.
                let name = dec.read_str_u16()?;
                Op::Load(name)
            }
            0x02 => {
                // Load: [name_len:1][name:N]
                Op::Load(dec.read_str()?)
            }
            0x03 => Op::Lca,
            0x04 => {
                // Edge: [rel:1]
                Op::Edge(dec.read_u8()?)
            }
            0x05 => {
                // Query: [rel:1]
                Op::Query(dec.read_u8()?)
            }
            0x06 => Op::Emit,
            0x07 => {
                // Call: [name_len:1][name:N]
                Op::Call(dec.read_str()?)
            }
            0x08 => Op::Ret,
            0x09 => {
                // Jmp: [target:4]
                Op::Jmp(dec.read_u32_le()? as usize)
            }
            0x0A => {
                // Jz: [target:4]
                Op::Jz(dec.read_u32_le()? as usize)
            }
            0x0B => Op::Dup,
            0x0C => Op::Pop,
            0x0D => Op::Swap,
            0x0E => {
                // Loop: [count:4]
                Op::Loop(dec.read_u32_le()?)
            }
            0x0F => Op::Halt,
            0x10 => Op::Dream,
            0x11 => Op::Stats,
            0x12 => Op::Nop,
            0x13 => {
                // Store: [name_len:1][name:N]
                Op::Store(dec.read_str()?)
            }
            0x14 => {
                // LoadLocal: [name_len:1][name:N]
                Op::LoadLocal(dec.read_str()?)
            }
            0x15 => {
                // PushNum: [f64:8]
                Op::PushNum(dec.read_f64_le()?)
            }
            0x16 => Op::Fuse,
            0x17 => Op::ScopeBegin,
            0x18 => Op::ScopeEnd,
            0x19 => {
                // PushMol: [s:1][r:1][v:1][a:1][t:1]
                let s = dec.read_u8()?;
                let r = dec.read_u8()?;
                let v = dec.read_u8()?;
                let a = dec.read_u8()?;
                let t = dec.read_u8()?;
                Op::PushMol(s, r, v, a, t)
            }
            0x1A => {
                // TryBegin: [catch_pc:4]
                Op::TryBegin(dec.read_u32_le()? as usize)
            }
            0x1B => Op::CatchEnd,
            0x1C => {
                // StoreUpdate: [name_len:1][name:N]
                Op::StoreUpdate(dec.read_str()?)
            }
            0x1D => Op::Trace,
            0x1E => Op::Inspect,
            0x1F => Op::Assert,
            0x20 => Op::TypeOf,
            0x21 => Op::Why,
            0x22 => Op::Explain,
            0x23 => {
                // Ffi: [name_len:1][name:N][arity:1]
                let name = dec.read_str()?;
                let arity = dec.read_u8()?;
                Op::Ffi(name, arity)
            }
            0x24 => {
                // CallClosure: [name_len:1][name:N][arity:1]
                // Note: in the bootstrap VM, CallClosure carries a name + arity.
                // The Rust Op::CallClosure only has arity. We decode the name
                // but map to CallClosure(arity) since the Rust VM uses it that way.
                let _name = dec.read_str()?;
                let arity = dec.read_u8()?;
                Op::CallClosure(arity)
            }
            0x25 => {
                // Closure: [param_count:1][body_len:4]
                let param_count = dec.read_u8()?;
                let body_len = dec.read_u32_le()? as usize;
                Op::Closure(param_count, body_len)
            }
            _ => return Err(DecodeError::UnknownOpcode(tag)),
        };
        ops.push(op);
    }

    Ok(ops)
}

// ─────────────────────────────────────────────────────────────────────────────
// Encoder (Rust-side, mirrors codegen.ol for round-trip testing)
// ─────────────────────────────────────────────────────────────────────────────

/// Encode a list of IR ops into PLAN_0_5 bytecode format.
/// This mirrors codegen.ol's `generate()` function for round-trip testing.
pub fn encode_bytecode(ops: &[Op]) -> Vec<u8> {
    let mut out = Vec::new();
    for op in ops {
        encode_op(&mut out, op);
    }
    out
}

fn emit_byte(out: &mut Vec<u8>, b: u8) {
    out.push(b);
}

fn emit_u16_le(out: &mut Vec<u8>, n: u16) {
    out.push((n & 0xFF) as u8);
    out.push(((n >> 8) & 0xFF) as u8);
}

fn emit_u32_le(out: &mut Vec<u8>, n: u32) {
    out.extend_from_slice(&n.to_le_bytes());
}

fn emit_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    out.push(bytes.len() as u8);
    out.extend_from_slice(bytes);
}

fn emit_str_u16(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    emit_u16_le(out, bytes.len() as u16);
    out.extend_from_slice(bytes);
}

fn encode_op(out: &mut Vec<u8>, op: &Op) {
    match op {
        Op::Push(chain) => {
            emit_byte(out, 0x01);
            // Push uses the chain's alias/display as the name
            let name = alloc::format!("{:?}", chain);
            emit_str_u16(out, &name);
        }
        Op::Load(name) => {
            emit_byte(out, 0x02);
            emit_str(out, name);
        }
        Op::Lca => emit_byte(out, 0x03),
        Op::Edge(rel) => {
            emit_byte(out, 0x04);
            emit_byte(out, *rel);
        }
        Op::Query(rel) => {
            emit_byte(out, 0x05);
            emit_byte(out, *rel);
        }
        Op::Emit => emit_byte(out, 0x06),
        Op::Call(name) => {
            emit_byte(out, 0x07);
            emit_str(out, name);
        }
        Op::Ret => emit_byte(out, 0x08),
        Op::Jmp(target) => {
            emit_byte(out, 0x09);
            emit_u32_le(out, *target as u32);
        }
        Op::Jz(target) => {
            emit_byte(out, 0x0A);
            emit_u32_le(out, *target as u32);
        }
        Op::Dup => emit_byte(out, 0x0B),
        Op::Pop => emit_byte(out, 0x0C),
        Op::Swap => emit_byte(out, 0x0D),
        Op::Loop(count) => {
            emit_byte(out, 0x0E);
            emit_u32_le(out, *count);
        }
        Op::Halt => emit_byte(out, 0x0F),
        Op::Dream => emit_byte(out, 0x10),
        Op::Stats => emit_byte(out, 0x11),
        Op::Nop => emit_byte(out, 0x12),
        Op::Store(name) => {
            emit_byte(out, 0x13);
            emit_str(out, name);
        }
        Op::LoadLocal(name) => {
            emit_byte(out, 0x14);
            emit_str(out, name);
        }
        Op::PushNum(n) => {
            emit_byte(out, 0x15);
            out.extend_from_slice(&n.to_le_bytes());
        }
        Op::Fuse => emit_byte(out, 0x16),
        Op::ScopeBegin => emit_byte(out, 0x17),
        Op::ScopeEnd => emit_byte(out, 0x18),
        Op::PushMol(s, r, v, a, t) => {
            emit_byte(out, 0x19);
            out.extend_from_slice(&[*s, *r, *v, *a, *t]);
        }
        Op::TryBegin(target) => {
            emit_byte(out, 0x1A);
            emit_u32_le(out, *target as u32);
        }
        Op::CatchEnd => emit_byte(out, 0x1B),
        Op::StoreUpdate(name) => {
            emit_byte(out, 0x1C);
            emit_str(out, name);
        }
        Op::Trace => emit_byte(out, 0x1D),
        Op::Inspect => emit_byte(out, 0x1E),
        Op::Assert => emit_byte(out, 0x1F),
        Op::TypeOf => emit_byte(out, 0x20),
        Op::Why => emit_byte(out, 0x21),
        Op::Explain => emit_byte(out, 0x22),
        Op::Ffi(name, arity) => {
            emit_byte(out, 0x23);
            emit_str(out, name);
            emit_byte(out, *arity);
        }
        Op::CallClosure(arity) => {
            emit_byte(out, 0x24);
            // CallClosure in Rust has no name; encode empty name for compat
            emit_str(out, "");
            emit_byte(out, *arity);
        }
        // 0x25: Closure [param_count:1][body_len:4]
        // Creates closure marker on stack, jumps over body.
        Op::Closure(param_count, body_len) => {
            emit_byte(out, 0x25);
            emit_byte(out, *param_count);
            emit_u32_le(out, *body_len as u32);
        }
        // Opcodes not in PLAN_0_5 format — skip silently
        Op::DeviceWrite(_) | Op::DeviceRead(_) | Op::DeviceList
        | Op::FileRead | Op::FileWrite | Op::FileAppend
        | Op::SpawnBegin | Op::SpawnEnd
        | Op::ChanNew | Op::ChanSend | Op::ChanRecv
        | Op::Select(_) => {}
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_empty() {
        let ops = decode_bytecode(&[]).unwrap();
        assert!(ops.is_empty());
    }

    #[test]
    fn decode_halt() {
        let ops = decode_bytecode(&[0x0F]).unwrap();
        assert_eq!(ops, alloc::vec![Op::Halt]);
    }

    #[test]
    fn decode_simple_no_payload() {
        let bytes = [0x03, 0x06, 0x08, 0x0B, 0x0C, 0x0D, 0x0F, 0x10, 0x11, 0x12];
        let ops = decode_bytecode(&bytes).unwrap();
        assert_eq!(ops, alloc::vec![
            Op::Lca, Op::Emit, Op::Ret, Op::Dup, Op::Pop, Op::Swap,
            Op::Halt, Op::Dream, Op::Stats, Op::Nop,
        ]);
    }

    #[test]
    fn decode_push_num() {
        let n: f64 = 42.0;
        let mut bytes = alloc::vec![0x15];
        bytes.extend_from_slice(&n.to_le_bytes());
        let ops = decode_bytecode(&bytes).unwrap();
        assert_eq!(ops.len(), 1);
        match &ops[0] {
            Op::PushNum(v) => assert_eq!(*v, 42.0),
            other => panic!("expected PushNum, got {:?}", other),
        }
    }

    #[test]
    fn decode_load_store() {
        // Load "x" = [0x02][0x01]['x']
        // Store "y" = [0x13][0x01]['y']
        let bytes = [0x02, 0x01, b'x', 0x13, 0x01, b'y'];
        let ops = decode_bytecode(&bytes).unwrap();
        assert_eq!(ops, alloc::vec![Op::Load("x".into()), Op::Store("y".into())]);
    }

    #[test]
    fn decode_jmp_jz() {
        // Jmp 100 = [0x09][100 as u32 LE]
        let mut bytes = alloc::vec![0x09];
        bytes.extend_from_slice(&100u32.to_le_bytes());
        // Jz 200 = [0x0A][200 as u32 LE]
        bytes.push(0x0A);
        bytes.extend_from_slice(&200u32.to_le_bytes());
        let ops = decode_bytecode(&bytes).unwrap();
        assert_eq!(ops, alloc::vec![Op::Jmp(100), Op::Jz(200)]);
    }

    #[test]
    fn decode_push_mol() {
        let bytes = [0x19, 1, 6, 200, 180, 4];
        let ops = decode_bytecode(&bytes).unwrap();
        assert_eq!(ops, alloc::vec![Op::PushMol(1, 6, 200, 180, 4)]);
    }

    #[test]
    fn decode_edge_query() {
        let bytes = [0x04, 0x03, 0x05, 0x07];
        let ops = decode_bytecode(&bytes).unwrap();
        assert_eq!(ops, alloc::vec![Op::Edge(3), Op::Query(7)]);
    }

    #[test]
    fn decode_ffi() {
        // Ffi: [0x23][name_len:1][name:N][arity:1]
        let bytes = [0x23, 0x04, b't', b'e', b's', b't', 0x02];
        let ops = decode_bytecode(&bytes).unwrap();
        assert_eq!(ops, alloc::vec![Op::Ffi("test".into(), 2)]);
    }

    #[test]
    fn decode_unknown_tag() {
        let result = decode_bytecode(&[0xFF]);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::UnknownOpcode(0xFF) => {}
            other => panic!("expected UnknownOpcode(0xFF), got {:?}", other),
        }
    }

    #[test]
    fn decode_truncated() {
        // PushNum without enough bytes
        let result = decode_bytecode(&[0x15, 0x00, 0x00]);
        assert!(result.is_err());
    }

    #[test]
    fn roundtrip_rust_encoder_decoder() {
        let ops = alloc::vec![
            Op::PushNum(42.0),
            Op::Store("x".into()),
            Op::LoadLocal("x".into()),
            Op::Emit,
            Op::Halt,
        ];
        let bytes = encode_bytecode(&ops);
        let decoded = decode_bytecode(&bytes).unwrap();
        assert_eq!(decoded, ops);
    }

    #[test]
    fn roundtrip_complex() {
        let ops = alloc::vec![
            Op::PushNum(3.14),
            Op::Store("pi".into()),
            Op::Jmp(10),
            Op::Jz(20),
            Op::Call("my_fn".into()),
            Op::Ret,
            Op::Loop(5),
            Op::ScopeBegin,
            Op::LoadLocal("pi".into()),
            Op::Emit,
            Op::ScopeEnd,
            Op::PushMol(1, 2, 128, 128, 3),
            Op::Edge(5),
            Op::Query(2),
            Op::Lca,
            Op::Dup,
            Op::Pop,
            Op::Swap,
            Op::Fuse,
            Op::Nop,
            Op::Dream,
            Op::Stats,
            Op::Trace,
            Op::Inspect,
            Op::Assert,
            Op::TypeOf,
            Op::Why,
            Op::Explain,
            Op::StoreUpdate("x".into()),
            Op::TryBegin(50),
            Op::CatchEnd,
            Op::Ffi("extern_fn".into(), 3),
            Op::Halt,
        ];
        let bytes = encode_bytecode(&ops);
        let decoded = decode_bytecode(&bytes).unwrap();
        assert_eq!(decoded, ops);
    }

    #[test]
    fn roundtrip_let_x_42() {
        // The canonical "let x = 42;" sequence
        let ops = alloc::vec![
            Op::PushNum(42.0),
            Op::Store("x".into()),
            Op::Halt,
        ];
        let bytes = encode_bytecode(&ops);
        let decoded = decode_bytecode(&bytes).unwrap();
        assert_eq!(decoded, ops);

        // Verify byte-level encoding:
        // PushNum: [0x15][f64 LE of 42.0]
        assert_eq!(bytes[0], 0x15);
        let n = f64::from_le_bytes(bytes[1..9].try_into().unwrap());
        assert_eq!(n, 42.0);
        // Store: [0x13][0x01]['x']
        assert_eq!(bytes[9], 0x13);
        assert_eq!(bytes[10], 1); // name length
        assert_eq!(bytes[11], b'x');
        // Halt: [0x0F]
        assert_eq!(bytes[12], 0x0F);
        assert_eq!(bytes.len(), 13);
    }
}
