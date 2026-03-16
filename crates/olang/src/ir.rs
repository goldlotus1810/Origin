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
//! PUSH_NUM f64       → push numeric chain (4-molecule encoding)
//! FUSE               → pop 1, check chain for infinite loops (QT2: ∞-1)
//! TRACE              → toggle execution tracing
//! INSPECT            → pop 1, emit chain structure info
//! ASSERT             → pop 1, error if empty
//! TYPEOF             → pop 1, emit type classification
//! WHY                → pop 2, explain connection between chains
//! EXPLAIN            → pop 1, trace chain's origin

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

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
    /// Pop top, store vào biến cục bộ (cho `let` binding)
    Store(String),
    /// Push biến cục bộ lên stack
    LoadLocal(String),
    /// Push numeric literal → encode as 4-molecule chain
    PushNum(f64),
    /// QT2: FUSE — pop 1 chain, kiểm tra không có vòng lặp vô hạn trong DNA.
    /// Nếu chain tham chiếu chính nó (cycle) → push empty chain (∞ = sai).
    /// Nếu chain hữu hạn → push lại chain (∞-1 = đúng).
    Fuse,

    /// Push new scope frame (for blocks: fn body, if branches, loops)
    ScopeBegin,
    /// Pop scope frame, discarding locals defined in this scope
    ScopeEnd,

    // ── Reasoning & Debug primitives ────────────────────────────────────────
    /// Trace: bật/tắt execution tracing (mỗi bước emit TraceStep event)
    Trace,
    /// Inspect: pop 1 chain, emit InspectChain event (hiển thị cấu trúc bên trong)
    Inspect,
    /// Assert: pop 1 chain, nếu empty → emit AssertFailed event
    Assert,
    /// TypeOf: pop 1 chain, emit TypeInfo event (phân loại: SDF/MATH/EMOTICON/MUSICAL/Mixed)
    TypeOf,
    /// Why: pop 2 chains, tìm đường kết nối giữa chúng (qua Silk)
    Why,
    /// Explain: pop 1 chain, truy ngược nguồn gốc (tại sao chain này tồn tại)
    Explain,
}

impl Op {
    /// Tên opcode (cho debugging).
    pub fn name(&self) -> &'static str {
        match self {
            Self::Push(_) => "PUSH",
            Self::Load(_) => "LOAD",
            Self::Lca => "LCA",
            Self::Edge(_) => "EDGE",
            Self::Query(_) => "QUERY",
            Self::Emit => "EMIT",
            Self::Call(_) => "CALL",
            Self::Ret => "RET",
            Self::Jmp(_) => "JMP",
            Self::Jz(_) => "JZ",
            Self::Dup => "DUP",
            Self::Pop => "POP",
            Self::Swap => "SWAP",
            Self::Loop(_) => "LOOP",
            Self::Halt => "HALT",
            Self::Dream => "DREAM",
            Self::Stats => "STATS",
            Self::Nop => "NOP",
            Self::Store(_) => "STORE",
            Self::LoadLocal(_) => "LOAD_LOCAL",
            Self::PushNum(_) => "PUSH_NUM",
            Self::Fuse => "FUSE",
            Self::ScopeBegin => "SCOPE_BEGIN",
            Self::ScopeEnd => "SCOPE_END",
            Self::Trace => "TRACE",
            Self::Inspect => "INSPECT",
            Self::Assert => "ASSERT",
            Self::TypeOf => "TYPEOF",
            Self::Why => "WHY",
            Self::Explain => "EXPLAIN",
        }
    }

    /// Serialize → bytes (compact).
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Nop => alloc::vec![0x00],
            Self::Lca => alloc::vec![0x01],
            Self::Emit => alloc::vec![0x02],
            Self::Ret => alloc::vec![0x03],
            Self::Dup => alloc::vec![0x04],
            Self::Pop => alloc::vec![0x05],
            Self::Swap => alloc::vec![0x06],
            Self::Halt => alloc::vec![0x07],
            Self::Dream => alloc::vec![0x08],
            Self::Stats => alloc::vec![0x09],
            Self::Edge(r) => alloc::vec![0x10, *r],
            Self::Query(r) => alloc::vec![0x11, *r],
            Self::Loop(n) => {
                let mut b = alloc::vec![0x20];
                b.extend_from_slice(&n.to_le_bytes());
                b
            }
            Self::Jmp(i) => {
                let mut b = alloc::vec![0x21];
                b.extend_from_slice(&(*i as u32).to_le_bytes());
                b
            }
            Self::Jz(i) => {
                let mut b = alloc::vec![0x22];
                b.extend_from_slice(&(*i as u32).to_le_bytes());
                b
            }
            Self::Push(c) => {
                let cb = c.to_bytes();
                let mut b = alloc::vec![0x30, cb.len() as u8];
                b.extend_from_slice(&cb);
                b
            }
            Self::Load(s) => {
                let sb = s.as_bytes();
                let mut b = alloc::vec![0x31, sb.len() as u8];
                b.extend_from_slice(sb);
                b
            }
            Self::Call(s) => {
                let sb = s.as_bytes();
                let mut b = alloc::vec![0x32, sb.len() as u8];
                b.extend_from_slice(sb);
                b
            }
            Self::Store(s) => {
                let sb = s.as_bytes();
                let mut b = alloc::vec![0x33, sb.len() as u8];
                b.extend_from_slice(sb);
                b
            }
            Self::LoadLocal(s) => {
                let sb = s.as_bytes();
                let mut b = alloc::vec![0x34, sb.len() as u8];
                b.extend_from_slice(sb);
                b
            }
            Self::PushNum(n) => {
                let mut b = alloc::vec![0x35];
                b.extend_from_slice(&n.to_le_bytes());
                b
            }
            Self::Fuse => alloc::vec![0x0A],
            Self::ScopeBegin => alloc::vec![0x13],
            Self::ScopeEnd => alloc::vec![0x14],
            Self::Trace => alloc::vec![0x0B],
            Self::Inspect => alloc::vec![0x0C],
            Self::Assert => alloc::vec![0x0D],
            Self::TypeOf => alloc::vec![0x0E],
            Self::Why => alloc::vec![0x0F],
            Self::Explain => alloc::vec![0x12],
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
    pub ops: Vec<Op>,
    pub name: String,
}

