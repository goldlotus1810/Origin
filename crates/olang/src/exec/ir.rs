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
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::molecular::MolecularChain;

/// Builtin IDs for Op::CallBuiltin — O(1) dispatch table.
/// Each ID maps to an inlined handler in the VM main loop.
pub(crate) const BID_EQ: u8 = 0;
pub(crate) const BID_CMP_LT: u8 = 1;
pub(crate) const BID_CMP_GT: u8 = 2;
pub(crate) const BID_CMP_LE: u8 = 3;
pub(crate) const BID_CMP_GE: u8 = 4;
pub(crate) const BID_CMP_NE: u8 = 5;
pub(crate) const BID_HYP_ADD: u8 = 6;
pub(crate) const BID_HYP_SUB: u8 = 7;
pub(crate) const BID_HYP_MUL: u8 = 8;
pub(crate) const BID_HYP_DIV: u8 = 9;
pub(crate) const BID_LOGIC_NOT: u8 = 10;
pub(crate) const BID_ASSERT_TRUTH: u8 = 11;
pub(crate) const BID_ARRAY_NEW: u8 = 12;
pub(crate) const BID_ARRAY_GET: u8 = 13;
pub(crate) const BID_ARRAY_LEN: u8 = 14;
pub(crate) const BID_ARRAY_PUSH: u8 = 15;
pub(crate) const BID_DICT_NEW: u8 = 16;
pub(crate) const BID_DICT_GET: u8 = 17;
pub(crate) const BID_DICT_SET: u8 = 18;
pub(crate) const BID_STR_CHAR_AT: u8 = 19;
pub(crate) const BID_STR_SUBSTR: u8 = 20;
pub(crate) const BID_STR_LEN: u8 = 21;
pub(crate) const BID_STR_CONCAT: u8 = 22;
pub(crate) const BID_TO_STRING: u8 = 23;
pub(crate) const BID_TO_NUM: u8 = 24;
pub(crate) const BID_STR_IS_KEYWORD: u8 = 25;
pub(crate) const BID_PHYS_ADD: u8 = 26;
pub(crate) const BID_PHYS_SUB: u8 = 27;
pub(crate) const BID_HYP_MOD: u8 = 28;
pub(crate) const BID_CHAIN_LEN: u8 = 29;
pub(crate) const BID_TYPE_OF: u8 = 30;

