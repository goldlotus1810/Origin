//! # vm — OlangVM Stack Machine
//!
//! Execute OlangProgram.
//! Stack-based: mọi thứ là MolecularChain.
//!
//! VM không có side effects trực tiếp — trả về Vec<VmEvent>
//! để caller (HomeRuntime) xử lý (ghi registry, trigger dream...).

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ir::{OlangProgram, Op};
use crate::lca::lca;
use crate::molecular::{EmotionDim, Molecule, MolecularChain};

// ─────────────────────────────────────────────────────────────────────────────
// VmEvent — side effects VM muốn thực hiện
// ─────────────────────────────────────────────────────────────────────────────

/// Extract readable text from a string-encoded MolecularChain.
/// String chains use shape=0x02, relation=0x01, with each byte stored in valence.
/// Returns None if the chain doesn't look like a string encoding.
pub fn chain_to_string(chain: &MolecularChain) -> Option<String> {
    if chain.is_empty() {
        return Some(String::new());
    }
    // Check if it looks like a string chain (all shape=0x02, relation=0x01)
    let is_string = chain.0.iter().all(|m| m.shape == 0x02 && m.relation == 0x01);
    if is_string {
        let s: String = chain.0.iter()
            .map(|m| m.emotion.valence as char)
            .collect();
        Some(s)
    } else {
        None
    }
}

/// Encode a string as a MolecularChain (each byte → 1 molecule).
/// Inverse of chain_to_string.
pub fn string_to_chain(s: &str) -> MolecularChain {
    let mols: Vec<Molecule> = s.bytes().map(|b| Molecule {
        shape: 0x02,
        relation: 0x01,
        emotion: EmotionDim { valence: b, arousal: 0 },
        time: 0x01,
    }).collect();
    MolecularChain(mols)
}

/// Format a chain for human-readable display.
/// Tries string decoding first, then number, then raw molecule info.
pub fn format_chain_display(chain: &MolecularChain) -> String {
    if chain.is_empty() {
        return "(empty)".into();
    }
    // Try string
    if let Some(s) = chain_to_string(chain) {
        return s;
    }
    // Try number
    if let Some(n) = chain.to_number() {
        return if n == (n as i64 as f64) {
            alloc::format!("{}", n as i64)
        } else {
            alloc::format!("{}", n)
        };
    }
    // Fallback: molecule count + hash
    alloc::format!("[chain: {} molecules, hash={:#x}]", chain.len(), chain.chain_hash())
}

/// Event từ VM → caller xử lý.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum VmEvent {
    /// Output chain (từ EMIT)
    Output(MolecularChain),
    /// Cần lookup alias trong Registry
    LookupAlias(String),
    /// Cần tạo Silk edge
    CreateEdge { from: u64, to: u64, rel: u8 },
    /// Cần query relation
    QueryRelation { hash: u64, rel: u8 },
    /// Trigger Dream cycle
    TriggerDream,
    /// Request system stats
    RequestStats,
    /// Error trong VM
    Error(VmError),
    // ── Reasoning & Debug ────────────────────────────────────────────────
    /// Execution trace step (opcode name, stack depth, pc)
    TraceStep {
        op_name: &'static str,
        stack_depth: usize,
        pc: usize,
    },
    /// Inspect chain structure (molecules count, hash, byte size)
    InspectChain {
        hash: u64,
        molecule_count: usize,
        byte_size: usize,
        is_empty: bool,
    },
    /// Assert failed — chain was empty
    AssertFailed,
    /// Type classification of a chain's molecules
    TypeInfo {
        hash: u64,
        classification: String,
    },
    /// Why: explain connection between two chains
    WhyConnection { from: u64, to: u64 },
    /// Explain: trace origin of a chain
    ExplainOrigin { hash: u64 },

    // ── Device I/O events ────────────────────────────────────────────────────
    // VM emit — Runtime xử lý → gọi HAL → phần cứng thật.

    /// Ghi giá trị ra thiết bị. Runtime gọi HAL.device_write().
    DeviceWrite {
        /// Device ID (VD: "gpio_relay", "light_0")
        device_id: String,
        /// Giá trị ghi (molecular dimension: 0x00=off, 0xFF=max)
        value: u8,
    },
    /// Đọc giá trị từ thiết bị. Runtime gọi HAL.device_read().
    DeviceRead {
        /// Device ID
        device_id: String,
    },
    /// Liệt kê thiết bị. Runtime gọi HAL.scan_devices().
    DeviceListRequest,

    // ── FFI & System I/O events ──────────────────────────────────────────────

    /// Gọi foreign function. Runtime dispatch → extern fn → inject result.
    FfiCall {
        /// Function name (VD: "gpio_write", "http_get")
        name: String,
        /// Arguments (popped from stack before event)
        args: Vec<MolecularChain>,
    },
    /// Đọc file. Runtime gọi HAL.read_file().
    FileReadRequest {
        /// File path (extracted from chain)
        path: String,
    },
    /// Ghi file. Runtime gọi HAL.write_file().
    FileWriteRequest {
        /// File path
        path: String,
        /// Data to write
        data: Vec<u8>,
    },
    /// Append file. Runtime gọi HAL.write_file() ở chế độ append.
    FileAppendRequest {
        /// File path
        path: String,
        /// Data to append
        data: Vec<u8>,
    },
    /// Spawn request. Runtime tạo ISL message hoặc async task.
    SpawnRequest {
        /// Number of opcodes in the spawned block
        body_ops_count: usize,
    },
    /// Module import request. Runtime loads + executes the module file.
    UseModule {
        /// Module name or path
        name: String,
    },
    /// Selective module import: load specific symbols from a module.
    UseModuleSelective {
        /// Module name or path
        name: String,
        /// Specific symbols to import
        imports: Vec<String>,
    },
    /// Module declaration.
    ModDecl {
        /// Module path (dot-separated)
        path: String,
    },
}

// ─────────────────────────────────────────────────────────────────────────────
// VmError
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum VmError {
    StackUnderflow,
    StackOverflow,
    InfiniteLoop,
    InvalidJump(usize),
    MaxStepsExceeded,
    MaxCallDepthExceeded,
    DivisionByZero,
}

