//! # ir — OlangIR Intermediate Representation
//!
//! Tầng giữa: ○{} → OlangIR → OlangVM hoặc → Target (Rust/WASM/x86)
//!
//! ## Instruction set (minimal, hoàn chỉnh):
//!
//! Stack-based. Mọi thứ là chain.
//!
//! PUSH  chain        → push chain lên stack
//! LOAD  alias        → lookup registry, push chain
//! LCA                → pop 2, push LCA
//! EDGE  rel          → pop 2, kết nối Silk edge
//! QUERY rel          → pop 1, query relation, push results
//! EMIT               → pop 1, output chain
//! CALL  name         → gọi named block
//! RET                → return từ block
//! JMP   label        → jump unconditional
//! JZ    label        → jump nếu top = empty
//! LOOP  n            → lặp n lần
//! HALT               → dừng
//! DREAM              → trigger dream cycle
//! STATS              → emit system stats

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

use crate::molecular::MolecularChain;

// ─────────────────────────────────────────────────────────────────────────────
// Opcode
// ─────────────────────────────────────────────────────────────────────────────

/// OlangIR opcode.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    /// Push chain literal lên stack
    Push(MolecularChain),
    /// Load chain từ alias trong registry
    Load(String),
    /// Pop 2 chains, push LCA(a, b)
    Lca,
    /// Pop 2 chains, tạo Silk edge với relation byte
    Edge(u8),
    /// Pop 1 chain, query tất cả nodes có relation với nó
    Query(u8),
    /// Pop 1 chain, output ra caller
    Emit,
    /// Call named block
    Call(String),
    /// Return từ block hiện tại
    Ret,
    /// Jump đến label (index trong program)
    Jmp(usize),
    /// Jump nếu stack top = empty chain
    Jz(usize),
    /// Duplicate top of stack
    Dup,
    /// Pop và discard top
    Pop,
    /// Swap top 2
    Swap,
    /// Lặp block N lần (N được pop từ stack — không dùng, dùng số literal)
    Loop(u32),
    /// Dừng
    Halt,
    /// Trigger Dream cycle
    Dream,
    /// Emit system stats
    Stats,
    /// No-op
    Nop,
}

impl Op {
    /// Tên opcode (cho debugging).
    pub fn name(&self) -> &'static str {
        match self {
            Self::Push(_)  => "PUSH",
            Self::Load(_)  => "LOAD",
            Self::Lca      => "LCA",
            Self::Edge(_)  => "EDGE",
            Self::Query(_) => "QUERY",
            Self::Emit     => "EMIT",
            Self::Call(_)  => "CALL",
            Self::Ret      => "RET",
            Self::Jmp(_)   => "JMP",
            Self::Jz(_)    => "JZ",
            Self::Dup      => "DUP",
            Self::Pop      => "POP",
            Self::Swap     => "SWAP",
            Self::Loop(_)  => "LOOP",
            Self::Halt     => "HALT",
            Self::Dream    => "DREAM",
            Self::Stats    => "STATS",
            Self::Nop      => "NOP",
        }
    }

    /// Serialize → bytes (compact).
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Nop      => alloc::vec![0x00],
            Self::Lca      => alloc::vec![0x01],
            Self::Emit     => alloc::vec![0x02],
            Self::Ret      => alloc::vec![0x03],
            Self::Dup      => alloc::vec![0x04],
            Self::Pop      => alloc::vec![0x05],
            Self::Swap     => alloc::vec![0x06],
            Self::Halt     => alloc::vec![0x07],
            Self::Dream    => alloc::vec![0x08],
            Self::Stats    => alloc::vec![0x09],
            Self::Edge(r)  => alloc::vec![0x10, *r],
            Self::Query(r) => alloc::vec![0x11, *r],
            Self::Loop(n)  => {
                let mut b = alloc::vec![0x20];
                b.extend_from_slice(&n.to_le_bytes());
                b
            }
            Self::Jmp(i)   => {
                let mut b = alloc::vec![0x21];
                b.extend_from_slice(&(*i as u32).to_le_bytes());
                b
            }
            Self::Jz(i)    => {
                let mut b = alloc::vec![0x22];
                b.extend_from_slice(&(*i as u32).to_le_bytes());
                b
            }
            Self::Push(c)  => {
                let cb = c.to_bytes();
                let mut b = alloc::vec![0x30, cb.len() as u8];
                b.extend_from_slice(&cb);
                b
            }
            Self::Load(s)  => {
                let sb = s.as_bytes();
                let mut b = alloc::vec![0x31, sb.len() as u8];
                b.extend_from_slice(sb);
                b
            }
            Self::Call(s)  => {
                let sb = s.as_bytes();
                let mut b = alloc::vec![0x32, sb.len() as u8];
                b.extend_from_slice(sb);
                b
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// OlangProgram — một chương trình IR
// ─────────────────────────────────────────────────────────────────────────────

/// Một chương trình OlangIR.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct OlangProgram {
    pub ops:  Vec<Op>,
    pub name: String,
}