/// Map builtin name → ID. Returns None for non-builtin or user functions.
/// Only maps builtins that have INLINED handlers in CallBuiltin dispatch.
pub fn builtin_name_to_id(name: &str) -> Option<u8> {
    match name {
        // Arithmetic — inlined
        "__hyp_add" | "__phys_add" => Some(BID_HYP_ADD),
        "__hyp_sub" | "__phys_sub" => Some(BID_HYP_SUB),
        "__hyp_mul" => Some(BID_HYP_MUL),
        "__hyp_div" => Some(BID_HYP_DIV),
        // "__hyp_mod" not mapped — ASM doesn't handle it
        // Comparison — inlined
        "__eq" => Some(BID_EQ),
        "__cmp_lt" => Some(BID_CMP_LT),
        "__cmp_gt" => Some(BID_CMP_GT),
        "__cmp_le" => Some(BID_CMP_LE),
        "__cmp_ge" => Some(BID_CMP_GE),
        "__cmp_ne" => Some(BID_CMP_NE),
        // Logic — inlined
        "__logic_not" => Some(BID_LOGIC_NOT),
        "__assert_truth" => Some(BID_ASSERT_TRUTH),
        // String ops stay as Op::Call(String) — ASM handles them via op_call hash dispatch
        // "__str_char_at" / "__str_substr" / "__str_is_keyword" NOT mapped here
        // because ASM CallBuiltin noop-skips unhandled IDs → stack corruption
        // Everything else stays as Op::Call(String) for now
        _ => None,
    }
}

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
    /// Call builtin by ID — O(1) dispatch instead of string matching.
    /// Top 32 most frequent builtins use this for performance.
    CallBuiltin(u8),
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

    /// Push 1-molecule chain from packed u16.
    /// v2: `{ S=1 R=2 V=4 A=4 T=2 }` → Molecule::pack() → u16 → MolecularChain.
    /// Bytecode: [0x19][lo][hi] = 3 bytes.
    PushMol(u16),
    /// Try: begin error-catching block. If any VmError occurs before
    /// the matching CatchEnd, jump to the catch handler instead of halting.
    TryBegin(usize),
    /// CatchEnd: end of catch handler, marks the resume point after try/catch.
    CatchEnd,
    /// Pop top, update existing variable searching ALL scopes (for reassignment).
    /// Unlike Store which always writes to innermost scope, StoreUpdate
    /// searches from innermost outward and updates the first match.
    StoreUpdate(String),

    // ── Device I/O ────────────────────────────────────────────────────────────
    // Bridge Olang → Hardware: VM emit VmEvent, Runtime gọi HAL.
    //
    // Tại sao cần opcodes riêng?
    //   C driver: 500 dòng (struct, init, read, write, interrupt, error handling)
    //   Olang:    1 opcode + device_id → Runtime gọi HAL → phần cứng thật
    //
    // ○{💡}.evolve(V, 0xFF) → DeviceWrite("light_0", 0xFF) → HAL.device_write()

    /// Ghi giá trị ra thiết bị. Pop 1 chain (value), device_id = tham số.
    /// VM emit DeviceWrite event → Runtime gọi HAL.device_write().
    DeviceWrite(String),

    /// Đọc giá trị từ thiết bị. Push kết quả lên stack (f32 → chain).
    /// VM emit DeviceRead event → Runtime gọi HAL.device_read() → push chain.
    DeviceRead(String),

    /// Liệt kê thiết bị có sẵn. Push danh sách device_id lên stack.
    /// VM emit DeviceList event → Runtime gọi HAL.scan_devices().
    DeviceList,

    // ── FFI & System I/O ──────────────────────────────────────────────────────
    // Olang gọi hàm bên ngoài (Rust/C) và tương tác hệ thống.
    //
    // FFI: gọi extern function bằng tên → VM emit FfiCall event
    // FileRead/FileWrite: đọc/ghi file → VM emit event → Runtime dùng HAL
    // Spawn: tạo task mới → VM emit SpawnRequest → Runtime tạo ISL message

    /// Gọi foreign function. Pop N args (theo arity), push kết quả.
    /// VM emit FfiCall event → Runtime dispatch → extern fn → push result.
    Ffi(String, u8), // (function_name, arity)

    /// Đọc file. Pop 1 chain (path), push nội dung lên stack.
    FileRead,
    /// Ghi file. Pop 2 chains (path, data).
    FileWrite,
    /// Append file. Pop 2 chains (path, data). QT9: Append-only.
    FileAppend,

    /// Spawn concurrent task. Marks begin of async block.
    /// VM emit SpawnRequest event → Runtime tạo ISL message → Worker/Chief xử lý.
    SpawnBegin,
    /// End of spawn block.
    SpawnEnd,

    /// Create closure: jump over body, push closure marker onto stack.
    /// `Closure(param_count, body_len)` — body follows immediately after this op.
    /// The closure captures the current local scope by value.
    Closure(u8, usize),
    /// Call a closure: pop closure + args from stack, execute body.
    CallClosure(u8),

    // ── Channel concurrency (Phase 4) ──────────────────────────────────────

    /// Create new channel. Push channel_id onto stack.
    ChanNew,
    /// Pop value + channel_id from stack. Send value to channel.
    ChanSend,
    /// Pop channel_id from stack. Receive value (non-blocking: empty if no msg).
    ChanRecv,
    /// Select: multi-channel wait. Arms encoded as sequence:
    ///   For each recv arm: ChanRecv + body + Jmp(end)
    ///   Timeout arm: PushNum(ms) + body
    /// `Select(arm_count)` — number of arms (including optional timeout)
    Select(u8),
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
            Self::CallBuiltin(_) => "CALL_BUILTIN",
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
            Self::PushMol(..) => "PUSH_MOL",
            Self::TryBegin(_) => "TRY_BEGIN",
            Self::CatchEnd => "CATCH_END",
            Self::StoreUpdate(_) => "STORE_UPDATE",
            Self::DeviceWrite(_) => "DEVICE_WRITE",
            Self::DeviceRead(_) => "DEVICE_READ",
            Self::DeviceList => "DEVICE_LIST",
            Self::Ffi(..) => "FFI",
            Self::FileRead => "FILE_READ",
            Self::FileWrite => "FILE_WRITE",
            Self::FileAppend => "FILE_APPEND",
            Self::SpawnBegin => "SPAWN_BEGIN",
            Self::SpawnEnd => "SPAWN_END",
            Self::Closure(..) => "CLOSURE",
            Self::CallClosure(_) => "CALL_CLOSURE",
            Self::ChanNew => "CHAN_NEW",
            Self::ChanSend => "CHAN_SEND",
            Self::ChanRecv => "CHAN_RECV",
            Self::Select(_) => "SELECT",
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
                let cb = c.to_tagged_bytes();
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
            Self::CallBuiltin(id) => alloc::vec![0x3A, *id],
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
            Self::PushMol(bits) => {
                let b = bits.to_le_bytes();
                alloc::vec![0x36, b[0], b[1]]
            }
            Self::TryBegin(target) => {
                let mut b = alloc::vec![0x37];
                b.extend_from_slice(&(*target as u32).to_le_bytes());
                b
            }
            Self::CatchEnd => alloc::vec![0x38],
            Self::StoreUpdate(s) => {
                let sb = s.as_bytes();
                let mut b = alloc::vec![0x39, sb.len() as u8];
                b.extend_from_slice(sb);
                b
            }
            Self::DeviceWrite(id) => {
                let sb = id.as_bytes();
                let mut b = alloc::vec![0x40, sb.len() as u8];
                b.extend_from_slice(sb);
                b
            }
            Self::DeviceRead(id) => {
                let sb = id.as_bytes();
                let mut b = alloc::vec![0x41, sb.len() as u8];
                b.extend_from_slice(sb);
                b
            }
            Self::DeviceList => alloc::vec![0x42],
            Self::Ffi(name, arity) => {
                let sb = name.as_bytes();
                let mut b = alloc::vec![0x50, sb.len() as u8, *arity];
                b.extend_from_slice(sb);
                b
            }
            Self::FileRead => alloc::vec![0x51],
            Self::FileWrite => alloc::vec![0x52],
            Self::FileAppend => alloc::vec![0x53],
            Self::SpawnBegin => alloc::vec![0x60],
            Self::SpawnEnd => alloc::vec![0x61],
            Self::Closure(params, body_len) => {
                let mut v = alloc::vec![0x70, *params];
                v.extend_from_slice(&(*body_len as u32).to_le_bytes());
                v
            }
            Self::CallClosure(arity) => alloc::vec![0x71, *arity],
            Self::ChanNew => alloc::vec![0x80],
            Self::ChanSend => alloc::vec![0x81],
            Self::ChanRecv => alloc::vec![0x82],
            Self::Select(n) => alloc::vec![0x83, *n],
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
    /// ○{1 + 2} — arithmetic (QT3: giả thuyết)
    /// Compiles to: PushNum(lhs), PushNum(rhs), Call("__hyp_<op>")
    Arithmetic {
        lhs: f64,
        /// "__hyp_add", "__hyp_sub", "__hyp_mul", "__hyp_div"
        builtin: String,
        rhs: f64,
    },
    /// { S=1 R=6 V=200 A=180 T=4 } — molecular literal → PushMol(u16) opcode
    MolecularLiteral {
        shape: u8,
        relation: u8,
        valence: u8,
        arousal: u8,
        time: u8,
    },
    /// let x = <expr> — variable binding → emit value + Store(name)
    LetBinding {
        name: String,
        value: alloc::boxed::Box<OlangIrExpr>,
    },
    /// if <cond> { <then> } else { <else> } — conditional branch
    IfElse {
        condition: alloc::boxed::Box<OlangIrExpr>,
        then_branch: Vec<OlangIrExpr>,
        else_branch: Vec<OlangIrExpr>,
    },
    /// loop N { <body> } — repeat N times
    LoopBlock {
        count: u32,
        body: Vec<OlangIrExpr>,
    },
    /// fn name { <body> } — function definition (inline block)
    FnDef {
        name: String,
        body: Vec<OlangIrExpr>,
    },
    /// spawn { <body> } — concurrent execution (Go-style: emit body as async task)
    Spawn {
        body: Vec<OlangIrExpr>,
    },
    /// expr |> expr |> expr — pipe chain (Julia-style: output of each feeds into next)
    Pipe(Vec<OlangIrExpr>),
    /// use <module> — import skill/module (Python-style)
    Use(String),
    /// emit <expr> — explicit output
    EmitExpr(alloc::boxed::Box<OlangIrExpr>),
    /// return <expr> — return from function
    ReturnExpr(alloc::boxed::Box<OlangIrExpr>),
    /// match <expr> { pattern => { body }, _ => { body } }
    Match {
        subject: alloc::boxed::Box<OlangIrExpr>,
        /// (pattern_name, body) — pattern is type name or "_" for wildcard
        arms: Vec<(String, Vec<OlangIrExpr>)>,
    },
    /// try { body } catch { handler } — error recovery
    TryCatch {
        try_body: Vec<OlangIrExpr>,
        catch_body: Vec<OlangIrExpr>,
    },
    /// for var in start..end { body } — range iteration
    ForIn {
        var: String,
        start: u32,
        end: u32,
        body: Vec<OlangIrExpr>,
    },
    /// while cond { body } — conditional loop (QT2: capped at 1024)
    While {
        cond: Box<OlangIrExpr>,
        body: Vec<OlangIrExpr>,
    },
    /// x < 10 — comparison → PushNum(lhs), PushNum(rhs), Call("__cmp_*")
    Compare {
        lhs: Box<OlangIrExpr>,
        /// "__cmp_lt", "__cmp_gt", "__cmp_le", "__cmp_ge"
        builtin: String,
        rhs: Box<OlangIrExpr>,
    },

    // ── Device I/O expressions ────────────────────────────────────────────────

    /// device "relay_0" write <expr> — ghi giá trị ra thiết bị
    /// Compile: emit_expr(value) → DeviceWrite(device_id)
    DeviceWriteExpr {
        /// ID thiết bị (VD: "gpio_relay", "light_0")
        device_id: String,
        /// Giá trị ghi (sẽ convert thành u8 molecular dimension)
        value: Box<OlangIrExpr>,
    },

    /// device "dht22" read — đọc giá trị từ thiết bị
    /// Compile: DeviceRead(device_id) → push f32 chain lên stack
    DeviceReadExpr {
        /// ID thiết bị
        device_id: String,
    },

    /// device list — liệt kê thiết bị
    DeviceListExpr,
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

        OlangIrExpr::Arithmetic { lhs, builtin, rhs } => {
            prog.push_op(Op::PushNum(*lhs));
            prog.push_op(Op::PushNum(*rhs));
            prog.push_op(Op::Call(builtin.clone()));
        }

        OlangIrExpr::MolecularLiteral {
            shape,
            relation,
            valence,
            arousal,
            time,
        } => {
            let packed = crate::molecular::Molecule::pack(*shape, *relation, *valence, *arousal, *time).bits;
            prog.push_op(Op::PushMol(packed));
        }

        OlangIrExpr::LetBinding { name, value } => {
            emit_expr(value, prog);
            prog.push_op(Op::Store(name.clone()));
        }

        OlangIrExpr::IfElse {
            condition,
            then_branch,
            else_branch,
        } => {
            // Emit condition → stack top
            emit_expr(condition, prog);
            // Jz to else/end (placeholder, patch later)
            let jz_idx = prog.ops.len();
            prog.push_op(Op::Jz(0)); // placeholder

            // Then branch
            prog.push_op(Op::ScopeBegin);
            for e in then_branch {
                emit_expr(e, prog);
            }
            prog.push_op(Op::ScopeEnd);

            if else_branch.is_empty() {
                // Patch Jz → jump past then
                let end = prog.ops.len();
                prog.ops[jz_idx] = Op::Jz(end);
            } else {
                // Jmp past else (placeholder)
                let jmp_idx = prog.ops.len();
                prog.push_op(Op::Jmp(0)); // placeholder

                // Patch Jz → jump to else start
                let else_start = prog.ops.len();
                prog.ops[jz_idx] = Op::Jz(else_start);

                // Else branch
                prog.push_op(Op::ScopeBegin);
                for e in else_branch {
                    emit_expr(e, prog);
                }
                prog.push_op(Op::ScopeEnd);

                // Patch Jmp → jump past else
                let end = prog.ops.len();
                prog.ops[jmp_idx] = Op::Jmp(end);
            }
        }

        OlangIrExpr::LoopBlock { count, body } => {
            prog.push_op(Op::Loop(*count));
            prog.push_op(Op::ScopeBegin);
            for e in body {
                emit_expr(e, prog);
            }
            prog.push_op(Op::ScopeEnd);
        }

        OlangIrExpr::FnDef { name, body } => {
            // Jump over the function body (don't execute at definition time)
            let jmp_idx = prog.ops.len();
            prog.push_op(Op::Jmp(0)); // placeholder

            // Function entry point — Call(name) will jump here
            let fn_start = prog.ops.len();
            prog.push_op(Op::ScopeBegin);
            for e in body {
                emit_expr(e, prog);
            }
            prog.push_op(Op::ScopeEnd);
            prog.push_op(Op::Ret);

            // Patch jump over function body
            let after_fn = prog.ops.len();
            prog.ops[jmp_idx] = Op::Jmp(after_fn);

            // Register function name → entry point (store as alias for Call)
            // Use Store to remember fn_start as a named entry
            // For now, emit a Nop — function lookup happens via Call(name)
            let _ = (name, fn_start); // fn table would go here
        }

        OlangIrExpr::Spawn { body } => {
            // Spawn: VM emit SpawnRequest → Runtime tạo ISL task.
            // SpawnBegin/SpawnEnd wrap body cho Runtime collect opcodes.
            prog.push_op(Op::SpawnBegin);
            prog.push_op(Op::ScopeBegin);
            for e in body {
                emit_expr(e, prog);
            }
            prog.push_op(Op::ScopeEnd);
            prog.push_op(Op::SpawnEnd);
        }

        OlangIrExpr::Pipe(exprs) => {
            // Julia-style: each expr's output feeds into next
            // First expr pushes result, subsequent exprs consume + push
            for e in exprs {
                emit_expr(e, prog);
            }
            // Final result is on top of stack
            prog.push_op(Op::Emit);
        }

        OlangIrExpr::Use(module) => {
            // Python-style: load module/skill into scope
            prog.push_op(Op::Load(module.clone()));
        }

        OlangIrExpr::EmitExpr(inner) => {
            emit_expr(inner, prog);
            prog.push_op(Op::Emit);
        }

        OlangIrExpr::ReturnExpr(inner) => {
            emit_expr(inner, prog);
            prog.push_op(Op::Ret);
        }

        OlangIrExpr::Match { subject, arms } => {
            // Evaluate subject
            emit_expr(subject, prog);

            let mut end_jumps: Vec<usize> = Vec::new();

            for (pattern, body) in arms {
                if pattern == "_" {
                    // Wildcard: always execute
                    prog.push_op(Op::ScopeBegin);
                    for e in body {
                        emit_expr(e, prog);
                    }
                    prog.push_op(Op::ScopeEnd);
                    break;
                }
                // Type match: DUP subject, TypeOf, Load pattern, compare
                prog.push_op(Op::Dup);
                prog.push_op(Op::TypeOf);
                prog.push_op(Op::Load(pattern.clone()));
                prog.push_op(Op::Call("__match_type".into()));
                let jz_idx = prog.ops.len();
                prog.push_op(Op::Jz(0)); // placeholder
                prog.push_op(Op::Pop); // pop match result

                prog.push_op(Op::ScopeBegin);
                for e in body {
                    emit_expr(e, prog);
                }
                prog.push_op(Op::ScopeEnd);

                end_jumps.push(prog.ops.len());
                prog.push_op(Op::Jmp(0)); // placeholder

                let next = prog.ops.len();
                prog.ops[jz_idx] = Op::Jz(next);
                prog.push_op(Op::Pop); // pop match result on no-match
            }

            // Pop subject
            prog.push_op(Op::Pop);

            // Patch end jumps
            let end = prog.ops.len();
            for jmp_pos in end_jumps {
                prog.ops[jmp_pos] = Op::Jmp(end);
            }
        }

        OlangIrExpr::TryCatch { try_body, catch_body } => {
            // TryBegin(catch_pc), [try_body], Jmp(end), [catch_body], CatchEnd
            let try_begin_idx = prog.ops.len();
            prog.push_op(Op::TryBegin(0)); // placeholder

            prog.push_op(Op::ScopeBegin);
            for e in try_body {
                emit_expr(e, prog);
            }
            prog.push_op(Op::ScopeEnd);

            let jmp_idx = prog.ops.len();
            prog.push_op(Op::Jmp(0)); // skip catch on success

            // Catch block
            let catch_start = prog.ops.len();
            prog.ops[try_begin_idx] = Op::TryBegin(catch_start);

            prog.push_op(Op::ScopeBegin);
            for e in catch_body {
                emit_expr(e, prog);
            }
            prog.push_op(Op::ScopeEnd);
            prog.push_op(Op::CatchEnd);

            let end = prog.ops.len();
            prog.ops[jmp_idx] = Op::Jmp(end);
        }

        OlangIrExpr::ForIn { var, start, end, body } => {
            // Counter lives on stack; each iteration DUP into scoped var.
            let count = end.saturating_sub(*start);
            prog.push_op(Op::PushNum(*start as f64));
            if count > 0 {
                prog.push_op(Op::Loop(count));
                prog.push_op(Op::ScopeBegin);
                prog.push_op(Op::Dup);
                prog.push_op(Op::Store(var.clone()));
                for e in body {
                    emit_expr(e, prog);
                }
                // Increment counter on stack
                prog.push_op(Op::PushNum(1.0));
                prog.push_op(Op::Call("__hyp_add".into()));
                prog.push_op(Op::ScopeEnd);
            }
            prog.push_op(Op::Pop);
        }

        OlangIrExpr::While { cond, body } => {
            // QT2: ∞-1 — capped at 1024 iterations
            // Layout: Loop(1024) ScopeBegin [cond] Jz(end) Pop [body] ScopeEnd [end:] Pop
            prog.push_op(Op::Loop(1024));
            prog.push_op(Op::ScopeBegin);
            emit_expr(cond, prog);
            let jz_idx = prog.ops.len();
            prog.push_op(Op::Jz(0)); // placeholder
            prog.push_op(Op::Pop); // pop cond (true path)
            for e in body {
                emit_expr(e, prog);
            }
            prog.push_op(Op::ScopeEnd); // loop jump-back
            let end = prog.ops.len();
            prog.ops[jz_idx] = Op::Jz(end);
            prog.push_op(Op::Pop); // pop cond (false path)
        }

        OlangIrExpr::Compare { lhs, builtin, rhs } => {
            emit_expr(lhs, prog);
            emit_expr(rhs, prog);
            prog.push_op(Op::Call(builtin.clone()));
        }

        // ── Device I/O ────────────────────────────────────────────────────────

        OlangIrExpr::DeviceWriteExpr { device_id, value } => {
            emit_expr(value, prog);
            prog.push_op(Op::DeviceWrite(device_id.clone()));
        }

        OlangIrExpr::DeviceReadExpr { device_id } => {
            prog.push_op(Op::DeviceRead(device_id.clone()));
        }

        OlangIrExpr::DeviceListExpr => {
            prog.push_op(Op::DeviceList);
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

    #[test]
    fn compile_if_then() {
        let expr = OlangIrExpr::IfElse {
            condition: alloc::boxed::Box::new(OlangIrExpr::Query("fire".into())),
            then_branch: alloc::vec![OlangIrExpr::Command("stats".into())],
            else_branch: alloc::vec![],
        };
        let prog = compile_expr(&expr);
        // Should contain: Load fire, Jz(end), ScopeBegin, Stats, ScopeEnd, Emit, Halt
        assert!(prog.ops.contains(&Op::Load("fire".into())));
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Jz(_))));
        assert!(prog.ops.contains(&Op::ScopeBegin));
        assert!(prog.ops.contains(&Op::Stats));
        assert!(prog.ops.contains(&Op::ScopeEnd));
    }

    #[test]
    fn compile_if_else() {
        let expr = OlangIrExpr::IfElse {
            condition: alloc::boxed::Box::new(OlangIrExpr::Query("fire".into())),
            then_branch: alloc::vec![OlangIrExpr::Command("stats".into())],
            else_branch: alloc::vec![OlangIrExpr::Command("dream".into())],
        };
        let prog = compile_expr(&expr);
        // Should contain: Load, Jz, ScopeBegin, Stats, ScopeEnd, Jmp, ScopeBegin, Dream, ScopeEnd
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Jz(_))));
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Jmp(_))));
        assert!(prog.ops.contains(&Op::Stats));
        assert!(prog.ops.contains(&Op::Dream));
    }

    #[test]
    fn compile_loop_block() {
        let expr = OlangIrExpr::LoopBlock {
            count: 3,
            body: alloc::vec![OlangIrExpr::Command("stats".into())],
        };
        let prog = compile_expr(&expr);
        assert!(prog.ops.contains(&Op::Loop(3)));
        assert!(prog.ops.contains(&Op::ScopeBegin));
        assert!(prog.ops.contains(&Op::Stats));
        assert!(prog.ops.contains(&Op::ScopeEnd));
    }

    #[test]
    fn compile_fn_def() {
        let expr = OlangIrExpr::FnDef {
            name: "test".into(),
            body: alloc::vec![OlangIrExpr::Command("stats".into())],
        };
        let prog = compile_expr(&expr);
        // Should contain: Jmp(skip fn), ScopeBegin, Stats, ScopeEnd, Ret
        assert!(prog.ops.iter().any(|op| matches!(op, Op::Jmp(_))));
        assert!(prog.ops.contains(&Op::ScopeBegin));
        assert!(prog.ops.contains(&Op::Stats));
        assert!(prog.ops.contains(&Op::Ret));
    }

    #[test]
    fn compile_let_binding() {
        let expr = OlangIrExpr::LetBinding {
            name: "x".into(),
            value: alloc::boxed::Box::new(OlangIrExpr::Query("fire".into())),
        };
        let prog = compile_expr(&expr);
        assert!(prog.ops.contains(&Op::Load("fire".into())));
        assert!(prog.ops.contains(&Op::Store("x".into())));
    }

    #[test]
    fn compile_device_write() {
        let expr = OlangIrExpr::DeviceWriteExpr {
            device_id: "relay_0".into(),
            value: alloc::boxed::Box::new(OlangIrExpr::MolecularLiteral {
                shape: 1, relation: 1, valence: 0xFF, arousal: 0x80, time: 1,
            }),
        };
        let prog = compile_expr(&expr);
        assert!(prog.ops.iter().any(|op| matches!(op, Op::DeviceWrite(id) if id == "relay_0")));
    }

    #[test]
    fn compile_device_read() {
        let expr = OlangIrExpr::DeviceReadExpr {
            device_id: "dht22".into(),
        };
        let prog = compile_expr(&expr);
        assert!(prog.ops.iter().any(|op| matches!(op, Op::DeviceRead(id) if id == "dht22")));
    }

    #[test]
    fn compile_device_list() {
        let expr = OlangIrExpr::DeviceListExpr;
        let prog = compile_expr(&expr);
        assert!(prog.ops.contains(&Op::DeviceList));
    }

    #[test]
    fn device_opcodes_serialize() {
        let ops = alloc::vec![
            Op::DeviceWrite("relay_0".into()),
            Op::DeviceRead("dht22".into()),
            Op::DeviceList,
        ];
        for op in &ops {
            let b = op.to_bytes();
            assert!(!b.is_empty(), "{} serialize phải non-empty", op.name());
        }
        assert_eq!(ops[0].to_bytes()[0], 0x40);
        assert_eq!(ops[1].to_bytes()[0], 0x41);
        assert_eq!(ops[2].to_bytes()[0], 0x42);
    }

    #[test]
    fn device_opcode_names() {
        assert_eq!(Op::DeviceWrite("x".into()).name(), "DEVICE_WRITE");
        assert_eq!(Op::DeviceRead("x".into()).name(), "DEVICE_READ");
        assert_eq!(Op::DeviceList.name(), "DEVICE_LIST");
    }

    #[test]
    fn compile_molecular_literal() {
        let expr = OlangIrExpr::MolecularLiteral {
            shape: 1,
            relation: 6,
            valence: 200,
            arousal: 180,
            time: 4,
        };
        let prog = compile_expr(&expr);
        assert!(prog.ops.contains(&Op::PushMol(crate::molecular::Molecule::pack(1, 6, 200, 180, 4).bits)));
    }
}