impl core::fmt::Display for VmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::StackUnderflow => write!(f, "Stack underflow — pop from empty stack"),
            Self::StackOverflow => write!(f, "Stack overflow — exceeded {} entries", STACK_MAX),
            Self::InfiniteLoop => write!(f, "Infinite loop detected (QT2: ∞ is wrong)"),
            Self::InvalidJump(target) => write!(f, "Invalid jump to position {}", target),
            Self::MaxStepsExceeded => write!(f, "Max steps exceeded ({}) — program too long", STEPS_MAX),
            Self::MaxCallDepthExceeded => write!(f, "Max call depth exceeded — too many nested scopes"),
            Self::DivisionByZero => write!(f, "Division by zero"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// VmStack
// ─────────────────────────────────────────────────────────────────────────────

const STACK_MAX: usize = 256;
const STEPS_MAX: u32 = 65_536;

/// Pop 1 chain từ stack, break nếu underflow.
macro_rules! vm_pop {
    ($stack:expr, $events:expr) => {
        match $stack.pop() {
            Ok(c) => c,
            Err(e) => {
                $events.push(VmEvent::Error(e));
                break;
            }
        }
    };
}

struct VmStack {
    data: Vec<MolecularChain>,
}

impl VmStack {
    fn new() -> Self {
        Self {
            data: Vec::with_capacity(32),
        }
    }

    fn push(&mut self, c: MolecularChain) -> Result<(), VmError> {
        if self.data.len() >= STACK_MAX {
            return Err(VmError::StackOverflow);
        }
        self.data.push(c);
        Ok(())
    }

    fn pop(&mut self) -> Result<MolecularChain, VmError> {
        self.data.pop().ok_or(VmError::StackUnderflow)
    }

    fn peek(&self) -> Option<&MolecularChain> {
        self.data.last()
    }

    fn _is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// OlangVM
// ─────────────────────────────────────────────────────────────────────────────

/// Classify chain by dominant molecule characteristics.
///
/// Maps ShapeBase to Unicode group categories:
/// Split an array-encoded MolecularChain by separator molecules (shape=0, relation=0).
fn split_array_chain(chain: &MolecularChain) -> Vec<MolecularChain> {
    if chain.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut current = Vec::new();
    for mol in &chain.0 {
        if mol.shape == 0 && mol.relation == 0
            && mol.emotion.valence == 0 && mol.emotion.arousal == 0
            && mol.time == 0
        {
            // Separator — finalize current element
            result.push(MolecularChain(core::mem::take(&mut current)));
        } else {
            current.push(*mol);
        }
    }
    // Last element (no trailing separator)
    result.push(MolecularChain(current));
    result
}

/// - SDF shapes (Sphere●, Capsule▬, Box■, Cone▲) → geometric primitives
/// - CSG ops (Torus○, Union∪, Intersect∩, Subtract∖) → mathematical composition
/// - High emotion valence → emoticon-like
///
/// Returns "SDF", "MATH", "EMOTICON", or "Mixed(SDF+MATH)".
fn classify_chain(chain: &MolecularChain) -> String {
    use crate::molecular::ShapeBase;
    if chain.is_empty() {
        return "Empty".into();
    }
    let (mut sdf, mut math, mut emo) = (0u32, 0u32, 0u32);
    for mol in &chain.0 {
        match mol.shape_base() {
            // SDF primitives — geometric shapes
            ShapeBase::Sphere | ShapeBase::Capsule | ShapeBase::Box | ShapeBase::Cone => {
                sdf += 1
            }
            // CSG/Math ops — compositional
            ShapeBase::Torus | ShapeBase::Union | ShapeBase::Intersect | ShapeBase::Subtract => {
                math += 1
            }
        }
        // Extreme valence → emoticon category
        let v = mol.emotion.valence;
        if !(80..=176).contains(&v) {
            emo += 1;
        }
    }
    let total = chain.len() as u32;
    let dominant = [("SDF", sdf), ("MATH", math), ("EMOTICON", emo)];
    let mut parts: Vec<&str> = dominant
        .iter()
        .filter(|(_, c)| *c * 2 >= total) // ≥50% of molecules
        .map(|(name, _)| *name)
        .collect();
    if parts.is_empty() {
        parts.push("Mixed");
    }
    if parts.len() == 1 {
        parts[0].into()
    } else {
        let mut s = String::from("Mixed(");
        for (i, p) in parts.iter().enumerate() {
            if i > 0 {
                s.push('+');
            }
            s.push_str(p);
        }
        s.push(')');
        s
    }
}

/// OlangVM — stack machine thực thi OlangProgram.
pub struct OlangVM {
    /// Max steps để tránh infinite loop (QT2: ∞-1)
    max_steps: u32,
    /// Max call depth để tránh stack overflow từ recursion
    max_call_depth: u32,
}

#[allow(missing_docs)]
impl OlangVM {
    pub fn new() -> Self {
        Self {
            max_steps: STEPS_MAX,
            max_call_depth: 256, // Fib-derived: prevent stack overflow
        }
    }

    pub fn with_max_steps(n: u32) -> Self {
        Self {
            max_steps: n,
            max_call_depth: 256,
        }
    }

    /// Execute program → Vec<VmEvent>.
    ///
    /// VM không access registry trực tiếp.
    /// LOAD → emit LookupAlias event → caller inject chain.
    /// Sau đó caller gọi resume_with(chain) để tiếp tục.
    pub fn execute(&self, prog: &OlangProgram) -> VmResult {
        let mut stack = VmStack::new();
        // Scope stack: each Vec is a scope frame with local variables.
        // ScopeBegin pushes new frame, ScopeEnd pops it.
        // LoadLocal searches from innermost scope outward.
        let mut scopes: Vec<Vec<(alloc::string::String, MolecularChain)>> = Vec::new();
        scopes.push(Vec::new()); // root scope
        let mut events = Vec::new();
        let mut steps = 0u32;
        let mut pc = 0usize;
        let mut call_depth = 0u32;
        // Loop stack: (jump_back_pc, remaining_iterations)
        let mut loop_stack: Vec<(usize, u32)> = Vec::new();
        // Try/catch stack: catch handler PC targets
        let mut try_stack: Vec<usize> = Vec::new();
        // Channel store: id → queue of messages (cooperative concurrency)
        let mut channels: Vec<Vec<MolecularChain>> = Vec::new();
        let mut next_channel_id: u64 = 1;

        while pc < prog.ops.len() {
            if steps >= self.max_steps {
                // If in try block, jump to catch instead of halting
                if let Some(catch_pc) = try_stack.pop() {
                    pc = catch_pc;
                    continue;
                }
                events.push(VmEvent::Error(VmError::MaxStepsExceeded));
                break;
            }
            steps += 1;

            let op = &prog.ops[pc];
            pc += 1;

            match op {
                Op::Nop => {}

                Op::Push(chain) => {
                    if let Err(e) = stack.push(chain.clone()) {
                        events.push(VmEvent::Error(e));
                        break;
                    }
                }

                Op::PushNum(n) => {
                    if let Err(e) = stack.push(MolecularChain::from_number(*n)) {
                        events.push(VmEvent::Error(e));
                        break;
                    }
                }

                Op::PushMol(s, r, v, a, t) => {
                    // Construct 1-molecule chain from explicit dimension values.
                    // Used by LeoAI to express knowledge as Olang code:
                    //   { S=1 R=2 V=128 A=128 T=3 } → Molecule → Chain
                    let mol = Molecule {
                        shape: *s,
                        relation: *r,
                        emotion: EmotionDim {
                            valence: *v,
                            arousal: *a,
                        },
                        time: *t,
                    };
                    let chain = MolecularChain(alloc::vec![mol]);
                    if let Err(e) = stack.push(chain) {
                        events.push(VmEvent::Error(e));
                        break;
                    }
                }

                Op::Load(alias) => {
                    // Emit event — caller sẽ inject chain
                    events.push(VmEvent::LookupAlias(alias.clone()));
                    // Push empty placeholder — real impl dùng callback
                    let _ = stack.push(MolecularChain::empty());
                }

                Op::Lca => {
                    let b = vm_pop!(stack, events);
                    let a = vm_pop!(stack, events);
                    let result = if a.is_empty() || b.is_empty() {
                        if !a.is_empty() {
                            a
                        } else {
                            b
                        }
                    } else {
                        lca(&a, &b)
                    };
                    if let Err(e) = stack.push(result) {
                        events.push(VmEvent::Error(e));
                        break;
                    }
                }

                Op::Edge(rel) => {
                    let b = vm_pop!(stack, events);
                    let a = vm_pop!(stack, events);
                    events.push(VmEvent::CreateEdge {
                        from: a.chain_hash(),
                        to: b.chain_hash(),
                        rel: *rel,
                    });
                    // Giữ lại b trên stack (kết quả của relation)
                    let _ = stack.push(b);
                }

                Op::Query(rel) => {
                    let a = vm_pop!(stack, events);
                    events.push(VmEvent::QueryRelation {
                        hash: a.chain_hash(),
                        rel: *rel,
                    });
                    // Push empty — caller sẽ inject results
                    let _ = stack.push(MolecularChain::empty());
                }

                Op::Emit => {
                    let c = vm_pop!(stack, events);
                    events.push(VmEvent::Output(c));
                }

                Op::Dup => {
                    let c = match stack.peek() {
                        Some(c) => c.clone(),
                        None => {
                            events.push(VmEvent::Error(VmError::StackUnderflow));
                            break;
                        }
                    };
                    let _ = stack.push(c);
                }

                Op::Pop => {
                    let _ = vm_pop!(stack, events);
                }

                Op::Swap => {
                    let b = vm_pop!(stack, events);
                    let a = vm_pop!(stack, events);
                    let _ = stack.push(b);
                    let _ = stack.push(a);
                }

                Op::Jmp(target) => {
                    if *target >= prog.ops.len() {
                        events.push(VmEvent::Error(VmError::InvalidJump(*target)));
                        break;
                    }
                    pc = *target;
                }

                Op::Jz(target) => {
                    let is_empty = stack.peek().map(|c| c.is_empty()).unwrap_or(true);
                    if is_empty {
                        if *target >= prog.ops.len() {
                            events.push(VmEvent::Error(VmError::InvalidJump(*target)));
                            break;
                        }
                        pc = *target;
                    }
                }

                Op::Loop(n) => {
                    // Loop(n): repeat next instruction block n times.
                    // The block runs from pc (current, after Loop opcode) to the next
                    // ScopeEnd or Halt. Uses loop_stack to track remaining iterations
                    // and the jump-back target.
                    // Max iterations capped at 1024 (QT2: ∞-1, không vô hạn).
                    let count = (*n).min(1024);
                    if count > 1 {
                        loop_stack.push((pc, count - 1)); // (jump_back_to, remaining)
                    }
                    // First iteration starts immediately (fall through)
                }

                Op::Call(name) => {
                    // Dispatch built-in functions first, otherwise emit lookup
                    match name.as_str() {
                        "__hyp_add" | "__hyp_sub" | "__hyp_mul" | "__hyp_div"
                        | "__phys_add" | "__phys_sub" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let nb = b.to_number().unwrap_or(0.0);
                            let result = match name.as_str() {
                                "__hyp_add" | "__phys_add" => na + nb,
                                "__hyp_sub" | "__phys_sub" => na - nb,
                                "__hyp_mul" => na * nb,
                                "__hyp_div" => {
                                    if nb.abs() < f64::EPSILON {
                                        events.push(VmEvent::Error(VmError::DivisionByZero));
                                        break;
                                    }
                                    na / nb
                                }
                                _ => 0.0,
                            };
                            let _ = stack.push(MolecularChain::from_number(result));
                        }
                        "__cmp_lt" | "__cmp_gt" | "__cmp_le" | "__cmp_ge" | "__cmp_ne" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let nb = b.to_number().unwrap_or(0.0);
                            let truthy = match name.as_str() {
                                "__cmp_lt" => na < nb,
                                "__cmp_gt" => na > nb,
                                "__cmp_le" => na <= nb,
                                "__cmp_ge" => na >= nb,
                                "__cmp_ne" => (na - nb).abs() >= f64::EPSILON,
                                _ => false,
                            };
                            // true → non-empty chain (1.0), false → empty chain
                            // Jz checks is_empty() so empty = falsy
                            if truthy {
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__logic_not" => {
                            let a = vm_pop!(stack, events);
                            // Invert: empty → 1.0 (truthy), non-empty → empty (falsy)
                            if a.is_empty() {
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__assert_truth" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            // Truth: chains equal OR numeric values equal
                            let is_true = if let (Some(na), Some(nb)) =
                                (a.to_number(), b.to_number())
                            {
                                (na - nb).abs() < f64::EPSILON
                            } else {
                                a == b
                            };
                            if is_true {
                                let _ = stack.push(a); // push back (truthy)
                            } else {
                                let _ = stack.push(MolecularChain::empty()); // falsy
                            }
                        }
                        "__match_type" => {
                            // Pop expected type name (from Load) and actual type chain
                            // The actual chain was DUPed then TypeOf → events emitted
                            // Stack has: [... type_info_chain, expected_name_chain]
                            let _expected = vm_pop!(stack, events);
                            let actual = vm_pop!(stack, events);
                            // classify_chain returns "SDF", "MATH", "EMOTICON", "Mixed(...)", "Empty"
                            let type_name = classify_chain(&actual);
                            // The expected pattern comes from Load("SDF") which triggers
                            // LookupAlias. For matching, we check if the type_name
                            // starts with common known types. The pattern name is stored
                            // in the Load op preceding this Call.
                            // Since the VM is stack-based and we already consumed the
                            // expected chain, we look at the previous Load op's name
                            // from the events (LookupAlias).
                            let expected_name = events.iter().rev().find_map(|e| {
                                if let VmEvent::LookupAlias(n) = e { Some(n.clone()) } else { None }
                            }).unwrap_or_default();
                            let matches = type_name.starts_with(&expected_name)
                                || (expected_name == "Mixed" && type_name.starts_with("Mixed"));
                            if matches {
                                let _ = stack.push(actual); // truthy
                            } else {
                                let _ = stack.push(MolecularChain::empty()); // falsy
                            }
                        }
                        "__match_mol" => {
                            // Pop expected mol pattern and subject
                            let expected = vm_pop!(stack, events);
                            let actual = vm_pop!(stack, events);
                            // Compare molecule dimensions
                            let matches = if !actual.is_empty() && !expected.is_empty() {
                                let a = &actual.0[0];
                                let e = &expected.0[0];
                                a.shape == e.shape
                                    && a.relation == e.relation
                                    && a.emotion.valence == e.emotion.valence
                                    && a.emotion.arousal == e.emotion.arousal
                                    && a.time == e.time
                            } else {
                                actual.is_empty() && expected.is_empty()
                            };
                            if matches {
                                let _ = stack.push(actual); // truthy
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__array_new" => {
                            // Stack: [... elem0, elem1, ..., elemN-1, count]
                            // Pop count first (on top), then elements in reverse order
                            let count_chain = vm_pop!(stack, events);
                            let count = count_chain.to_number().unwrap_or(0.0) as usize;
                            let mut elements = Vec::new();
                            for _ in 0..count {
                                elements.push(vm_pop!(stack, events));
                            }
                            elements.reverse(); // restore original order
                            // Build array chain: elem0 | sep | elem1 | sep | elem2 ...
                            let sep = Molecule {
                                shape: 0, relation: 0,
                                emotion: EmotionDim { valence: 0, arousal: 0 },
                                time: 0,
                            };
                            let mut result = MolecularChain(Vec::new());
                            for (i, elem) in elements.into_iter().enumerate() {
                                if i > 0 {
                                    result.0.push(sep);
                                }
                                result.0.extend(elem.0.iter().cloned());
                            }
                            let _ = stack.push(result);
                        }
                        "__array_get" => {
                            // Stack: [array, index]
                            let idx_chain = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let idx = idx_chain.to_number().unwrap_or(0.0) as usize;
                            // Split array by separator molecules (shape=0, relation=0)
                            let elements = split_array_chain(&arr);
                            if idx < elements.len() {
                                let _ = stack.push(elements[idx].clone());
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__array_len" => {
                            let arr = vm_pop!(stack, events);
                            if arr.is_empty() {
                                let _ = stack.push(MolecularChain::from_number(0.0));
                            } else {
                                let count = split_array_chain(&arr).len();
                                let _ = stack.push(MolecularChain::from_number(count as f64));
                            }
                        }
                        "__concat" => {
                            // Concatenate two chains: pop b, pop a, push a+b
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let mut result = MolecularChain(Vec::new());
                            result.0.extend(a.0.iter().copied());
                            result.0.extend(b.0.iter().copied());
                            let _ = stack.push(result);
                        }
                        "__head" => {
                            // First molecule of chain
                            let chain = vm_pop!(stack, events);
                            if let Some(mol) = chain.0.first() {
                                let _ = stack.push(MolecularChain(alloc::vec![*mol]));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__tail" => {
                            // Chain without first molecule
                            let chain = vm_pop!(stack, events);
                            if chain.0.len() > 1 {
                                let _ = stack.push(MolecularChain(chain.0[1..].to_vec()));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__array_push" => {
                            // Stack: [array, element]
                            let elem = vm_pop!(stack, events);
                            let mut arr = vm_pop!(stack, events);
                            let sep = Molecule {
                                shape: 0, relation: 0,
                                emotion: EmotionDim { valence: 0, arousal: 0 },
                                time: 0,
                            };
                            if !arr.is_empty() {
                                arr.0.push(sep);
                            }
                            arr.0.extend(elem.0.iter().cloned());
                            let _ = stack.push(arr);
                        }
                        "__dict_new" => {
                            // Stack: [key0, val0, key1, val1, ..., count]
                            let count_chain = vm_pop!(stack, events);
                            let count = count_chain.to_number().unwrap_or(0.0) as usize;
                            let mut pairs = Vec::new();
                            for _ in 0..count {
                                let val = vm_pop!(stack, events);
                                let key = vm_pop!(stack, events);
                                pairs.push((key, val));
                            }
                            pairs.reverse();
                            // Encode as flat chain: key0|sep|val0|sep|key1|sep|val1...
                            let sep = Molecule {
                                shape: 0, relation: 0,
                                emotion: EmotionDim { valence: 0, arousal: 0 },
                                time: 0,
                            };
                            let mut result = MolecularChain(Vec::new());
                            for (i, (key, val)) in pairs.into_iter().enumerate() {
                                if i > 0 {
                                    result.0.push(sep);
                                }
                                result.0.extend(key.0.iter().cloned());
                                result.0.push(sep);
                                result.0.extend(val.0.iter().cloned());
                            }
                            let _ = stack.push(result);
                        }
                        "__dict_get" => {
                            // Stack: [dict, key]
                            let key = vm_pop!(stack, events);
                            let dict = vm_pop!(stack, events);
                            let elements = split_array_chain(&dict);
                            // Keys at even indices, values at odd indices
                            let mut found = false;
                            let mut i = 0;
                            while i + 1 < elements.len() {
                                if elements[i].0 == key.0 {
                                    let _ = stack.push(elements[i + 1].clone());
                                    found = true;
                                    break;
                                }
                                i += 2;
                            }
                            if !found {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__dict_keys" => {
                            // Stack: [dict]
                            // Returns array of keys (even-indexed elements)
                            let dict = vm_pop!(stack, events);
                            let elements = split_array_chain(&dict);
                            let sep = Molecule {
                                shape: 0, relation: 0,
                                emotion: EmotionDim { valence: 0, arousal: 0 },
                                time: 0,
                            };
                            let mut result = MolecularChain(Vec::new());
                            let mut key_idx = 0;
                            let mut i = 0;
                            while i < elements.len() {
                                if key_idx > 0 {
                                    result.0.push(sep);
                                }
                                result.0.extend(elements[i].0.iter().cloned());
                                key_idx += 1;
                                i += 2; // skip values
                            }
                            let _ = stack.push(result);
                        }
                        "__dict_set" => {
                            // Stack: [dict, key, value]
                            let value = vm_pop!(stack, events);
                            let key = vm_pop!(stack, events);
                            let dict = vm_pop!(stack, events);
                            let mut elements = split_array_chain(&dict);
                            // Find and update existing key, or append
                            let mut found = false;
                            let mut i = 0;
                            while i + 1 < elements.len() {
                                if elements[i].0 == key.0 {
                                    elements[i + 1] = value.clone();
                                    found = true;
                                    break;
                                }
                                i += 2;
                            }
                            if !found {
                                elements.push(key);
                                elements.push(value);
                            }
                            // Rebuild chain from elements
                            let sep = Molecule {
                                shape: 0, relation: 0,
                                emotion: EmotionDim { valence: 0, arousal: 0 },
                                time: 0,
                            };
                            let mut result = MolecularChain(Vec::new());
                            for (j, elem) in elements.into_iter().enumerate() {
                                if j > 0 {
                                    result.0.push(sep);
                                }
                                result.0.extend(elem.0.iter().cloned());
                            }
                            let _ = stack.push(result);
                        }
                        "__str_len" => {
                            // String length: count molecules in chain
                            let s = vm_pop!(stack, events);
                            let _ = stack.push(MolecularChain::from_number(s.0.len() as f64));
                        }
                        "__str_concat" => {
                            // Concatenate two chains
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let mut result = a.0.clone();
                            result.extend(b.0.iter().cloned());
                            let _ = stack.push(MolecularChain(result));
                        }
                        "__to_string" => {
                            // Number → string chain: encode digits as molecules
                            let val = vm_pop!(stack, events);
                            let n = val.to_number().unwrap_or(0.0);
                            let s = if n == (n as i64 as f64) {
                                alloc::format!("{}", n as i64)
                            } else {
                                alloc::format!("{}", n)
                            };
                            let mut mols = Vec::new();
                            for b in s.bytes() {
                                mols.push(Molecule {
                                    shape: 0x02, relation: 0x01,
                                    emotion: EmotionDim { valence: b, arousal: 0 },
                                    time: 0x01,
                                });
                            }
                            let _ = stack.push(MolecularChain(mols));
                        }
                        "__to_number" => {
                            // String chain → number: decode molecules back to digits
                            let val = vm_pop!(stack, events);
                            // Try as number first
                            if let Some(n) = val.to_number() {
                                let _ = stack.push(MolecularChain::from_number(n));
                            } else {
                                // Decode string bytes from valence
                                let s: String = val.0.iter()
                                    .map(|m| m.emotion.valence as char)
                                    .collect();
                                if let Ok(n) = s.parse::<f64>() {
                                    let _ = stack.push(MolecularChain::from_number(n));
                                } else {
                                    let _ = stack.push(MolecularChain::from_number(0.0));
                                }
                            }
                        }
                        "__print" => {
                            // Print: emit top of stack as output (same as Emit but via call)
                            let val = vm_pop!(stack, events);
                            events.push(VmEvent::Output(val));
                        }
                        "__println" => {
                            // Print with newline: emit value + newline as string chain
                            let val = vm_pop!(stack, events);
                            let text = format_chain_display(&val);
                            let with_nl = alloc::format!("{}\n", text);
                            events.push(VmEvent::Output(string_to_chain(&with_nl)));
                        }
                        "__hyp_mod" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let nb = b.to_number().unwrap_or(1.0);
                            let _ = stack.push(MolecularChain::from_number(na % nb));
                        }
                        "__hyp_neg" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(-na));
                        }
                        "__hyp_abs" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(na.abs()));
                        }
                        "__hyp_min" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let nb = b.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(na.min(nb)));
                        }
                        "__hyp_max" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let nb = b.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(na.max(nb)));
                        }
                        "__array_set" => {
                            // Stack: [array, index, value]
                            let value = vm_pop!(stack, events);
                            let idx_chain = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let idx = idx_chain.to_number().unwrap_or(0.0) as usize;
                            let mut elements = split_array_chain(&arr);
                            if idx < elements.len() {
                                elements[idx] = value;
                            }
                            let sep = Molecule {
                                shape: 0, relation: 0,
                                emotion: EmotionDim { valence: 0, arousal: 0 },
                                time: 0,
                            };
                            let mut result = MolecularChain(Vec::new());
                            for (j, elem) in elements.into_iter().enumerate() {
                                if j > 0 { result.0.push(sep); }
                                result.0.extend(elem.0.iter().cloned());
                            }
                            let _ = stack.push(result);
                        }
                        "__array_slice" => {
                            // Stack: [array, start, end]
                            let end_chain = vm_pop!(stack, events);
                            let start_chain = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let start = start_chain.to_number().unwrap_or(0.0) as usize;
                            let end = end_chain.to_number().unwrap_or(0.0) as usize;
                            let elements = split_array_chain(&arr);
                            let sliced: Vec<_> = elements.into_iter()
                                .skip(start)
                                .take(end.saturating_sub(start))
                                .collect();
                            let sep = Molecule {
                                shape: 0, relation: 0,
                                emotion: EmotionDim { valence: 0, arousal: 0 },
                                time: 0,
                            };
                            let mut result = MolecularChain(Vec::new());
                            for (j, elem) in sliced.into_iter().enumerate() {
                                if j > 0 { result.0.push(sep); }
                                result.0.extend(elem.0.iter().cloned());
                            }
                            let _ = stack.push(result);
                        }
                        "__is_empty" => {
                            let val = vm_pop!(stack, events);
                            let result = if val.is_empty() { 1.0 } else { 0.0 };
                            let _ = stack.push(MolecularChain::from_number(result));
                        }
                        "__eq" => {
                            // Deep equality: compare two chains molecule by molecule
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let result = if a.0 == b.0 { 1.0 } else { 0.0 };
                            let _ = stack.push(MolecularChain::from_number(result));
                        }
                        // ── String builtins ────────────────────────────────
                        "__str_split" => {
                            // Stack: [string_chain, delimiter_chain]
                            // Split string by delimiter, return array of sub-strings
                            let delim = vm_pop!(stack, events);
                            let s = vm_pop!(stack, events);
                            // Decode both to byte strings via valence
                            let s_bytes: Vec<u8> = s.0.iter().map(|m| m.emotion.valence).collect();
                            let d_bytes: Vec<u8> = delim.0.iter().map(|m| m.emotion.valence).collect();
                            if d_bytes.is_empty() {
                                let _ = stack.push(s); // no split on empty delim
                            } else {
                                // Split
                                let sep = Molecule {
                                    shape: 0, relation: 0,
                                    emotion: EmotionDim { valence: 0, arousal: 0 },
                                    time: 0,
                                };
                                let mut result = MolecularChain(Vec::new());
                                let mut start = 0;
                                let mut elem_idx = 0;
                                while start <= s_bytes.len() {
                                    // Find next occurrence of delimiter
                                    let found = if start + d_bytes.len() <= s_bytes.len() {
                                        s_bytes[start..].windows(d_bytes.len())
                                            .position(|w| w == d_bytes.as_slice())
                                    } else {
                                        None
                                    };
                                    let end = match found {
                                        Some(pos) => start + pos,
                                        None => s_bytes.len(),
                                    };
                                    if elem_idx > 0 { result.0.push(sep); }
                                    for &b in &s_bytes[start..end] {
                                        result.0.push(Molecule {
                                            shape: 0x02, relation: 0x01,
                                            emotion: EmotionDim { valence: b, arousal: 0 },
                                            time: 0x01,
                                        });
                                    }
                                    elem_idx += 1;
                                    if found.is_some() {
                                        start = end + d_bytes.len();
                                    } else {
                                        break;
                                    }
                                }
                                let _ = stack.push(result);
                            }
                        }
                        "__str_contains" => {
                            // Stack: [haystack, needle] → 1.0 if contains, empty if not
                            let needle = vm_pop!(stack, events);
                            let haystack = vm_pop!(stack, events);
                            let h_bytes: Vec<u8> = haystack.0.iter().map(|m| m.emotion.valence).collect();
                            let n_bytes: Vec<u8> = needle.0.iter().map(|m| m.emotion.valence).collect();
                            let found = if n_bytes.is_empty() {
                                true
                            } else {
                                h_bytes.windows(n_bytes.len()).any(|w| w == n_bytes.as_slice())
                            };
                            if found {
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__str_replace" => {
                            // Stack: [string, old_pattern, new_pattern] → replaced string
                            let new_pat = vm_pop!(stack, events);
                            let old_pat = vm_pop!(stack, events);
                            let s = vm_pop!(stack, events);
                            let s_bytes: Vec<u8> = s.0.iter().map(|m| m.emotion.valence).collect();
                            let old_bytes: Vec<u8> = old_pat.0.iter().map(|m| m.emotion.valence).collect();
                            let new_bytes: Vec<u8> = new_pat.0.iter().map(|m| m.emotion.valence).collect();
                            let mut result_bytes = Vec::new();
                            let mut i = 0;
                            if old_bytes.is_empty() {
                                result_bytes = s_bytes;
                            } else {
                                while i < s_bytes.len() {
                                    if i + old_bytes.len() <= s_bytes.len()
                                        && s_bytes[i..i + old_bytes.len()] == *old_bytes.as_slice()
                                    {
                                        result_bytes.extend_from_slice(&new_bytes);
                                        i += old_bytes.len();
                                    } else {
                                        result_bytes.push(s_bytes[i]);
                                        i += 1;
                                    }
                                }
                            }
                            let mut mols = Vec::new();
                            for b in result_bytes {
                                mols.push(Molecule {
                                    shape: 0x02, relation: 0x01,
                                    emotion: EmotionDim { valence: b, arousal: 0 },
                                    time: 0x01,
                                });
                            }
                            let _ = stack.push(MolecularChain(mols));
                        }
                        "__str_starts_with" => {
                            // Stack: [string, prefix] → 1.0 if starts with, empty if not
                            let prefix = vm_pop!(stack, events);
                            let s = vm_pop!(stack, events);
                            let s_bytes: Vec<u8> = s.0.iter().map(|m| m.emotion.valence).collect();
                            let p_bytes: Vec<u8> = prefix.0.iter().map(|m| m.emotion.valence).collect();
                            let starts = s_bytes.starts_with(&p_bytes);
                            if starts {
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__str_ends_with" => {
                            let suffix = vm_pop!(stack, events);
                            let s = vm_pop!(stack, events);
                            let s_bytes: Vec<u8> = s.0.iter().map(|m| m.emotion.valence).collect();
                            let x_bytes: Vec<u8> = suffix.0.iter().map(|m| m.emotion.valence).collect();
                            let ends = s_bytes.ends_with(&x_bytes);
                            if ends {
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__str_index_of" => {
                            // Stack: [haystack, needle] → index (number) or -1
                            let needle = vm_pop!(stack, events);
                            let haystack = vm_pop!(stack, events);
                            let h_bytes: Vec<u8> = haystack.0.iter().map(|m| m.emotion.valence).collect();
                            let n_bytes: Vec<u8> = needle.0.iter().map(|m| m.emotion.valence).collect();
                            let idx = if n_bytes.is_empty() {
                                0i64
                            } else {
                                h_bytes.windows(n_bytes.len())
                                    .position(|w| w == n_bytes.as_slice())
                                    .map(|i| i as i64)
                                    .unwrap_or(-1)
                            };
                            let _ = stack.push(MolecularChain::from_number(idx as f64));
                        }
                        "__str_trim" => {
                            let s = vm_pop!(stack, events);
                            // Trim leading/trailing whitespace (space=0x20, tab=0x09, etc)
                            let bytes: Vec<u8> = s.0.iter().map(|m| m.emotion.valence).collect();
                            let trimmed: &[u8] = {
                                let start = bytes.iter().position(|&b| b != b' ' && b != b'\t' && b != b'\n' && b != b'\r').unwrap_or(bytes.len());
                                let end = bytes.iter().rposition(|&b| b != b' ' && b != b'\t' && b != b'\n' && b != b'\r').map(|i| i + 1).unwrap_or(start);
                                &bytes[start..end]
                            };
                            let mut mols = Vec::new();
                            for &b in trimmed {
                                mols.push(Molecule {
                                    shape: 0x02, relation: 0x01,
                                    emotion: EmotionDim { valence: b, arousal: 0 },
                                    time: 0x01,
                                });
                            }
                            let _ = stack.push(MolecularChain(mols));
                        }
                        "__str_upper" => {
                            let s = vm_pop!(stack, events);
                            let mut mols = Vec::new();
                            for m in &s.0 {
                                let b = m.emotion.valence;
                                let upper = if b.is_ascii_lowercase() { b - 32 } else { b };
                                mols.push(Molecule {
                                    shape: 0x02, relation: 0x01,
                                    emotion: EmotionDim { valence: upper, arousal: 0 },
                                    time: 0x01,
                                });
                            }
                            let _ = stack.push(MolecularChain(mols));
                        }
                        "__str_lower" => {
                            let s = vm_pop!(stack, events);
                            let mut mols = Vec::new();
                            for m in &s.0 {
                                let b = m.emotion.valence;
                                let lower = if b.is_ascii_uppercase() { b + 32 } else { b };
                                mols.push(Molecule {
                                    shape: 0x02, relation: 0x01,
                                    emotion: EmotionDim { valence: lower, arousal: 0 },
                                    time: 0x01,
                                });
                            }
                            let _ = stack.push(MolecularChain(mols));
                        }
                        "__str_substr" => {
                            // Stack: [string, start, length]
                            let len_chain = vm_pop!(stack, events);
                            let start_chain = vm_pop!(stack, events);
                            let s = vm_pop!(stack, events);
                            let start = start_chain.to_number().unwrap_or(0.0) as usize;
                            let len = len_chain.to_number().unwrap_or(0.0) as usize;
                            let mols: Vec<Molecule> = s.0.iter()
                                .skip(start)
                                .take(len)
                                .copied()
                                .collect();
                            let _ = stack.push(MolecularChain(mols));
                        }
                        // ── Math builtins ──────────────────────────────────
                        "__hyp_floor" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::floor(na)));
                        }
                        "__hyp_ceil" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::ceil(na)));
                        }
                        "__hyp_round" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::round(na)));
                        }
                        "__hyp_sqrt" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::sqrt(na)));
                        }
                        "__hyp_pow" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let nb = b.to_number().unwrap_or(1.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::pow(na, nb)));
                        }
                        "__hyp_log" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(1.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::log(na)));
                        }
                        "__hyp_sin" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::sin(na)));
                        }
                        "__hyp_cos" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::cos(na)));
                        }
                        // ── Dict builtins ──────────────────────────────────
                        "__dict_has_key" => {
                            // Stack: [dict, key] → 1.0 if key exists, empty if not
                            let key = vm_pop!(stack, events);
                            let dict = vm_pop!(stack, events);
                            let elements = split_array_chain(&dict);
                            let mut found = false;
                            let mut i = 0;
                            while i + 1 < elements.len() {
                                if elements[i].0 == key.0 {
                                    found = true;
                                    break;
                                }
                                i += 2;
                            }
                            if found {
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__dict_values" => {
                            // Stack: [dict] → array of values (odd-indexed elements)
                            let dict = vm_pop!(stack, events);
                            let elements = split_array_chain(&dict);
                            let sep = Molecule {
                                shape: 0, relation: 0,
                                emotion: EmotionDim { valence: 0, arousal: 0 },
                                time: 0,
                            };
                            let mut result = MolecularChain(Vec::new());
                            let mut val_idx = 0;
                            let mut i = 1; // start at first value
                            while i < elements.len() {
                                if val_idx > 0 { result.0.push(sep); }
                                result.0.extend(elements[i].0.iter().cloned());
                                val_idx += 1;
                                i += 2;
                            }
                            let _ = stack.push(result);
                        }
                        "__dict_merge" => {
                            // Stack: [dict_a, dict_b] → merged dict (b overrides a)
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let mut a_elems = split_array_chain(&a);
                            let b_elems = split_array_chain(&b);
                            // Merge: for each key in b, update or append to a
                            let mut j = 0;
                            while j + 1 < b_elems.len() {
                                let bkey = &b_elems[j];
                                let bval = &b_elems[j + 1];
                                let mut found = false;
                                let mut k = 0;
                                while k + 1 < a_elems.len() {
                                    if a_elems[k].0 == bkey.0 {
                                        a_elems[k + 1] = bval.clone();
                                        found = true;
                                        break;
                                    }
                                    k += 2;
                                }
                                if !found {
                                    a_elems.push(bkey.clone());
                                    a_elems.push(bval.clone());
                                }
                                j += 2;
                            }
                            let sep = Molecule {
                                shape: 0, relation: 0,
                                emotion: EmotionDim { valence: 0, arousal: 0 },
                                time: 0,
                            };
                            let mut result = MolecularChain(Vec::new());
                            for (idx, elem) in a_elems.into_iter().enumerate() {
                                if idx > 0 { result.0.push(sep); }
                                result.0.extend(elem.0.iter().cloned());
                            }
                            let _ = stack.push(result);
                        }
                        "__dict_remove" => {
                            // Stack: [dict, key] → dict without that key
                            let key = vm_pop!(stack, events);
                            let dict = vm_pop!(stack, events);
                            let elements = split_array_chain(&dict);
                            let sep = Molecule {
                                shape: 0, relation: 0,
                                emotion: EmotionDim { valence: 0, arousal: 0 },
                                time: 0,
                            };
                            let mut result = MolecularChain(Vec::new());
                            let mut out_idx = 0;
                            let mut i = 0;
                            while i + 1 < elements.len() {
                                if elements[i].0 != key.0 {
                                    if out_idx > 0 { result.0.push(sep); }
                                    result.0.extend(elements[i].0.iter().cloned());
                                    result.0.push(sep);
                                    result.0.extend(elements[i + 1].0.iter().cloned());
                                    out_idx += 1;
                                }
                                i += 2;
                            }
                            let _ = stack.push(result);
                        }
                        // ── Array builtins ─────────────────────────────────
                        "__array_pop" => {
                            // Stack: [array] → [array_without_last, last_element]
                            let arr = vm_pop!(stack, events);
                            let mut elements = split_array_chain(&arr);
                            if let Some(last) = elements.pop() {
                                let sep = Molecule {
                                    shape: 0, relation: 0,
                                    emotion: EmotionDim { valence: 0, arousal: 0 },
                                    time: 0,
                                };
                                let mut rest = MolecularChain(Vec::new());
                                for (j, elem) in elements.into_iter().enumerate() {
                                    if j > 0 { rest.0.push(sep); }
                                    rest.0.extend(elem.0.iter().cloned());
                                }
                                let _ = stack.push(rest);
                                let _ = stack.push(last);
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__array_reverse" => {
                            let arr = vm_pop!(stack, events);
                            let mut elements = split_array_chain(&arr);
                            elements.reverse();
                            let sep = Molecule {
                                shape: 0, relation: 0,
                                emotion: EmotionDim { valence: 0, arousal: 0 },
                                time: 0,
                            };
                            let mut result = MolecularChain(Vec::new());
                            for (j, elem) in elements.into_iter().enumerate() {
                                if j > 0 { result.0.push(sep); }
                                result.0.extend(elem.0.iter().cloned());
                            }
                            let _ = stack.push(result);
                        }
                        "__array_contains" => {
                            // Stack: [array, element] → 1.0 if found, empty if not
                            let elem = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let elements = split_array_chain(&arr);
                            let found = elements.iter().any(|e| e.0 == elem.0);
                            if found {
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__array_join" => {
                            // Stack: [array, separator_string] → joined string
                            let sep_chain = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let elements = split_array_chain(&arr);
                            let mut result = MolecularChain(Vec::new());
                            for (j, elem) in elements.into_iter().enumerate() {
                                if j > 0 {
                                    result.0.extend(sep_chain.0.iter().cloned());
                                }
                                result.0.extend(elem.0.iter().cloned());
                            }
                            let _ = stack.push(result);
                        }
                        "__array_map" => {
                            // Simple map: applies a number transform from stack
                            // Stack: [array, function_chain]
                            // For now: just return the array (closures needed for full impl)
                            let _fn = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let _ = stack.push(arr);
                        }
                        "__array_filter" => {
                            // Simple filter: for now just return the array
                            let _fn = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let _ = stack.push(arr);
                        }
                        // ── ISL builtins ───────────────────────────────────
                        "__isl_send" => {
                            // Stack: [address_chain, payload_chain]
                            let payload = vm_pop!(stack, events);
                            let addr = vm_pop!(stack, events);
                            events.push(VmEvent::Output(MolecularChain(alloc::vec![
                                Molecule { shape: 0x0A, relation: 0x06,
                                    emotion: EmotionDim { valence: 0x01, arousal: 0 },
                                    time: 0x03 }
                            ])));
                            // Emit ISL send event for runtime to handle
                            events.push(VmEvent::CreateEdge {
                                from: addr.chain_hash(),
                                to: payload.chain_hash(),
                                rel: 0x06, // Causes (send triggers receive)
                            });
                            let _ = stack.push(MolecularChain::from_number(1.0)); // success
                        }
                        "__isl_broadcast" => {
                            // Stack: [payload_chain]
                            let payload = vm_pop!(stack, events);
                            events.push(VmEvent::Output(payload));
                            let _ = stack.push(MolecularChain::from_number(1.0));
                        }
                        // ── Type conversion ────────────────────────────────
                        "__type_of" => {
                            let val = vm_pop!(stack, events);
                            let type_name = classify_chain(&val);
                            // Encode type name as string chain
                            let mut mols = Vec::new();
                            for b in type_name.bytes() {
                                mols.push(Molecule {
                                    shape: 0x02, relation: 0x01,
                                    emotion: EmotionDim { valence: b, arousal: 0 },
                                    time: 0x01,
                                });
                            }
                            let _ = stack.push(MolecularChain(mols));
                        }
                        "__chain_hash" => {
                            let val = vm_pop!(stack, events);
                            let hash = val.chain_hash();
                            let _ = stack.push(MolecularChain::from_number(hash as f64));
                        }
                        "__chain_len" => {
                            let val = vm_pop!(stack, events);
                            let _ = stack.push(MolecularChain::from_number(val.0.len() as f64));
                        }
                        // ── Device I/O builtins ──────────────────────────
                        "__device_write" => {
                            // Stack: [device_id_string, value_chain]
                            let value_chain = vm_pop!(stack, events);
                            let id_chain = vm_pop!(stack, events);
                            let device_id = chain_to_string(&id_chain).unwrap_or_default();
                            let value = value_chain.to_number().unwrap_or(0.0) as u8;
                            events.push(VmEvent::DeviceWrite { device_id, value });
                            let _ = stack.push(MolecularChain::from_number(1.0));
                        }
                        "__device_read" => {
                            // Stack: [device_id_string]
                            let id_chain = vm_pop!(stack, events);
                            let device_id = chain_to_string(&id_chain).unwrap_or_default();
                            events.push(VmEvent::DeviceRead { device_id });
                            let _ = stack.push(MolecularChain::empty()); // placeholder
                        }
                        "__device_list" => {
                            events.push(VmEvent::DeviceListRequest);
                            let _ = stack.push(MolecularChain::empty());
                        }
                        // ── FFI builtins ─────────────────────────────────
                        "__ffi" => {
                            // Stack: [function_name_string, ...args]
                            let name_chain = vm_pop!(stack, events);
                            let fn_name = chain_to_string(&name_chain).unwrap_or_default();
                            events.push(VmEvent::FfiCall { name: fn_name, args: Vec::new() });
                            let _ = stack.push(MolecularChain::empty());
                        }
                        // ── File I/O builtins ────────────────────────────
                        "__file_read" => {
                            let path_chain = vm_pop!(stack, events);
                            let path = chain_to_string(&path_chain).unwrap_or_default();
                            events.push(VmEvent::FileReadRequest { path });
                            let _ = stack.push(MolecularChain::empty());
                        }
                        "__file_write" => {
                            let data_chain = vm_pop!(stack, events);
                            let path_chain = vm_pop!(stack, events);
                            let path = chain_to_string(&path_chain).unwrap_or_default();
                            let data = if let Some(s) = chain_to_string(&data_chain) {
                                s.into_bytes()
                            } else {
                                data_chain.to_tagged_bytes()
                            };
                            events.push(VmEvent::FileWriteRequest { path, data });
                            let _ = stack.push(MolecularChain::from_number(1.0));
                        }
                        "__file_append" => {
                            let data_chain = vm_pop!(stack, events);
                            let path_chain = vm_pop!(stack, events);
                            let path = chain_to_string(&path_chain).unwrap_or_default();
                            let data = if let Some(s) = chain_to_string(&data_chain) {
                                s.into_bytes()
                            } else {
                                data_chain.to_tagged_bytes()
                            };
                            events.push(VmEvent::FileAppendRequest { path, data });
                            let _ = stack.push(MolecularChain::from_number(1.0));
                        }
                        // ── Time builtins ────────────────────────────────
                        "__time" => {
                            // Push 0 — Runtime injects actual timestamp
                            let _ = stack.push(MolecularChain::from_number(0.0));
                        }
                        "__sleep" => {
                            // Pop duration_ms — no-op in VM (Runtime handles)
                            let _duration = vm_pop!(stack, events);
                        }

                        // ── Type system builtins ──────────────────────────────
                        "__struct_def" => {
                            // Stack: [fields_array, name_chain]
                            // Register struct type — store in scope as metadata
                            let name_chain = vm_pop!(stack, events);
                            let fields = vm_pop!(stack, events);
                            // Store struct definition: "__struct_Name" → fields array
                            let name = chain_to_string(&name_chain).unwrap_or_default();
                            let key = alloc::format!("__struct_{}", name);
                            let scope = scopes.last_mut().unwrap();
                            scope.push((key, fields));
                        }
                        "__struct_tag" => {
                            // Stack: [dict_chain, name_chain]
                            // Tag a dict with a struct type name
                            let name_chain = vm_pop!(stack, events);
                            let dict = vm_pop!(stack, events);
                            // Prepend type tag as first key-value pair "__type" => name
                            let sep = Molecule {
                                shape: 0, relation: 0,
                                emotion: EmotionDim { valence: 0, arousal: 0 },
                                time: 0,
                            };
                            let type_key = string_to_chain("__type");
                            let mut tagged = MolecularChain(Vec::new());
                            tagged.0.extend(type_key.0.iter().copied());
                            tagged.0.push(sep);
                            tagged.0.extend(name_chain.0.iter().copied());
                            if !dict.is_empty() {
                                tagged.0.push(sep);
                                tagged.0.extend(dict.0.iter().copied());
                            }
                            let _ = stack.push(tagged);
                        }
                        "__enum_def" => {
                            // Stack: [variants_array, name_chain]
                            let name_chain = vm_pop!(stack, events);
                            let variants = vm_pop!(stack, events);
                            let name = chain_to_string(&name_chain).unwrap_or_default();
                            let key = alloc::format!("__enum_{}", name);
                            let scope = scopes.last_mut().unwrap();
                            scope.push((key, variants));
                        }
                        "__enum_unit" => {
                            // Stack: [tag_chain]
                            // Unit variant — just push tag as-is
                            // tag is already on stack
                        }
                        "__enum_payload" => {
                            // Stack: [tag, arg0, arg1, ..., count]
                            // Build: tag | sep | arg0 | sep | arg1 ...
                            let count_chain = vm_pop!(stack, events);
                            let count = count_chain.to_number().unwrap_or(0.0) as usize;
                            let mut args = Vec::new();
                            for _ in 0..count {
                                args.push(vm_pop!(stack, events));
                            }
                            args.reverse();
                            let tag = vm_pop!(stack, events);
                            let sep = Molecule {
                                shape: 0, relation: 0,
                                emotion: EmotionDim { valence: 0, arousal: 0 },
                                time: 0,
                            };
                            let mut result = MolecularChain(Vec::new());
                            result.0.extend(tag.0.iter().copied());
                            for arg in args {
                                result.0.push(sep);
                                result.0.extend(arg.0.iter().copied());
                            }
                            let _ = stack.push(result);
                        }
                        "__method_call" => {
                            // Stack: [self, arg0, ..., argN, arg_count, method_name]
                            let method_name_chain = vm_pop!(stack, events);
                            let count_chain = vm_pop!(stack, events);
                            let count = count_chain.to_number().unwrap_or(1.0) as usize;
                            let method_name = chain_to_string(&method_name_chain).unwrap_or_default();
                            // Pop all args (including self)
                            let mut args = Vec::new();
                            for _ in 0..count {
                                args.push(vm_pop!(stack, events));
                            }
                            args.reverse();
                            // Get type tag from self (first arg) — look for __type key
                            let self_val = &args[0];
                            let elements = split_array_chain(self_val);
                            let mut type_name = String::new();
                            let type_key = string_to_chain("__type");
                            let mut i = 0;
                            while i + 1 < elements.len() {
                                if elements[i].0 == type_key.0 {
                                    type_name = chain_to_string(&elements[i + 1]).unwrap_or_default();
                                    break;
                                }
                                i += 2;
                            }
                            // Lookup mangled function name: __Type_method
                            let mangled = alloc::format!("__{type_name}_{method_name}");
                            // Push args back and call the mangled function
                            for arg in &args {
                                let _ = stack.push(arg.clone());
                            }
                            events.push(VmEvent::LookupAlias(mangled));
                        }
                        // ── Channel builtins ──────────────────────────────
                        "__channel_new" => {
                            // Create a new channel, push its ID as a number
                            let id = next_channel_id;
                            next_channel_id += 1;
                            channels.push(Vec::new()); // queue for this channel
                            let _ = stack.push(MolecularChain::from_number(id as f64));
                        }
                        "__channel_send" => {
                            // Stack: [channel_id, value]
                            let value = vm_pop!(stack, events);
                            let ch_chain = vm_pop!(stack, events);
                            let ch_id = ch_chain.to_number().unwrap_or(0.0) as usize;
                            if ch_id >= 1 && ch_id <= channels.len() {
                                channels[ch_id - 1].push(value);
                            }
                            let _ = stack.push(MolecularChain::from_number(1.0)); // success
                        }
                        "__channel_recv" => {
                            // Stack: [channel_id] → pops first message or empty
                            let ch_chain = vm_pop!(stack, events);
                            let ch_id = ch_chain.to_number().unwrap_or(0.0) as usize;
                            if ch_id >= 1 && ch_id <= channels.len() && !channels[ch_id - 1].is_empty() {
                                let msg = channels[ch_id - 1].remove(0);
                                let _ = stack.push(msg);
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }

                        "__use_module" => {
                            // Module loading — emit event for Runtime to handle
                            let module_chain = vm_pop!(stack, events);
                            let name = chain_to_string(&module_chain).unwrap_or_default();
                            events.push(VmEvent::UseModule { name });
                        }

                        "__use_module_selective" => {
                            // Selective module import: pop count, import names, module name
                            let count_chain = vm_pop!(stack, events);
                            let count = count_chain.to_number().unwrap_or(0.0) as usize;
                            let mut imports = Vec::new();
                            for _ in 0..count {
                                let imp = vm_pop!(stack, events);
                                imports.push(chain_to_string(&imp).unwrap_or_default());
                            }
                            imports.reverse(); // restore original order
                            let module_chain = vm_pop!(stack, events);
                            let name = chain_to_string(&module_chain).unwrap_or_default();
                            events.push(VmEvent::UseModuleSelective { name, imports });
                        }

                        "__mod_decl" => {
                            // Module declaration — register module path
                            let path_chain = vm_pop!(stack, events);
                            let path = chain_to_string(&path_chain).unwrap_or_default();
                            events.push(VmEvent::ModDecl { path });
                        }

                        _ => {
                            // Unknown function → emit lookup event
                            events.push(VmEvent::LookupAlias(name.clone()));
                        }
                    }
                }

                Op::Ret => {
                    break;
                }

                Op::Dream => {
                    events.push(VmEvent::TriggerDream);
                }

                Op::Stats => {
                    events.push(VmEvent::RequestStats);
                }

                Op::Store(name) => {
                    let val = vm_pop!(stack, events);
                    // Store in current (innermost) scope.
                    // Update existing in current scope, else insert new.
                    // SAFETY: scopes always has root scope (initialized above)
                    let Some(scope) = scopes.last_mut() else { break };
                    if let Some(entry) = scope.iter_mut().find(|(n, _)| n == name) {
                        entry.1 = val;
                    } else {
                        scope.push((name.clone(), val));
                    }
                }

                Op::StoreUpdate(name) => {
                    let val = vm_pop!(stack, events);
                    // Search ALL scopes from innermost outward; update first match.
                    // If not found anywhere, store in current scope (fallback).
                    let mut found = false;
                    for scope in scopes.iter_mut().rev() {
                        if let Some(entry) = scope.iter_mut().find(|(n, _)| n == name) {
                            entry.1 = val.clone();
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        if let Some(scope) = scopes.last_mut() {
                            scope.push((name.clone(), val));
                        }
                    }
                }

                Op::LoadLocal(name) => {
                    // Search from innermost scope outward (lexical scoping)
                    let val = scopes
                        .iter()
                        .rev()
                        .find_map(|scope| {
                            scope.iter().rev().find(|(n, _)| n == name).map(|(_, c)| c.clone())
                        })
                        .unwrap_or_else(MolecularChain::empty);
                    if let Err(e) = stack.push(val) {
                        events.push(VmEvent::Error(e));
                        break;
                    }
                }

                Op::ScopeBegin => {
                    call_depth += 1;
                    if call_depth > self.max_call_depth {
                        events.push(VmEvent::Error(VmError::MaxCallDepthExceeded));
                        break;
                    }
                    scopes.push(Vec::new());
                }

                Op::ScopeEnd => {
                    // Pop innermost scope (discard locals)
                    // Never pop root scope
                    if scopes.len() > 1 {
                        scopes.pop();
                        call_depth = call_depth.saturating_sub(1);
                    }
                    // Check loop stack: if we're at end of a loop body, jump back
                    if let Some(entry) = loop_stack.last_mut() {
                        if entry.1 > 0 {
                            entry.1 -= 1;
                            pc = entry.0; // jump back to loop body start
                        } else {
                            loop_stack.pop();
                        }
                    }
                }

                Op::Fuse => {
                    // QT2: ∞ là sai — ∞-1 mới đúng
                    // Pop chain, check nó hữu hạn (không có self-reference loop).
                    // Nếu chain hữu hạn → push lại (∞-1 = đúng).
                    // Nếu chain rỗng hoặc bất thường → push empty (∞ = sai).
                    let chain = vm_pop!(stack, events);
                    // Finite check: chain must have bounded length
                    // (MolecularChain is always finite by construction,
                    //  but FUSE ensures no runtime-generated infinite loops)
                    if chain.is_empty() {
                        // ∞ = sai → empty
                        let _ = stack.push(MolecularChain::empty());
                    } else {
                        // ∞-1 = đúng → push back
                        let _ = stack.push(chain);
                    }
                }

                Op::Trace => {
                    // Toggle tracing: emit a trace event for current state
                    events.push(VmEvent::TraceStep {
                        op_name: "TRACE",
                        stack_depth: stack.data.len(),
                        pc: pc - 1,
                    });
                }

                Op::Inspect => {
                    let chain = vm_pop!(stack, events);
                    let bytes = chain.to_bytes();
                    events.push(VmEvent::InspectChain {
                        hash: chain.chain_hash(),
                        molecule_count: chain.len(),
                        byte_size: bytes.len(),
                        is_empty: chain.is_empty(),
                    });
                    // Push back so chain is still available
                    let _ = stack.push(chain);
                }

                Op::Assert => {
                    let chain = vm_pop!(stack, events);
                    if chain.is_empty() {
                        events.push(VmEvent::AssertFailed);
                    }
                    // Push back regardless
                    let _ = stack.push(chain);
                }

                Op::TypeOf => {
                    let chain = vm_pop!(stack, events);
                    // Classify based on molecule bytes
                    let classification = classify_chain(&chain);
                    events.push(VmEvent::TypeInfo {
                        hash: chain.chain_hash(),
                        classification,
                    });
                    let _ = stack.push(chain);
                }

                Op::Why => {
                    let b = vm_pop!(stack, events);
                    let a = vm_pop!(stack, events);
                    events.push(VmEvent::WhyConnection {
                        from: a.chain_hash(),
                        to: b.chain_hash(),
                    });
                    // Push LCA as result (their common ancestor = their connection)
                    let result = if a.is_empty() || b.is_empty() {
                        if !a.is_empty() {
                            a
                        } else {
                            b
                        }
                    } else {
                        lca(&a, &b)
                    };
                    let _ = stack.push(result);
                }

                Op::Explain => {
                    let chain = vm_pop!(stack, events);
                    events.push(VmEvent::ExplainOrigin {
                        hash: chain.chain_hash(),
                    });
                    let _ = stack.push(chain);
                }

                // ── Device I/O opcodes ─────────────────────────────────────────
                // VM = side-effect free. Emit events → Runtime xử lý → HAL.
                // Đây là bridge: Olang → VmEvent → Runtime → HAL → phần cứng.

                Op::DeviceWrite(device_id) => {
                    let val_chain = vm_pop!(stack, events);
                    // Extract value: nếu là number chain → u8, nếu là mol → valence
                    let value = if let Some(n) = val_chain.to_number() {
                        n as u8
                    } else if let Some(mol) = val_chain.0.first() {
                        mol.emotion.valence
                    } else {
                        0
                    };
                    events.push(VmEvent::DeviceWrite {
                        device_id: device_id.clone(),
                        value,
                    });
                }

                Op::DeviceRead(device_id) => {
                    // Emit event — Runtime sẽ gọi HAL.device_read() và inject kết quả.
                    // Tạm push empty chain → Runtime sẽ replace.
                    events.push(VmEvent::DeviceRead {
                        device_id: device_id.clone(),
                    });
                    // Push placeholder — caller (Runtime) có thể inject actual value
                    let _ = stack.push(MolecularChain::empty());
                }

                Op::DeviceList => {
                    events.push(VmEvent::DeviceListRequest);
                }

                // ── FFI & System I/O ──────────────────────────────────────────

                Op::Ffi(name, arity) => {
                    let mut args = Vec::new();
                    for _ in 0..*arity {
                        args.push(vm_pop!(stack, events));
                    }
                    args.reverse(); // stack order → call order
                    events.push(VmEvent::FfiCall {
                        name: name.clone(),
                        args,
                    });
                    // Push placeholder — Runtime injects actual result
                    let _ = stack.push(MolecularChain::empty());
                }

                Op::FileRead => {
                    let path_chain = vm_pop!(stack, events);
                    let path = chain_to_string(&path_chain)
                        .unwrap_or_default();
                    events.push(VmEvent::FileReadRequest { path });
                    // Push placeholder — Runtime injects file contents
                    let _ = stack.push(MolecularChain::empty());
                }

                Op::FileWrite => {
                    let data_chain = vm_pop!(stack, events);
                    let path_chain = vm_pop!(stack, events);
                    let path = chain_to_string(&path_chain)
                        .unwrap_or_default();
                    let data = if let Some(s) = chain_to_string(&data_chain) {
                        s.into_bytes()
                    } else {
                        data_chain.to_tagged_bytes()
                    };
                    events.push(VmEvent::FileWriteRequest { path, data });
                }

                Op::FileAppend => {
                    let data_chain = vm_pop!(stack, events);
                    let path_chain = vm_pop!(stack, events);
                    let path = chain_to_string(&path_chain)
                        .unwrap_or_default();
                    let data = if let Some(s) = chain_to_string(&data_chain) {
                        s.into_bytes()
                    } else {
                        data_chain.to_tagged_bytes()
                    };
                    events.push(VmEvent::FileAppendRequest { path, data });
                }

                Op::SpawnBegin => {
                    // Count opcodes until SpawnEnd
                    let mut count = 0usize;
                    let mut search_pc = pc;
                    while search_pc < prog.ops.len() {
                        if matches!(prog.ops[search_pc], Op::SpawnEnd) {
                            break;
                        }
                        count += 1;
                        search_pc += 1;
                    }
                    events.push(VmEvent::SpawnRequest {
                        body_ops_count: count,
                    });
                    // Skip past SpawnEnd — the body will be executed by Runtime as async
                    pc = search_pc + 1;
                    continue;
                }

                Op::SpawnEnd => {
                    // Should not be reached — SpawnBegin skips past SpawnEnd
                }

                Op::TryBegin(catch_target) => {
                    // Push catch handler PC onto try stack
                    try_stack.push(*catch_target);
                }

                Op::CatchEnd => {
                    // End of catch block — pop try entry if still present
                    // (already popped if error occurred and caught)
                }

                Op::Closure(_param_count, body_len) => {
                    // Create closure: jump over body, push closure marker.
                    // Closure marker = chain with special encoding:
                    //   molecule.shape = 0xFF (closure tag)
                    //   molecule.relation = param_count
                    //   molecule.emotion.valence/arousal = body PC as 2 bytes (low/high)
                    let body_pc = pc;
                    let pc_low = (body_pc & 0xFF) as u8;
                    let pc_high = ((body_pc >> 8) & 0xFF) as u8;
                    let marker = MolecularChain::single(Molecule {
                        shape: 0xFF,
                        relation: *_param_count,
                        emotion: EmotionDim { valence: pc_low, arousal: pc_high },
                        time: 1,
                    });
                    let _ = stack.push(marker);
                    // Jump past the body
                    pc += body_len;
                }

                Op::CallClosure(arity) => {
                    // Call a closure: stack has [closure, arg1, arg2, ...]
                    let arity_val = *arity as usize;
                    // Pop args first (they're on top)
                    let mut closure_args = Vec::new();
                    for _ in 0..arity_val {
                        closure_args.push(vm_pop!(stack, events));
                    }
                    closure_args.reverse();
                    // Pop closure marker
                    let closure = vm_pop!(stack, events);
                    // Check if it's a closure marker (shape == 0xFF)
                    if let Some(mol) = closure.first() {
                        if mol.shape == 0xFF {
                            let body_pc = mol.emotion.valence as usize
                                | ((mol.emotion.arousal as usize) << 8);
                            // Push args back so body can Store them
                            for arg in closure_args.into_iter().rev() {
                                let _ = stack.push(arg);
                            }
                            // New scope for closure execution
                            scopes.push(Vec::new());
                            // Save current PC, jump to closure body
                            let saved_pc = pc;
                            pc = body_pc;
                            // Execute closure body inline until Ret
                            while pc < prog.ops.len() && steps < self.max_steps {
                                let op = &prog.ops[pc];
                                pc += 1;
                                steps += 1;
                                if matches!(op, Op::Ret) {
                                    break;
                                }
                                match op {
                                    Op::Store(name) => {
                                        let val = vm_pop!(stack, events);
                                        if let Some(scope) = scopes.last_mut() {
                                            if let Some(entry) = scope.iter_mut().find(|(n, _)| n == name) {
                                                entry.1 = val;
                                            } else {
                                                scope.push((name.clone(), val));
                                            }
                                        }
                                    }
                                    Op::LoadLocal(name) => {
                                        let val = scopes.iter().rev()
                                            .find_map(|s| s.iter().rev().find(|(n, _)| n == name).map(|(_, c)| c.clone()))
                                            .unwrap_or_else(MolecularChain::empty);
                                        let _ = stack.push(val);
                                    }
                                    Op::PushNum(n) => {
                                        let _ = stack.push(MolecularChain::from_number(*n));
                                    }
                                    Op::Push(chain) => {
                                        let _ = stack.push(chain.clone());
                                    }
                                    Op::Call(fname) => {
                                        match fname.as_str() {
                                            "__hyp_add" | "__hyp_sub" | "__hyp_mul" | "__hyp_div"
                                            | "__hyp_mod" | "__phys_add" | "__phys_sub" => {
                                                let b = vm_pop!(stack, events);
                                                let a = vm_pop!(stack, events);
                                                let fa = a.to_number().unwrap_or(0.0);
                                                let fb = b.to_number().unwrap_or(0.0);
                                                let result = match fname.as_str() {
                                                    "__hyp_add" | "__phys_add" => fa + fb,
                                                    "__hyp_sub" | "__phys_sub" => fa - fb,
                                                    "__hyp_mul" => fa * fb,
                                                    "__hyp_div" => if fb.abs() > f64::EPSILON { fa / fb } else { 0.0 },
                                                    "__hyp_mod" => if fb.abs() > f64::EPSILON { fa % fb } else { 0.0 },
                                                    _ => 0.0,
                                                };
                                                let _ = stack.push(MolecularChain::from_number(result));
                                            }
                                            "__cmp_lt" | "__cmp_gt" | "__cmp_le" | "__cmp_ge" | "__cmp_ne" => {
                                                let b = vm_pop!(stack, events);
                                                let a = vm_pop!(stack, events);
                                                let fa = a.to_number().unwrap_or(0.0);
                                                let fb = b.to_number().unwrap_or(0.0);
                                                let result = match fname.as_str() {
                                                    "__cmp_lt" => fa < fb,
                                                    "__cmp_gt" => fa > fb,
                                                    "__cmp_le" => fa <= fb,
                                                    "__cmp_ge" => fa >= fb,
                                                    "__cmp_ne" => (fa - fb).abs() >= f64::EPSILON,
                                                    _ => false,
                                                };
                                                let _ = stack.push(if result {
                                                    MolecularChain::from_number(1.0)
                                                } else {
                                                    MolecularChain::empty()
                                                });
                                            }
                                            _ => {
                                                events.push(VmEvent::LookupAlias(fname.clone()));
                                            }
                                        }
                                    }
                                    Op::Lca => {
                                        let b = vm_pop!(stack, events);
                                        let a = vm_pop!(stack, events);
                                        let _ = stack.push(lca(&a, &b));
                                    }
                                    Op::Pop => { let _ = vm_pop!(stack, events); }
                                    Op::Dup => {
                                        if let Some(top) = stack.peek() {
                                            let _ = stack.push(top.clone());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            scopes.pop(); // pop closure scope
                            pc = saved_pc;
                        } else {
                            let _ = stack.push(MolecularChain::empty());
                        }
                    } else {
                        let _ = stack.push(MolecularChain::empty());
                    }
                }

                Op::Halt => {
                    break;
                }
            }
        }

        // Check: if we broke due to error and have a try handler, resume there
        if !try_stack.is_empty() {
            // Check if last event was an error
            let has_error = events.iter().rev().any(|e| matches!(e, VmEvent::Error(_)));
            if has_error {
                if let Some(catch_pc) = try_stack.pop() {
                    // Remove the error event (caught)
                    if let Some(pos) = events.iter().rposition(|e| matches!(e, VmEvent::Error(_))) {
                        events.remove(pos);
                    }
                    // Resume at catch handler — re-enter execution loop
                    pc = catch_pc;
                    let mut remaining_steps = self.max_steps.saturating_sub(steps);
                    while pc < prog.ops.len() && remaining_steps > 0 {
                        remaining_steps -= 1;
                        steps += 1;
                        let op = &prog.ops[pc];
                        pc += 1;
                        match op {
                            Op::Halt | Op::CatchEnd => break,
                            Op::Emit => {
                                if let Ok(c) = stack.pop() {
                                    events.push(VmEvent::Output(c));
                                }
                            }
                            Op::Dream => events.push(VmEvent::TriggerDream),
                            Op::Stats => events.push(VmEvent::RequestStats),
                            Op::Load(name) => events.push(VmEvent::LookupAlias(name.clone())),
                            Op::PushNum(n) => {
                                let _ = stack.push(MolecularChain::from_number(*n));
                            }
                            Op::Push(chain) => {
                                let _ = stack.push(chain.clone());
                            }
                            Op::PushMol(s, r, v, a, t) => {
                                let mol = Molecule {
                                    shape: *s,
                                    relation: *r,
                                    emotion: EmotionDim { valence: *v, arousal: *a },
                                    time: *t,
                                };
                                let chain = MolecularChain(alloc::vec![mol]);
                                let _ = stack.push(chain);
                            }
                            Op::Dup => {
                                if let Some(top) = stack.data.last() {
                                    let c = top.clone();
                                    let _ = stack.push(c);
                                }
                            }
                            Op::Store(name) => {
                                if let Ok(c) = stack.pop() {
                                    if let Some(scope) = scopes.last_mut() {
                                        if let Some(entry) = scope.iter_mut().find(|(n, _)| n == name) {
                                            entry.1 = c;
                                        } else {
                                            scope.push((name.clone(), c));
                                        }
                                    }
                                }
                            }
                            Op::LoadLocal(name) => {
                                for scope in scopes.iter().rev() {
                                    if let Some((_, c)) = scope.iter().find(|(n, _)| n == name) {
                                        let _ = stack.push(c.clone());
                                        break;
                                    }
                                }
                            }
                            Op::StoreUpdate(name) => {
                                if let Ok(c) = stack.pop() {
                                    let mut found = false;
                                    for scope in scopes.iter_mut().rev() {
                                        if let Some(entry) = scope.iter_mut().find(|(n, _)| n == name) {
                                            entry.1 = c.clone();
                                            found = true;
                                            break;
                                        }
                                    }
                                    if !found {
                                        if let Some(scope) = scopes.last_mut() {
                                            scope.push((name.clone(), c));
                                        }
                                    }
                                }
                            }
                            Op::Nop | Op::ScopeBegin | Op::ScopeEnd => {}
                            Op::Pop => { let _ = stack.pop(); }
                            _ => {} // other ops skipped in catch recovery
                        }
                    }
                }
            }
        }

        VmResult {
            events,
            steps,
            stack_depth: stack.data.len(),
        }
    }
}

impl Default for OlangVM {
    fn default() -> Self {
        Self::new()
    }
}

/// Kết quả execute.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct VmResult {
    pub events: Vec<VmEvent>,
    pub steps: u32,
    pub stack_depth: usize,
}

#[allow(missing_docs)]
impl VmResult {
    pub fn outputs(&self) -> Vec<&MolecularChain> {
        self.events
            .iter()
            .filter_map(|e| {
                if let VmEvent::Output(c) = e {
                    Some(c)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn has_error(&self) -> bool {
        self.events.iter().any(|e| matches!(e, VmEvent::Error(_)))
    }

    pub fn errors(&self) -> Vec<&VmError> {
        self.events
            .iter()
            .filter_map(|e| {
                if let VmEvent::Error(err) = e {
                    Some(err)
                } else {
                    None
                }
            })
            .collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::encode_codepoint;
    use crate::ir::{compile_expr, OlangIrExpr};

    fn vm() -> OlangVM {
        OlangVM::new()
    }

    // ── Basic execution ──────────────────────────────────────────────────────

    #[test]
    fn execute_halt_immediately() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert_eq!(result.steps, 1);
        assert!(!result.has_error());
    }

    #[test]
    fn execute_nop() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Nop).push_op(Op::Nop).push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert_eq!(result.steps, 3);
    }

    #[test]
    fn execute_push_emit() {
        let chain = encode_codepoint(0x1F525); // 🔥
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(chain.clone()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1);
        assert_eq!(*outputs[0], chain);
    }

    #[test]
    fn execute_lca() {
        let fire = encode_codepoint(0x1F525);
        let water = encode_codepoint(0x1F4A7);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(fire))
            .push_op(Op::Push(water))
            .push_op(Op::Lca)
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1);
        assert!(!outputs[0].is_empty(), "LCA output không rỗng");
    }

    #[test]
    fn execute_dup() {
        let chain = encode_codepoint(0x1F525);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(chain))
            .push_op(Op::Dup)
            .push_op(Op::Emit) // emit copy
            .push_op(Op::Emit) // emit original
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert_eq!(result.outputs().len(), 2, "DUP → 2 outputs");
    }

    #[test]
    fn execute_swap() {
        let fire = encode_codepoint(0x1F525);
        let water = encode_codepoint(0x1F4A7);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(fire.clone()))
            .push_op(Op::Push(water.clone()))
            .push_op(Op::Swap)
            .push_op(Op::Emit) // emit fire (sau swap)
            .push_op(Op::Emit) // emit water
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert_eq!(result.outputs().len(), 2);
        // Sau swap: stack = [fire, water] → emit water trước, fire sau?
        // Không — swap đổi top 2: push fire, push water → swap → [water, fire]
        // Emit → fire (top), emit → water
        assert_eq!(*result.outputs()[0], fire);
    }

    #[test]
    fn execute_load_emits_event() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Load("fire".into())).push_op(Op::Halt);
        let result = vm().execute(&prog);
        let has_lookup = result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::LookupAlias(s) if s == "fire"));
        assert!(has_lookup, "LOAD phải emit LookupAlias event");
    }

    #[test]
    fn execute_edge_emits_event() {
        let fire = encode_codepoint(0x1F525);
        let water = encode_codepoint(0x1F4A7);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(fire))
            .push_op(Op::Push(water))
            .push_op(Op::Edge(0x06)) // Causes relation
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let has_edge = result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::CreateEdge { rel: 0x06, .. }));
        assert!(has_edge, "EDGE phải emit CreateEdge event");
    }

    #[test]
    fn execute_dream_event() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Dream).push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::TriggerDream)));
    }

    #[test]
    fn execute_stats_event() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Stats).push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::RequestStats)));
    }

    // ── Control flow ─────────────────────────────────────────────────────────

    #[test]
    fn execute_jmp() {
        let chain = encode_codepoint(0x1F525);
        // JMP 2 → skip PUSH → chỉ HALT
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Jmp(2)) // 0: jump to 2
            .push_op(Op::Push(chain.clone())) // 1: SKIP
            .push_op(Op::Halt); // 2: halt
        let result = vm().execute(&prog);
        assert_eq!(result.outputs().len(), 0, "JMP phải skip PUSH");
    }

    #[test]
    fn execute_max_steps_guard() {
        // Vòng lặp vô hạn → bị chặn bởi max_steps
        let vm = OlangVM::with_max_steps(10);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Jmp(0)); // infinite loop
        let result = vm.execute(&prog);
        assert!(result.has_error(), "Infinite loop phải bị chặn");
        assert!(result.errors().contains(&&VmError::MaxStepsExceeded));
    }

    #[test]
    fn execute_stack_underflow() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Pop) // pop khi stack rỗng
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result.has_error(), "Pop khi rỗng → error");
    }

    #[test]
    fn execute_invalid_jump() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Jmp(9999)) // target không tồn tại
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result.has_error());
    }

    // ── Compile + Execute ────────────────────────────────────────────────────

    #[test]
    fn compile_and_execute_query() {
        let expr = OlangIrExpr::Query("fire".into());
        let prog = compile_expr(&expr);
        let result = vm().execute(&prog);
        // LOAD → LookupAlias event
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::LookupAlias(s) if s == "fire")));
    }

    #[test]
    fn compile_and_execute_compose() {
        let expr = OlangIrExpr::Compose("fire".into(), "water".into());
        let prog = compile_expr(&expr);
        // Phải có: LOAD fire, LOAD water, LCA, EMIT, HALT
        assert_eq!(prog.ops.len(), 5);
        let result = vm().execute(&prog);
        // 2 LookupAlias events (fire, water)
        let lookups: Vec<_> = result
            .events
            .iter()
            .filter_map(|e| {
                if let VmEvent::LookupAlias(s) = e {
                    Some(s.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert!(lookups.contains(&"fire"));
        assert!(lookups.contains(&"water"));
    }

    #[test]
    fn compile_and_execute_dream() {
        let expr = OlangIrExpr::Command("dream".into());
        let prog = compile_expr(&expr);
        let result = vm().execute(&prog);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::TriggerDream)));
    }

    // ── Reasoning & Debug primitives ────────────────────────────────────────

    #[test]
    fn execute_trace() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Trace).push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::TraceStep { .. })));
    }

    #[test]
    fn execute_inspect() {
        let chain = encode_codepoint(0x1F525); // 🔥
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(chain))
            .push_op(Op::Inspect)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let has_inspect = result.events.iter().any(|e| {
            matches!(
                e,
                VmEvent::InspectChain {
                    is_empty: false,
                    ..
                }
            )
        });
        assert!(has_inspect, "INSPECT phải emit InspectChain event");
        // Chain should still be on stack (inspect doesn't consume)
        assert_eq!(result.stack_depth, 1);
    }

    #[test]
    fn execute_assert_pass() {
        let chain = encode_codepoint(0x1F525);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(chain))
            .push_op(Op::Assert)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        // No AssertFailed since chain is non-empty
        assert!(!result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::AssertFailed)));
    }

    #[test]
    fn execute_assert_fail() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(MolecularChain::empty()))
            .push_op(Op::Assert)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::AssertFailed)));
    }

    #[test]
    fn execute_typeof() {
        let chain = encode_codepoint(0x1F525);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(chain))
            .push_op(Op::TypeOf)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::TypeInfo { .. })));
    }

    #[test]
    fn execute_explain() {
        let chain = encode_codepoint(0x1F525);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(chain))
            .push_op(Op::Explain)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::ExplainOrigin { .. })));
        // Chain still on stack
        assert_eq!(result.stack_depth, 1);
    }

    #[test]
    fn execute_why() {
        let fire = encode_codepoint(0x1F525);
        let water = encode_codepoint(0x1F4A7);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(fire))
            .push_op(Op::Push(water))
            .push_op(Op::Why)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::WhyConnection { .. })));
        // LCA result on stack
        assert_eq!(result.stack_depth, 1);
    }

    // ── Phase 1: Math Runtime — arithmetic execution ────────────────────────

    #[test]
    fn math_addition() {
        // 1 + 2 = 3
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(1.0))
            .push_op(Op::PushNum(2.0))
            .push_op(Op::Call("__hyp_add".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1);
        let n = outputs[0].to_number().unwrap();
        assert!((n - 3.0).abs() < f64::EPSILON, "1 + 2 should = 3, got {}", n);
    }

    #[test]
    fn math_subtraction() {
        // 10 - 3 = 7
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(10.0))
            .push_op(Op::PushNum(3.0))
            .push_op(Op::Call("__hyp_sub".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn math_multiplication() {
        // 6 × 7 = 42
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(6.0))
            .push_op(Op::PushNum(7.0))
            .push_op(Op::Call("__hyp_mul".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn math_division() {
        // 10 ÷ 4 = 2.5
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(10.0))
            .push_op(Op::PushNum(4.0))
            .push_op(Op::Call("__hyp_div".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn math_division_by_zero() {
        // 5 ÷ 0 → error
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(5.0))
            .push_op(Op::PushNum(0.0))
            .push_op(Op::Call("__hyp_div".into()))
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result.has_error(), "Division by zero should error");
        assert!(
            result.errors().contains(&&VmError::DivisionByZero),
            "Should be DivisionByZero, not StackUnderflow"
        );
    }

    #[test]
    fn math_physical_add() {
        // Physical add: same as hyp_add but for proven values
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(100.0))
            .push_op(Op::PushNum(50.0))
            .push_op(Op::Call("__phys_add".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn math_truth_equal() {
        // fire == fire → truthy (push back)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(42.0))
            .push_op(Op::PushNum(42.0))
            .push_op(Op::Call("__assert_truth".into()))
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        // Stack should have non-empty chain (truthy)
        assert_eq!(result.stack_depth, 1);
    }

    #[test]
    fn math_truth_not_equal() {
        // 1 == 2 → falsy (empty chain)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(1.0))
            .push_op(Op::PushNum(2.0))
            .push_op(Op::Call("__assert_truth".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1);
        assert!(outputs[0].is_empty(), "1 == 2 should be falsy (empty chain)");
    }

    #[test]
    fn math_chained_operations() {
        // (2 + 3) * 4 = 20
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(2.0))
            .push_op(Op::PushNum(3.0))
            .push_op(Op::Call("__hyp_add".into()))
            .push_op(Op::PushNum(4.0))
            .push_op(Op::Call("__hyp_mul".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 20.0).abs() < f64::EPSILON, "(2+3)*4 = {}", n);
    }

    #[test]
    fn math_negative_result() {
        // 3 - 7 = -4
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(3.0))
            .push_op(Op::PushNum(7.0))
            .push_op(Op::Call("__hyp_sub".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - (-4.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn math_pushnum_roundtrip() {
        // PushNum → Emit → to_number roundtrip
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(3.14159))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 3.14159).abs() < 1e-10);
    }

    // ── Scope tests ─────────────────────────────────────────────────────────

    #[test]
    fn scope_basic_store_load() {
        // Store x=5 in root scope, load x → emit
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(5.0))
            .push_op(Op::Store("x".into()))
            .push_op(Op::LoadLocal("x".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn scope_nested_shadows_outer() {
        // Root: x = 10
        // Inner scope: x = 20, emit x (should be 20)
        // After scope end: emit x (should be 10 again)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(10.0))
            .push_op(Op::Store("x".into()))
            .push_op(Op::ScopeBegin)
            .push_op(Op::PushNum(20.0))
            .push_op(Op::Store("x".into()))
            .push_op(Op::LoadLocal("x".into()))
            .push_op(Op::Emit) // should output 20
            .push_op(Op::ScopeEnd)
            .push_op(Op::LoadLocal("x".into()))
            .push_op(Op::Emit) // should output 10 (inner x discarded)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 2);
        let inner = outputs[0].to_number().unwrap();
        let outer = outputs[1].to_number().unwrap();
        assert!((inner - 20.0).abs() < f64::EPSILON, "Inner scope x=20: {}", inner);
        assert!((outer - 10.0).abs() < f64::EPSILON, "Outer scope x=10: {}", outer);
    }

    #[test]
    fn scope_inner_reads_outer() {
        // Root: y = 42
        // Inner scope: load y (should find in outer scope)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(42.0))
            .push_op(Op::Store("y".into()))
            .push_op(Op::ScopeBegin)
            .push_op(Op::LoadLocal("y".into()))
            .push_op(Op::Emit)
            .push_op(Op::ScopeEnd)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 42.0).abs() < f64::EPSILON, "Inner reads outer y=42");
    }

    #[test]
    fn scope_double_nesting() {
        // Root: a = 1
        // Scope 1: b = 2
        // Scope 2: c = 3, emit a+b+c
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(1.0))
            .push_op(Op::Store("a".into()))
            .push_op(Op::ScopeBegin) // scope 1
            .push_op(Op::PushNum(2.0))
            .push_op(Op::Store("b".into()))
            .push_op(Op::ScopeBegin) // scope 2
            .push_op(Op::PushNum(3.0))
            .push_op(Op::Store("c".into()))
            // a + b
            .push_op(Op::LoadLocal("a".into()))
            .push_op(Op::LoadLocal("b".into()))
            .push_op(Op::Call("__hyp_add".into()))
            // + c
            .push_op(Op::LoadLocal("c".into()))
            .push_op(Op::Call("__hyp_add".into()))
            .push_op(Op::Emit) // 1+2+3 = 6
            .push_op(Op::ScopeEnd) // pop scope 2
            .push_op(Op::ScopeEnd) // pop scope 1
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 6.0).abs() < f64::EPSILON, "a+b+c = 6: {}", n);
    }

    #[test]
    fn scope_end_without_begin_safe() {
        // ScopeEnd without Begin should not crash (root scope protected)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::ScopeEnd)
            .push_op(Op::ScopeEnd)
            .push_op(Op::PushNum(1.0))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert_eq!(result.outputs().len(), 1);
    }

    #[test]
    fn scope_undefined_var_returns_empty() {
        // Loading undefined variable → empty chain
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::LoadLocal("nonexistent".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result.outputs()[0].is_empty(), "Undefined var → empty");
    }

    // ── Recursion depth limit ───────────────────────────────────────────────

    #[test]
    fn call_depth_exceeded_triggers_error() {
        // Build program with deeply nested ScopeBegin without ScopeEnd
        let mut prog = OlangProgram::new("test");
        for _ in 0..260 {
            prog.push_op(Op::ScopeBegin);
        }
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result.has_error(), "Should error on depth > 256");
        let has_depth_err = result.events.iter().any(|e| {
            matches!(e, VmEvent::Error(VmError::MaxCallDepthExceeded))
        });
        assert!(has_depth_err, "Should have MaxCallDepthExceeded error");
    }

    #[test]
    fn call_depth_within_limit_ok() {
        // 10 nested scopes should be fine
        let mut prog = OlangProgram::new("test");
        for _ in 0..10 {
            prog.push_op(Op::ScopeBegin);
        }
        prog.push_op(Op::PushNum(42.0));
        prog.push_op(Op::Emit);
        for _ in 0..10 {
            prog.push_op(Op::ScopeEnd);
        }
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "10 nested scopes should work");
        assert_eq!(result.outputs().len(), 1);
    }

    #[test]
    fn call_depth_decrements_on_scope_end() {
        // Open 5 scopes, close 5, open 5 more → should be within limit
        let mut prog = OlangProgram::new("test");
        for _ in 0..5 {
            prog.push_op(Op::ScopeBegin);
        }
        for _ in 0..5 {
            prog.push_op(Op::ScopeEnd);
        }
        for _ in 0..5 {
            prog.push_op(Op::ScopeBegin);
        }
        prog.push_op(Op::PushNum(99.0));
        prog.push_op(Op::Emit);
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "Depth should reset after ScopeEnd");
    }

    // ── Loop ──────────────────────────────────────────────────────────────

    #[test]
    fn loop_basic_3_times() {
        // Loop 3 times: emit 1.0 each iteration → 3 outputs
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Loop(3));
        prog.push_op(Op::ScopeBegin);
        prog.push_op(Op::PushNum(1.0));
        prog.push_op(Op::Emit);
        prog.push_op(Op::ScopeEnd);
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "Loop 3 should not error");
        assert_eq!(result.outputs().len(), 3, "3 iterations → 3 outputs");
    }

    #[test]
    fn loop_once_same_as_no_loop() {
        // Loop(1) = execute body once (no repeat)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Loop(1));
        prog.push_op(Op::ScopeBegin);
        prog.push_op(Op::PushNum(42.0));
        prog.push_op(Op::Emit);
        prog.push_op(Op::ScopeEnd);
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert_eq!(result.outputs().len(), 1);
    }

    #[test]
    fn loop_zero_no_body() {
        // Loop(0) = skip body entirely? No — falls through once, no repeat.
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Loop(0));
        prog.push_op(Op::ScopeBegin);
        prog.push_op(Op::PushNum(1.0));
        prog.push_op(Op::Emit);
        prog.push_op(Op::ScopeEnd);
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        // Loop(0).min(1024) = 0, so no push to loop_stack, body runs once (fall-through)
        assert_eq!(result.outputs().len(), 1, "Loop(0) falls through once");
    }

    #[test]
    fn loop_capped_at_1024() {
        // Loop(u32::MAX) should be capped to 1024
        let vm = OlangVM::with_max_steps(65_536);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Loop(u32::MAX));
        prog.push_op(Op::ScopeBegin);
        prog.push_op(Op::Nop); // lightweight body
        prog.push_op(Op::ScopeEnd);
        prog.push_op(Op::Halt);
        let result = vm.execute(&prog);
        // Should complete without MaxStepsExceeded (1024 * 3 ops = 3072 < 65536)
        assert!(!result.has_error(), "Capped loop should complete");
    }

    // ── PushMol — molecular literal execution ──────────────────────────────

    #[test]
    fn push_mol_creates_chain() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushMol(1, 6, 200, 180, 4));
        prog.push_op(Op::Emit);
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "PushMol should not error");
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1, "One chain emitted");
        let chain = outputs[0];
        assert_eq!(chain.len(), 1, "Chain has exactly 1 molecule");
        let mol = chain.first().unwrap();
        assert_eq!(mol.shape, 1);
        assert_eq!(mol.relation, 6);
        assert_eq!(mol.emotion.valence, 200);
        assert_eq!(mol.emotion.arousal, 180);
        assert_eq!(mol.time, 4);
    }

    #[test]
    fn push_mol_default_values() {
        // Defaults from semantic: S=1, R=1, V=128, A=128, T=3
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushMol(1, 1, 128, 128, 3));
        prog.push_op(Op::Emit);
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let mol = result.outputs()[0].first().unwrap();
        assert_eq!(mol.shape, 1);
        assert_eq!(mol.relation, 1);
        assert_eq!(mol.emotion.valence, 128);
        assert_eq!(mol.emotion.arousal, 128);
        assert_eq!(mol.time, 3);
    }

    #[test]
    fn push_mol_then_lca() {
        // Two molecular literals → LCA → single output
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushMol(1, 6, 200, 180, 4));
        prog.push_op(Op::PushMol(2, 3, 100, 90, 2));
        prog.push_op(Op::Lca);
        prog.push_op(Op::Emit);
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "LCA of two mol literals should work");
        assert_eq!(result.outputs().len(), 1);
        assert!(!result.outputs()[0].is_empty());
    }

    #[test]
    fn push_mol_dup_and_truth() {
        // PushMol → Dup → Truth (==) → should produce 1 output
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushMol(1, 6, 200, 180, 4));
        prog.push_op(Op::Dup);
        prog.push_op(Op::Call("__assert_truth".into()));
        prog.push_op(Op::Emit);
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        // Truth of identical chains → non-empty output
        assert_eq!(result.outputs().len(), 1);
        assert!(!result.outputs()[0].is_empty(), "Same chain == same chain should be truthy");
    }

    #[test]
    fn match_mol_same() {
        // __match_mol: compare two identical PushMol chains → truthy
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushMol(1, 6, 200, 180, 4))
            .push_op(Op::PushMol(1, 6, 200, 180, 4))
            .push_op(Op::Call("__match_mol".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(!result.outputs()[0].is_empty(), "Same mol should match");
    }

    #[test]
    fn match_mol_different() {
        // __match_mol: different mols → falsy (empty)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushMol(1, 6, 200, 180, 4))
            .push_op(Op::PushMol(2, 3, 100, 50, 1))
            .push_op(Op::Call("__match_mol".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(result.outputs()[0].is_empty(), "Different mol should not match");
    }

    #[test]
    fn try_catch_no_error() {
        // try { emit 42 } catch { emit 99 }
        // No error → should output 42, not 99
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::TryBegin(5))  // catch at position 5
            .push_op(Op::PushNum(42.0))
            .push_op(Op::Emit)
            .push_op(Op::Jmp(7))        // skip catch
            .push_op(Op::CatchEnd)       // 4 (should not execute)
            .push_op(Op::PushNum(99.0))  // 5: catch body
            .push_op(Op::Emit)           // 6
            .push_op(Op::Halt);          // 7
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1);
        let n = outputs[0].to_number().unwrap();
        assert!((n - 42.0).abs() < f64::EPSILON, "Should output 42, got {}", n);
    }

    #[test]
    fn try_catch_with_error_recovery() {
        // try { pop (underflow!) } catch { emit 99 }
        // Error in try → should recover and emit from catch
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::TryBegin(4))  // catch at position 4
            .push_op(Op::Pop)           // underflow! → error
            .push_op(Op::Jmp(7))        // skip catch
            .push_op(Op::Halt)          // 3 (fallthrough guard)
            .push_op(Op::PushNum(99.0)) // 4: catch body
            .push_op(Op::Emit)          // 5
            .push_op(Op::CatchEnd)      // 6
            .push_op(Op::Halt);         // 7
        let result = vm().execute(&prog);
        // Error should be caught — no errors in result
        assert!(!result.has_error(), "Error should be caught: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1, "Should have catch output");
        let n = outputs[0].to_number().unwrap();
        assert!((n - 99.0).abs() < f64::EPSILON, "Catch should output 99, got {}", n);
    }

    #[test]
    fn for_in_emits_counter_values() {
        // for i in 0..3 { emit i }
        // Counter on stack, DUP into scoped var each iteration.
        // Should output: 0.0, 1.0, 2.0
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(0.0));          // counter = 0 on stack
        prog.push_op(Op::Loop(3));               // 3 iterations
        prog.push_op(Op::ScopeBegin);
        prog.push_op(Op::Dup);                   // dup counter for body
        prog.push_op(Op::Store("i".into()));     // body can use i
        prog.push_op(Op::LoadLocal("i".into())); // load counter var
        prog.push_op(Op::Emit);                  // emit it
        // Increment counter on stack
        prog.push_op(Op::PushNum(1.0));
        prog.push_op(Op::Call("__hyp_add".into()));
        prog.push_op(Op::ScopeEnd);              // destroys scope, triggers loop
        prog.push_op(Op::Pop);                   // discard counter after loop
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "for-in error: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 3, "Should emit 3 values, got {}", outputs.len());
        let v0 = outputs[0].to_number().unwrap();
        let v1 = outputs[1].to_number().unwrap();
        let v2 = outputs[2].to_number().unwrap();
        assert!((v0 - 0.0).abs() < f64::EPSILON, "First should be 0, got {}", v0);
        assert!((v1 - 1.0).abs() < f64::EPSILON, "Second should be 1, got {}", v1);
        assert!((v2 - 2.0).abs() < f64::EPSILON, "Third should be 2, got {}", v2);
    }

    #[test]
    fn cmp_lt_true() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(3.0))
            .push_op(Op::PushNum(5.0))
            .push_op(Op::Call("__cmp_lt".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 1.0).abs() < f64::EPSILON, "3 < 5 should be true (1.0)");
    }

    #[test]
    fn cmp_lt_false() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(7.0))
            .push_op(Op::PushNum(5.0))
            .push_op(Op::Call("__cmp_lt".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        // False comparisons return empty chain (falsy for Jz)
        assert!(result.outputs()[0].is_empty(), "7 < 5 should be false (empty)");
    }

    #[test]
    fn cmp_ge_true() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(5.0))
            .push_op(Op::PushNum(5.0))
            .push_op(Op::Call("__cmp_ge".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 1.0).abs() < f64::EPSILON, "5 >= 5 should be true");
    }

    #[test]
    fn while_loop_counts_to_three() {
        // while i < 3 { emit i; i = i + 1 }
        // Simulate: counter on stack, Loop(1024), cond check via __cmp_lt + Jz
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(0.0));          // counter on stack
        prog.push_op(Op::Loop(1024));
        prog.push_op(Op::ScopeBegin);
        // Check: counter < 3?
        prog.push_op(Op::Dup);                    // dup counter for cmp
        prog.push_op(Op::PushNum(3.0));
        prog.push_op(Op::Call("__cmp_lt".into()));
        let jz_idx = prog.ops.len();
        prog.push_op(Op::Jz(0));                  // placeholder → jumps to end
        prog.push_op(Op::Pop);                    // pop cmp result (true path)
        // Body: emit counter
        prog.push_op(Op::Dup);
        prog.push_op(Op::Emit);
        // Increment
        prog.push_op(Op::PushNum(1.0));
        prog.push_op(Op::Call("__hyp_add".into()));
        prog.push_op(Op::ScopeEnd);               // loop back
        let end = prog.ops.len();
        prog.ops[jz_idx] = Op::Jz(end);
        prog.push_op(Op::Pop);                    // pop cmp result (false path, jumped here)
        prog.push_op(Op::Pop);                    // discard counter
        prog.push_op(Op::Halt);

        let result = vm().execute(&prog);
        assert!(!result.has_error(), "while errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 3, "Should emit 3 values");
        let v0 = outputs[0].to_number().unwrap();
        let v1 = outputs[1].to_number().unwrap();
        let v2 = outputs[2].to_number().unwrap();
        assert!((v0 - 0.0).abs() < f64::EPSILON);
        assert!((v1 - 1.0).abs() < f64::EPSILON);
        assert!((v2 - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cmp_ne_true() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(3.0))
            .push_op(Op::PushNum(5.0))
            .push_op(Op::Call("__cmp_ne".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 1.0).abs() < f64::EPSILON, "3 != 5 should be true");
    }

    #[test]
    fn cmp_ne_false() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(5.0))
            .push_op(Op::PushNum(5.0))
            .push_op(Op::Call("__cmp_ne".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(result.outputs()[0].is_empty(), "5 != 5 should be false");
    }

    #[test]
    fn logic_not_empty_becomes_truthy() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(MolecularChain::empty()))
            .push_op(Op::Call("__logic_not".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 1.0).abs() < f64::EPSILON, "!empty should be truthy (1.0)");
    }

    #[test]
    fn logic_not_truthy_becomes_empty() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(42.0))
            .push_op(Op::Call("__logic_not".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(result.outputs()[0].is_empty(), "!42 should be empty (falsy)");
    }

    // ── StoreUpdate ──────────────────────────────────────────────────────────

    #[test]
    fn store_update_modifies_outer_scope() {
        // Root: x = 10
        // Inner scope: StoreUpdate x = 20, emit x (should be 20)
        // After scope end: emit x (should be 20 — updated in outer scope)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(10.0))
            .push_op(Op::Store("x".into()))
            .push_op(Op::ScopeBegin)
            .push_op(Op::PushNum(20.0))
            .push_op(Op::StoreUpdate("x".into()))
            .push_op(Op::LoadLocal("x".into()))
            .push_op(Op::Emit) // should output 20
            .push_op(Op::ScopeEnd)
            .push_op(Op::LoadLocal("x".into()))
            .push_op(Op::Emit) // should output 20 (outer scope was updated)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "StoreUpdate errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 2);
        let inner = outputs[0].to_number().unwrap();
        let outer = outputs[1].to_number().unwrap();
        assert!((inner - 20.0).abs() < f64::EPSILON, "Inner scope sees 20: {}", inner);
        assert!((outer - 20.0).abs() < f64::EPSILON, "Outer scope updated to 20: {}", outer);
    }

    #[test]
    fn store_update_vs_store_shadowing() {
        // Store shadows (creates new in inner scope), StoreUpdate modifies outer.
        // Root: x = 10
        // Inner scope: Store(x)=99 → shadows outer
        // After scope end: emit x (should be 10, shadow discarded)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(10.0))
            .push_op(Op::Store("x".into()))
            .push_op(Op::ScopeBegin)
            .push_op(Op::PushNum(99.0))
            .push_op(Op::Store("x".into())) // shadows, not updates outer
            .push_op(Op::ScopeEnd)
            .push_op(Op::LoadLocal("x".into()))
            .push_op(Op::Emit) // should output 10
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let outer = result.outputs()[0].to_number().unwrap();
        assert!((outer - 10.0).abs() < f64::EPSILON, "Store shadows, outer still 10: {}", outer);
    }

    // ── Array builtins ───────────────────────────────────────────────────────

    #[test]
    fn array_new_and_get() {
        // [10, 20, 30] then get index 1
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(10.0))
            .push_op(Op::PushNum(20.0))
            .push_op(Op::PushNum(30.0))
            .push_op(Op::PushNum(3.0))
            .push_op(Op::Call("__array_new".into()))
            .push_op(Op::Store("arr".into()))
            .push_op(Op::LoadLocal("arr".into()))
            .push_op(Op::PushNum(1.0))
            .push_op(Op::Call("__array_get".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "array errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1);
        let v = outputs[0].to_number();
        assert!(v.is_some(), "arr[1] should be number, mol_count={}", outputs[0].len());
        assert!((v.unwrap() - 20.0).abs() < f64::EPSILON, "arr[1]=20, got {}", v.unwrap());
    }

    #[test]
    fn array_len() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(10.0))
            .push_op(Op::PushNum(20.0))
            .push_op(Op::PushNum(30.0))
            .push_op(Op::PushNum(3.0))
            .push_op(Op::Call("__array_new".into()))
            .push_op(Op::Call("__array_len".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "errors: {:?}", result.errors());
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 3.0).abs() < f64::EPSILON, "len=3, got {}", v);
    }

    // ── Dict builtins ────────────────────────────────────────────────────────

    fn key_chain(s: &str) -> MolecularChain {
        let mut mols = alloc::vec::Vec::new();
        for b in s.bytes() {
            mols.push(Molecule {
                shape: 0x02, relation: 0x01,
                emotion: EmotionDim { valence: b, arousal: 0 },
                time: 0x01,
            });
        }
        MolecularChain(mols)
    }

    #[test]
    fn dict_new_and_get() {
        // Create dict {key1: 10, key2: 20}, get key2 → 20
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(key_chain("key1")))
            .push_op(Op::PushNum(10.0))
            .push_op(Op::Push(key_chain("key2")))
            .push_op(Op::PushNum(20.0))
            .push_op(Op::PushNum(2.0))
            .push_op(Op::Call("__dict_new".into()))
            .push_op(Op::Store("d".into()))
            // Get key2
            .push_op(Op::LoadLocal("d".into()))
            .push_op(Op::Push(key_chain("key2")))
            .push_op(Op::Call("__dict_get".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(!outputs.is_empty(), "Should have output");
        let v = outputs[0].to_number().unwrap();
        assert!((v - 20.0).abs() < f64::EPSILON, "dict.key2 should be 20, got {}", v);
    }

    #[test]
    fn dict_set_updates_value() {
        // Create dict {key1: 10}, set key1 = 99, get key1 → 99
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(key_chain("key1")))
            .push_op(Op::PushNum(10.0))
            .push_op(Op::PushNum(1.0))
            .push_op(Op::Call("__dict_new".into()))
            // Set key1 = 99
            .push_op(Op::Push(key_chain("key1")))
            .push_op(Op::PushNum(99.0))
            .push_op(Op::Call("__dict_set".into()))
            .push_op(Op::Store("d".into()))
            // Get key1
            .push_op(Op::LoadLocal("d".into()))
            .push_op(Op::Push(key_chain("key1")))
            .push_op(Op::Call("__dict_get".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "errors: {:?}", result.errors());
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 99.0).abs() < f64::EPSILON, "dict.key1 should be 99, got {}", v);
    }

    // ── Phase 5: String Builtins ────────────────────────────────────────────

    fn str_chain(s: &str) -> MolecularChain {
        let mut mols = Vec::new();
        for b in s.bytes() {
            mols.push(Molecule {
                shape: 0x02, relation: 0x01,
                emotion: EmotionDim { valence: b, arousal: 0 },
                time: 0x01,
            });
        }
        MolecularChain(mols)
    }

    #[test]
    fn str_contains_found() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(str_chain("hello world")))
            .push_op(Op::Push(str_chain("world")))
            .push_op(Op::Call("__str_contains".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(!result.outputs()[0].is_empty(), "Should find 'world'");
    }

    #[test]
    fn str_contains_not_found() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(str_chain("hello")))
            .push_op(Op::Push(str_chain("xyz")))
            .push_op(Op::Call("__str_contains".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(result.outputs()[0].is_empty(), "Should not find 'xyz'");
    }

    #[test]
    fn str_starts_with_true() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(str_chain("hello world")))
            .push_op(Op::Push(str_chain("hello")))
            .push_op(Op::Call("__str_starts_with".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.outputs()[0].is_empty());
    }

    #[test]
    fn str_index_of_found() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(str_chain("abcdef")))
            .push_op(Op::Push(str_chain("cd")))
            .push_op(Op::Call("__str_index_of".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 2.0).abs() < f64::EPSILON, "index of 'cd' = 2, got {}", v);
    }

    #[test]
    fn str_replace_basic() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(str_chain("hello world")))
            .push_op(Op::Push(str_chain("world")))
            .push_op(Op::Push(str_chain("olang")))
            .push_op(Op::Call("__str_replace".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let out = &result.outputs()[0];
        // Result should be "hello olang" encoded as molecules
        let decoded: Vec<u8> = out.0.iter().map(|m| m.emotion.valence).collect();
        assert_eq!(&decoded, b"hello olang");
    }

    #[test]
    fn str_split_basic() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(str_chain("a,b,c")))
            .push_op(Op::Push(str_chain(",")))
            .push_op(Op::Call("__str_split".into()))
            .push_op(Op::Call("__array_len".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 3.0).abs() < f64::EPSILON, "split 'a,b,c' by ',' = 3 parts, got {}", v);
    }

    #[test]
    fn str_trim_whitespace() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(str_chain("  hello  ")))
            .push_op(Op::Call("__str_trim".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let decoded: Vec<u8> = result.outputs()[0].0.iter().map(|m| m.emotion.valence).collect();
        assert_eq!(&decoded, b"hello");
    }

    #[test]
    fn str_upper_lower() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(str_chain("Hello")))
            .push_op(Op::Call("__str_upper".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let decoded: Vec<u8> = result.outputs()[0].0.iter().map(|m| m.emotion.valence).collect();
        assert_eq!(&decoded, b"HELLO");
    }

    // ── Phase 5: Math Builtins ──────────────────────────────────────────────

    #[test]
    fn math_floor() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(3.7))
            .push_op(Op::Call("__hyp_floor".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 3.0).abs() < f64::EPSILON, "floor(3.7)=3, got {}", v);
    }

    #[test]
    fn math_ceil() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(3.2))
            .push_op(Op::Call("__hyp_ceil".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 4.0).abs() < f64::EPSILON, "ceil(3.2)=4, got {}", v);
    }

    #[test]
    fn math_sqrt() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(16.0))
            .push_op(Op::Call("__hyp_sqrt".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 4.0).abs() < 0.01, "sqrt(16)=4, got {}", v);
    }

    #[test]
    fn math_pow() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(2.0))
            .push_op(Op::PushNum(8.0))
            .push_op(Op::Call("__hyp_pow".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 256.0).abs() < f64::EPSILON, "pow(2,8)=256, got {}", v);
    }

    // ── Phase 5: Dict Builtins ──────────────────────────────────────────────

    #[test]
    fn dict_has_key_exists() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(key_chain("x")))
            .push_op(Op::PushNum(10.0))
            .push_op(Op::PushNum(1.0))
            .push_op(Op::Call("__dict_new".into()))
            .push_op(Op::Push(key_chain("x")))
            .push_op(Op::Call("__dict_has_key".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.outputs()[0].is_empty(), "has_key should be truthy");
    }

    #[test]
    fn dict_has_key_missing() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(key_chain("x")))
            .push_op(Op::PushNum(10.0))
            .push_op(Op::PushNum(1.0))
            .push_op(Op::Call("__dict_new".into()))
            .push_op(Op::Push(key_chain("z")))
            .push_op(Op::Call("__dict_has_key".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result.outputs()[0].is_empty(), "has_key for missing key should be falsy");
    }

    #[test]
    fn array_reverse() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(1.0))
            .push_op(Op::PushNum(2.0))
            .push_op(Op::PushNum(3.0))
            .push_op(Op::PushNum(3.0))
            .push_op(Op::Call("__array_new".into()))
            .push_op(Op::Call("__array_reverse".into()))
            // Get first element (should be 3 after reverse)
            .push_op(Op::PushNum(0.0))
            .push_op(Op::Call("__array_get".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 3.0).abs() < f64::EPSILON, "first after reverse should be 3, got {}", v);
    }

    #[test]
    fn array_contains_found() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(10.0))
            .push_op(Op::PushNum(20.0))
            .push_op(Op::PushNum(30.0))
            .push_op(Op::PushNum(3.0))
            .push_op(Op::Call("__array_new".into()))
            .push_op(Op::PushNum(20.0))
            .push_op(Op::Call("__array_contains".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.outputs()[0].is_empty(), "should contain 20");
    }

    #[test]
    fn type_of_builtin() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(42.0))
            .push_op(Op::Call("__type_of".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(!result.outputs()[0].is_empty());
    }

    #[test]
    fn chain_len_builtin() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(42.0))
            .push_op(Op::Call("__chain_len".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        // Number chain has 4 molecules
        assert!(v > 0.0, "chain_len should be > 0, got {}", v);
    }
}
