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
                        "__cmp_lt" | "__cmp_gt" | "__cmp_le" | "__cmp_ge" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let nb = b.to_number().unwrap_or(0.0);
                            let truthy = match name.as_str() {
                                "__cmp_lt" => na < nb,
                                "__cmp_gt" => na > nb,
                                "__cmp_le" => na <= nb,
                                "__cmp_ge" => na >= nb,
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

                Op::TryBegin(catch_target) => {
                    // Push catch handler PC onto try stack
                    try_stack.push(*catch_target);
                }

                Op::CatchEnd => {
                    // End of catch block — pop try entry if still present
                    // (already popped if error occurred and caught)
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
}
