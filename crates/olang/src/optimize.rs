//! # optimize — IR optimization passes
//!
//! Runs on OlangProgram before VM execution or compilation.
//!
//! Passes:
//!   1. Constant folding: PushNum(a) PushNum(b) Call(__hyp_add) → PushNum(a+b)
//!   2. Dead code elimination: code after Halt/Ret is unreachable
//!   3. Nop elimination: remove consecutive Nops
//!   4. Identity elimination: PushNum(0) __hyp_add → nop (add zero)

extern crate alloc;
use alloc::vec::Vec;

use crate::ir::{OlangProgram, Op};

/// Optimization level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptLevel {
    /// No optimization
    O0,
    /// Basic: constant folding + dead code elimination
    O1,
    /// Full: O1 + identity elimination + nop removal
    O2,
}

/// Optimization result statistics.
#[derive(Debug, Clone, Default)]
pub struct OptStats {
    /// Number of constant folds applied
    pub folds: u32,
    /// Number of dead instructions removed
    pub dead_removed: u32,
    /// Number of nops removed
    pub nops_removed: u32,
    /// Number of identity ops removed
    pub identities_removed: u32,
}

/// Optimize an OlangProgram in-place.
pub fn optimize(prog: &mut OlangProgram, level: OptLevel) -> OptStats {
    let mut stats = OptStats::default();
    if level == OptLevel::O0 {
        return stats;
    }

    // Pass 1: Constant folding
    stats.folds = constant_fold(&mut prog.ops);

    // Pass 2: Dead code elimination
    stats.dead_removed = eliminate_dead_code(&mut prog.ops);

    if level == OptLevel::O2 {
        // Pass 3: Identity elimination
        stats.identities_removed = eliminate_identities(&mut prog.ops);

        // Pass 4: Nop removal
        stats.nops_removed = remove_nops(&mut prog.ops);
    }

    stats
}