#[allow(missing_docs)]
impl OlangProgram {
    pub fn new(name: &str) -> Self {
        Self { ops: Vec::new(), name: name.into() }
    }

    pub fn push_op(&mut self, op: Op) -> &mut Self {
        self.ops.push(op);
        self
    }

    pub fn len(&self) -> usize { self.ops.len() }
    pub fn is_empty(&self) -> bool { self.ops.is_empty() }

    /// Serialize toàn bộ program → bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.ops.iter().flat_map(|op| op.to_bytes()).collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Compiler: OlangExpr → OlangProgram
// ─────────────────────────────────────────────────────────────────────────────

/// Compile một ○{} expression sang OlangProgram.
pub fn compile_expr(expr: &OlangIrExpr) -> OlangProgram {
    let mut prog = OlangProgram::new("expr");
    emit_expr(expr, &mut prog);
    prog.push_op(Op::Emit);
    prog.push_op(Op::Halt);
    prog
}

/// IR expression types (từ ○{} parser).
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum OlangIrExpr {
    /// ○{🔥} — query
    Query(String),
    /// ○{🔥 ∘ 💧} — compose (LCA)
    Compose(String, String),
    /// ○{🔥 ∈ ?} — relation query
    Relation { subject: String, rel: u8, object: Option<String> },
    /// ○{dream} — system command
    Command(String),
    /// Pipeline
    Pipeline(Vec<OlangIrExpr>),
}

fn emit_expr(expr: &OlangIrExpr, prog: &mut OlangProgram) {
    match expr {
        OlangIrExpr::Query(name) => {
            prog.push_op(Op::Load(name.clone()));
        }

        OlangIrExpr::Compose(a, b) => {
            prog.push_op(Op::Load(a.clone()));
            prog.push_op(Op::Load(b.clone()));
            prog.push_op(Op::Lca);
        }

        OlangIrExpr::Relation { subject, rel, object } => {
            prog.push_op(Op::Load(subject.clone()));
            if let Some(obj) = object {
                prog.push_op(Op::Load(obj.clone()));
                prog.push_op(Op::Edge(*rel));
            } else {
                prog.push_op(Op::Query(*rel));
            }
        }

        OlangIrExpr::Command(cmd) => {
            match cmd.as_str() {
                "dream" => { prog.push_op(Op::Dream); }
                "stats" => { prog.push_op(Op::Stats); }
                _       => { prog.push_op(Op::Nop);   }
            }
        }

        OlangIrExpr::Pipeline(exprs) => {
            for e in exprs {
                emit_expr(e, prog);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_names() {
        assert_eq!(Op::Lca.name(), "LCA");
        assert_eq!(Op::Halt.name(), "HALT");
        assert_eq!(Op::Emit.name(), "EMIT");
        assert_eq!(Op::Dream.name(), "DREAM");
    }

    #[test]
    fn op_serialize_nop() {
        let b = Op::Nop.to_bytes();
        assert_eq!(b, alloc::vec![0x00]);
    }

    #[test]
    fn op_serialize_lca() {
        let b = Op::Lca.to_bytes();
        assert_eq!(b, alloc::vec![0x01]);
    }

    #[test]
    fn op_serialize_edge() {
        let b = Op::Edge(0x06).to_bytes();
        assert_eq!(b, alloc::vec![0x10, 0x06]);
    }

    #[test]
    fn op_serialize_loop() {
        let b = Op::Loop(5).to_bytes();
        assert_eq!(b[0], 0x20);
        let n = u32::from_le_bytes(b[1..5].try_into().unwrap());
        assert_eq!(n, 5);
    }

    #[test]
    fn op_serialize_jmp() {
        let b = Op::Jmp(42).to_bytes();
        assert_eq!(b[0], 0x21);
        let idx = u32::from_le_bytes(b[1..5].try_into().unwrap());
        assert_eq!(idx as usize, 42);
    }

    #[test]
    fn op_serialize_load() {
        let b = Op::Load("fire".into()).to_bytes();
        assert_eq!(b[0], 0x31);
        assert_eq!(b[1] as usize, 4); // "fire" = 4 bytes
        assert_eq!(&b[2..], b"fire");
    }

    #[test]
    fn program_empty() {
        let p = OlangProgram::new("test");
        assert!(p.is_empty());
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn program_push_ops() {
        let mut p = OlangProgram::new("test");
        p.push_op(Op::Load("fire".into()))
         .push_op(Op::Load("water".into()))
         .push_op(Op::Lca)
         .push_op(Op::Emit)
         .push_op(Op::Halt);
        assert_eq!(p.len(), 5);
    }

    #[test]
    fn program_to_bytes_non_empty() {
        let mut p = OlangProgram::new("test");
        p.push_op(Op::Lca).push_op(Op::Halt);
        let b = p.to_bytes();
        assert!(!b.is_empty());
        assert_eq!(b[0], 0x01); // LCA
        assert_eq!(b[1], 0x07); // HALT
    }

    #[test]
    fn compile_query() {
        let expr = OlangIrExpr::Query("fire".into());
        let prog = compile_expr(&expr);
        assert_eq!(prog.ops[0], Op::Load("fire".into()));
        assert_eq!(*prog.ops.last().unwrap(), Op::Halt);
    }

    #[test]
    fn compile_compose() {
        let expr = OlangIrExpr::Compose("fire".into(), "water".into());
        let prog = compile_expr(&expr);
        assert_eq!(prog.ops[0], Op::Load("fire".into()));
        assert_eq!(prog.ops[1], Op::Load("water".into()));
        assert_eq!(prog.ops[2], Op::Lca);
        assert_eq!(prog.ops[3], Op::Emit);
        assert_eq!(prog.ops[4], Op::Halt);
    }

    #[test]
    fn compile_command_dream() {
        let expr = OlangIrExpr::Command("dream".into());
        let prog = compile_expr(&expr);
        assert_eq!(prog.ops[0], Op::Dream);
    }

    #[test]
    fn compile_pipeline() {
        let expr = OlangIrExpr::Pipeline(alloc::vec![
            OlangIrExpr::Query("fire".into()),
            OlangIrExpr::Query("water".into()),
        ]);
        let prog = compile_expr(&expr);
        // Pipeline: LOAD fire, LOAD water, EMIT, HALT
        assert!(prog.ops.contains(&Op::Load("fire".into())));
        assert!(prog.ops.contains(&Op::Load("water".into())));
    }

    #[test]
    fn all_opcodes_serialize() {
        // Verify tất cả opcodes serialize không panic
        let ops = alloc::vec![
            Op::Nop, Op::Lca, Op::Emit, Op::Ret, Op::Dup,
            Op::Pop, Op::Swap, Op::Halt, Op::Dream, Op::Stats,
            Op::Edge(0x01), Op::Query(0x06),
            Op::Loop(3), Op::Jmp(0), Op::Jz(0),
            Op::Load("x".into()), Op::Call("f".into()),
        ];
        for op in &ops {
            let b = op.to_bytes();
            assert!(!b.is_empty(), "{} serialize không được empty", op.name());
        }
    }
}