#[allow(missing_docs)]
impl OlangProgram {
    pub fn new(name: &str) -> Self {
        Self {
            ops: Vec::new(),
            name: name.into(),
        }
    }

    pub fn push_op(&mut self, op: Op) -> &mut Self {
        self.ops.push(op);
        self
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

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
    Relation {
        subject: String,
        rel: u8,
        object: Option<String>,
    },
    /// Direct chain push (e.g. ZWJ sequence already encoded)
    Push(crate::molecular::MolecularChain),
    /// ZWJ sequence: preserve original codepoints for display
    ZwjDisplay {
        original: alloc::string::String,
        chain: crate::molecular::MolecularChain,
    },
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

        OlangIrExpr::Relation {
            subject,
            rel,
            object,
        } => {
            prog.push_op(Op::Load(subject.clone()));
            if let Some(obj) = object {
                prog.push_op(Op::Load(obj.clone()));
                prog.push_op(Op::Edge(*rel));
            } else {
                prog.push_op(Op::Query(*rel));
            }
        }

        OlangIrExpr::Push(chain) => {
            prog.push_op(Op::Push(chain.clone()));
        }
        OlangIrExpr::ZwjDisplay { chain, .. } => {
            prog.push_op(Op::Push(chain.clone()));
        }

        OlangIrExpr::Command(cmd) => match cmd.as_str() {
            "dream" => {
                prog.push_op(Op::Dream);
            }
            "stats" => {
                prog.push_op(Op::Stats);
            }
            _ => {
                prog.push_op(Op::Nop);
            }
        },

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
            Op::Nop,
            Op::Lca,
            Op::Emit,
            Op::Ret,
            Op::Dup,
            Op::Pop,
            Op::Swap,
            Op::Halt,
            Op::Dream,
            Op::Stats,
            Op::Edge(0x01),
            Op::Query(0x06),
            Op::Loop(3),
            Op::Jmp(0),
            Op::Jz(0),
            Op::Load("x".into()),
            Op::Call("f".into()),
            Op::Store("v".into()),
            Op::LoadLocal("v".into()),
            Op::PushNum(3.14),
            Op::Fuse,
            Op::Trace,
            Op::Inspect,
            Op::Assert,
            Op::TypeOf,
            Op::Why,
            Op::Explain,
        ];
        for op in &ops {
            let b = op.to_bytes();
            assert!(!b.is_empty(), "{} serialize không được empty", op.name());
        }
    }
}