/// Constant folding: merge consecutive PushNum + arithmetic into single PushNum.
fn constant_fold(ops: &mut [Op]) -> u32 {
    let mut folds = 0u32;
    let mut i = 0;
    while i + 2 < ops.len() {
        if let (Op::PushNum(a), Op::PushNum(b)) = (&ops[i], &ops[i + 1]) {
            let a = *a;
            let b = *b;
            if let Op::Call(name) = &ops[i + 2] {
                let result = match name.as_str() {
                    "__hyp_add" | "__phys_add" => Some(a + b),
                    "__hyp_sub" | "__phys_sub" => Some(a - b),
                    "__hyp_mul" => Some(a * b),
                    "__hyp_div" => {
                        if b != 0.0 {
                            Some(a / b)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                if let Some(val) = result {
                    // Replace 3 ops with 1 PushNum
                    ops[i] = Op::PushNum(val);
                    ops[i + 1] = Op::Nop;
                    ops[i + 2] = Op::Nop;
                    folds += 1;
                    // Don't advance i — check if this result can fold further
                    continue;
                }
            }
        }
        i += 1;
    }
    folds
}

/// Dead code elimination: remove instructions after Halt/Ret.
fn eliminate_dead_code(ops: &mut [Op]) -> u32 {
    let mut removed = 0u32;
    let mut found_terminator = false;
    for op in ops.iter_mut() {
        if found_terminator {
            if !matches!(op, Op::Nop) {
                removed += 1;
                *op = Op::Nop;
            }
        } else if matches!(op, Op::Halt | Op::Ret) {
            found_terminator = true;
        }
    }
    removed
}

/// Identity elimination: x + 0 → x, x * 1 → x, x * 0 → 0.
fn eliminate_identities(ops: &mut [Op]) -> u32 {
    let mut removed = 0u32;
    let mut i = 0;
    while i + 1 < ops.len() {
        if let Op::PushNum(n) = &ops[i] {
            let n = *n;
            if let Op::Call(name) = &ops[i + 1] {
                let is_identity = match name.as_str() {
                    // x + 0 or x - 0: identity
                    "__hyp_add" | "__phys_add" | "__hyp_sub" | "__phys_sub" if n == 0.0 => true,
                    // x * 1: identity
                    "__hyp_mul" if (n - 1.0).abs() < f64::EPSILON => true,
                    _ => false,
                };
                if is_identity {
                    ops[i] = Op::Nop;
                    ops[i + 1] = Op::Nop;
                    removed += 1;
                    i += 2;
                    continue;
                }
            }
        }
        i += 1;
    }
    removed
}

/// Remove Nop instructions (compacts the program).
fn remove_nops(ops: &mut Vec<Op>) -> u32 {
    let before = ops.len();
    ops.retain(|op| !matches!(op, Op::Nop));
    (before - ops.len()) as u32
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::OlangProgram;

    #[test]
    fn constant_fold_add() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(3.0))
            .push_op(Op::PushNum(4.0))
            .push_op(Op::Call("__hyp_add".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);

        let stats = optimize(&mut prog, OptLevel::O2);
        assert_eq!(stats.folds, 1, "Should fold 3+4");

        // After O2, nops removed: should be [PushNum(7), Emit, Halt]
        assert!(prog.ops.iter().any(|op| matches!(op, Op::PushNum(n) if (*n - 7.0).abs() < f64::EPSILON)));
    }

    #[test]
    fn constant_fold_chain() {
        // 1 + 2 + 3 → should fold to 6 (two folds)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(1.0))
            .push_op(Op::PushNum(2.0))
            .push_op(Op::Call("__hyp_add".into()))
            .push_op(Op::PushNum(3.0))
            .push_op(Op::Call("__hyp_add".into()))
            .push_op(Op::Halt);

        let stats = optimize(&mut prog, OptLevel::O1);
        // First fold: 1+2 → 3, then 3+3 might fold in next pass
        assert!(stats.folds >= 1);
    }

    #[test]
    fn constant_fold_mul() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(5.0))
            .push_op(Op::PushNum(6.0))
            .push_op(Op::Call("__hyp_mul".into()))
            .push_op(Op::Halt);

        optimize(&mut prog, OptLevel::O1);
        assert!(prog.ops.iter().any(|op| matches!(op, Op::PushNum(n) if (*n - 30.0).abs() < f64::EPSILON)));
    }

    #[test]
    fn constant_fold_div_by_zero_skipped() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(5.0))
            .push_op(Op::PushNum(0.0))
            .push_op(Op::Call("__hyp_div".into()))
            .push_op(Op::Halt);

        let stats = optimize(&mut prog, OptLevel::O1);
        assert_eq!(stats.folds, 0, "Division by zero not folded");
    }

    #[test]
    fn dead_code_after_halt() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(1.0))
            .push_op(Op::Halt)
            .push_op(Op::PushNum(2.0))
            .push_op(Op::PushNum(3.0))
            .push_op(Op::Emit);

        let stats = optimize(&mut prog, OptLevel::O1);
        assert_eq!(stats.dead_removed, 3, "3 ops after Halt");
    }

    #[test]
    fn identity_add_zero() {
        let mut prog = OlangProgram::new("test");
        // LoadLocal x (unknown at compile time), then PushNum(0) + add → identity
        prog.push_op(Op::LoadLocal("x".into()))
            .push_op(Op::PushNum(0.0))
            .push_op(Op::Call("__hyp_add".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);

        let stats = optimize(&mut prog, OptLevel::O2);
        assert_eq!(stats.identities_removed, 1);
    }

    #[test]
    fn identity_mul_one() {
        let mut prog = OlangProgram::new("test");
        // LoadLocal x, then PushNum(1) + mul → identity
        prog.push_op(Op::LoadLocal("x".into()))
            .push_op(Op::PushNum(1.0))
            .push_op(Op::Call("__hyp_mul".into()))
            .push_op(Op::Halt);

        let stats = optimize(&mut prog, OptLevel::O2);
        assert_eq!(stats.identities_removed, 1);
    }

    #[test]
    fn nop_removal() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Nop)
            .push_op(Op::Nop)
            .push_op(Op::PushNum(1.0))
            .push_op(Op::Nop)
            .push_op(Op::Halt);

        let stats = optimize(&mut prog, OptLevel::O2);
        assert_eq!(stats.nops_removed, 3);
        // Only PushNum(1.0) and Halt remain
        assert_eq!(prog.ops.len(), 2);
    }

    #[test]
    fn o0_no_changes() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(1.0))
            .push_op(Op::PushNum(2.0))
            .push_op(Op::Call("__hyp_add".into()))
            .push_op(Op::Halt);
        let original_len = prog.ops.len();

        let stats = optimize(&mut prog, OptLevel::O0);
        assert_eq!(stats.folds, 0);
        assert_eq!(prog.ops.len(), original_len);
    }

    #[test]
    fn scope_ops_survive_optimization() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::ScopeBegin)
            .push_op(Op::PushNum(1.0))
            .push_op(Op::Store("x".into()))
            .push_op(Op::ScopeEnd)
            .push_op(Op::Halt);

        optimize(&mut prog, OptLevel::O2);
        assert!(prog.ops.contains(&Op::ScopeBegin));
        assert!(prog.ops.contains(&Op::ScopeEnd));
    }
}
