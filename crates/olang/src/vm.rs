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
use crate::molecular::MolecularChain;

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
}

// ─────────────────────────────────────────────────────────────────────────────
// VmError
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum VmError {
    StackUnderflow,
    InfiniteLoop,
    InvalidJump(usize),
    MaxStepsExceeded,
}

// ─────────────────────────────────────────────────────────────────────────────
// VmStack
// ─────────────────────────────────────────────────────────────────────────────

const STACK_MAX: usize = 256;
const STEPS_MAX: u32 = 65_536;

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
            return Err(VmError::StackUnderflow); // reuse error type
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

/// OlangVM — stack machine thực thi OlangProgram.
pub struct OlangVM {
    /// Max steps để tránh infinite loop (QT2: ∞-1)
    max_steps: u32,
}

#[allow(missing_docs)]
impl OlangVM {
    pub fn new() -> Self {
        Self {
            max_steps: STEPS_MAX,
        }
    }

    pub fn with_max_steps(n: u32) -> Self {
        Self { max_steps: n }
    }

    /// Execute program → Vec<VmEvent>.
    ///
    /// VM không access registry trực tiếp.
    /// LOAD → emit LookupAlias event → caller inject chain.
    /// Sau đó caller gọi resume_with(chain) để tiếp tục.
    pub fn execute(&self, prog: &OlangProgram) -> VmResult {
        let mut stack = VmStack::new();
        let mut events = Vec::new();
        let mut steps = 0u32;
        let mut pc = 0usize;

        while pc < prog.ops.len() {
            if steps >= self.max_steps {
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

                Op::Load(alias) => {
                    // Emit event — caller sẽ inject chain
                    events.push(VmEvent::LookupAlias(alias.clone()));
                    // Push empty placeholder — real impl dùng callback
                    let _ = stack.push(MolecularChain::empty());
                }

                Op::Lca => {
                    let b = match stack.pop() {
                        Ok(c) => c,
                        Err(e) => {
                            events.push(VmEvent::Error(e));
                            break;
                        }
                    };
                    let a = match stack.pop() {
                        Ok(c) => c,
                        Err(e) => {
                            events.push(VmEvent::Error(e));
                            break;
                        }
                    };
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
                    let b = match stack.pop() {
                        Ok(c) => c,
                        Err(e) => {
                            events.push(VmEvent::Error(e));
                            break;
                        }
                    };
                    let a = match stack.pop() {
                        Ok(c) => c,
                        Err(e) => {
                            events.push(VmEvent::Error(e));
                            break;
                        }
                    };
                    events.push(VmEvent::CreateEdge {
                        from: a.chain_hash(),
                        to: b.chain_hash(),
                        rel: *rel,
                    });
                    // Giữ lại b trên stack (kết quả của relation)
                    let _ = stack.push(b);
                }

                Op::Query(rel) => {
                    let a = match stack.pop() {
                        Ok(c) => c,
                        Err(e) => {
                            events.push(VmEvent::Error(e));
                            break;
                        }
                    };
                    events.push(VmEvent::QueryRelation {
                        hash: a.chain_hash(),
                        rel: *rel,
                    });
                    // Push empty — caller sẽ inject results
                    let _ = stack.push(MolecularChain::empty());
                }

                Op::Emit => {
                    let c = match stack.pop() {
                        Ok(c) => c,
                        Err(e) => {
                            events.push(VmEvent::Error(e));
                            break;
                        }
                    };
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
                    if let Err(e) = stack.pop() {
                        events.push(VmEvent::Error(e));
                        break;
                    }
                }

                Op::Swap => {
                    let b = match stack.pop() {
                        Ok(c) => c,
                        Err(e) => {
                            events.push(VmEvent::Error(e));
                            break;
                        }
                    };
                    let a = match stack.pop() {
                        Ok(c) => c,
                        Err(e) => {
                            events.push(VmEvent::Error(e));
                            break;
                        }
                    };
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
                    // Simple: push loop marker, unroll is caller's job
                    // Trong VM đơn giản này: Loop(n) = noop (unroll ở compile time)
                    let _ = n;
                }

                Op::Call(name) => {
                    // Simple call — emit event, không có call stack thật
                    events.push(VmEvent::LookupAlias(name.clone()));
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

                Op::Halt => {
                    break;
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

    fn skip() -> bool {
        ucd::table_len() == 0
    }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
        if skip() {
            return;
        }
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
}
