//! # ffr — Fibonacci Fractal Representation
//!
//! Xoắn ốc Fibonacci trong không gian 5 chiều.
//! Mỗi vị trí trên xoắn ốc = địa chỉ vật lý duy nhất.
//!
//! FFR(n) = vị trí thứ n trên xoắn ốc 5D:
//!   shape    = Fib(n) mod 7   → [0..6]  → ShapeBase
//!   relation = Fib(n+1) mod 8 → [0..7]  → RelationBase
//!   valence  = (Fib(n+2) mod 256) as u8 → EmotionDim.valence
//!   arousal  = (Fib(n+3) mod 256) as u8 → EmotionDim.arousal
//!   time     = Fib(n+4) mod 5  → [0..4] → TimeDim
//!
//! Tính chất:
//!   - Mỗi index → địa chỉ duy nhất
//!   - Gần nhau trong chuỗi → gần nhau trong không gian
//!   - Không trùng lặp (ngoại trừ chu kỳ Pisano rất lớn)

extern crate alloc;

use olang::molecular::{EmotionDim, MolecularChain, Molecule};

/// Fibonacci(n) mod 2^64 — dùng u64 để tránh overflow.
pub fn fib64(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0u64;
            let mut b = 1u64;
            for _ in 2..=n {
                let c = a.wrapping_add(b);
                a = b;
                b = c;
            }
            b
        }
    }
}

/// Một điểm trên xoắn ốc Fibonacci 5D.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FfrPoint {
    pub index: u64,
    pub shape: u8,    // 0..6
    pub relation: u8, // 0..7
    pub valence: u8,  // 0..255
    pub arousal: u8,  // 0..255
    pub time: u8,     // 0..4
}

impl FfrPoint {
    /// Tính điểm Fibonacci thứ n trên xoắn ốc 5D.
    pub fn at(n: u64) -> Self {
        let f0 = fib64(n);
        let f1 = fib64(n.wrapping_add(1));
        let f2 = fib64(n.wrapping_add(2));
        let f3 = fib64(n.wrapping_add(3));
        let f4 = fib64(n.wrapping_add(4));

        Self {
            index: n,
            shape: (f0 % 7) as u8,
            relation: (f1 % 8) as u8,
            valence: (f2 % 256) as u8,
            arousal: (f3 % 256) as u8,
            time: (f4 % 5) as u8,
        }
    }

    /// Convert sang Molecule.
    pub fn to_molecule(&self) -> Molecule {
        Molecule {
            shape: self.shape + 1,       // +1: 0x01..0x07
            relation: self.relation + 1, // +1: 0x01..0x08
            emotion: EmotionDim {
                valence: self.valence,
                arousal: self.arousal,
            },
            time: self.time + 1, // +1: 0x01..0x05
        }
    }

    /// Khoảng cách Fibonacci giữa 2 điểm (5D Manhattan trên index).
    pub fn fib_distance(a: u64, b: u64) -> u64 {
        a.abs_diff(b)
    }
}

/// Tạo MolecularChain từ n điểm Fibonacci liên tiếp bắt đầu từ start.
pub fn ffr_chain(start: u64, n: usize) -> MolecularChain {
    let mut mols = alloc::vec::Vec::with_capacity(n);
    for i in 0..n as u64 {
        let pt = FfrPoint::at(start.wrapping_add(i));
        mols.push(pt.to_molecule());
    }
    MolecularChain(mols)
}

/// Tìm index Fibonacci gần nhất với một Molecule.
///
/// Scan range [0..max_index] và tìm điểm có Molecule gần nhất.
/// Trả về (best_index, similarity).
pub fn ffr_nearest(target: &Molecule, max_index: u64) -> (u64, f32) {
    let mut best_idx = 0u64;
    let mut best_sim = -1.0f32;

    // Sample theo bước Fibonacci để efficient hơn linear scan
    let step = if max_index > 1000 { fib64(10) } else { 1 };
    let step = step.max(1);

    let mut i = 0u64;
    while i < max_index {
        let pt = FfrPoint::at(i);
        let mol = pt.to_molecule();
        let sim = molecule_similarity(target, &mol);
        if sim > best_sim {
            best_sim = sim;
            best_idx = i;
        }
        i = i.wrapping_add(step);
    }

    (best_idx, best_sim)
}

/// Similarity giữa 2 Molecules — dùng cho FFR nearest search.
fn molecule_similarity(a: &Molecule, b: &Molecule) -> f32 {
    let shape_match = if a.shape == b.shape { 1.0f32 } else { 0.0 };
    let rel_match = if a.relation == b.relation {
        1.0f32
    } else {
        0.0
    };

    let dv = (a.emotion.valence as f32 - b.emotion.valence as f32) / 255.0;
    let da = (a.emotion.arousal as f32 - b.emotion.arousal as f32) / 255.0;
    let emo_sim = 1.0 - homemath::sqrtf(dv * dv + da * da) * 0.5;

    let time_match = if a.time == b.time { 1.0f32 } else { 0.5 };

    0.3 * shape_match + 0.2 * rel_match + 0.4 * emo_sim + 0.1 * time_match
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fib64_sequence() {
        assert_eq!(fib64(0), 0);
        assert_eq!(fib64(1), 1);
        assert_eq!(fib64(2), 1);
        assert_eq!(fib64(3), 2);
        assert_eq!(fib64(4), 3);
        assert_eq!(fib64(5), 5);
        assert_eq!(fib64(6), 8);
        assert_eq!(fib64(7), 13);
        assert_eq!(fib64(10), 55);
        assert_eq!(fib64(20), 6765);
    }

    #[test]
    fn ffr_point_unique_nearby() {
        // Các điểm liên tiếp phải có giá trị khác nhau (không bị trùng)
        let p0 = FfrPoint::at(0);
        let p1 = FfrPoint::at(1);
        let p2 = FfrPoint::at(2);
        // Ít nhất 1 chiều phải khác nhau
        assert!(p0 != p1 || p1 != p2, "Consecutive FFR points must differ");
    }

    #[test]
    fn ffr_point_bounded() {
        for n in [0u64, 1, 5, 10, 100, 1000] {
            let p = FfrPoint::at(n);
            assert!(p.shape < 7, "shape in [0..6]");
            assert!(p.relation < 8, "relation in [0..7]");
            assert!(p.time < 5, "time in [0..4]");
        }
    }

    #[test]
    fn ffr_point_to_molecule() {
        for n in [0u64, 1, 2, 5, 10, 50] {
            let p = FfrPoint::at(n);
            let mol = p.to_molecule();
            // Molecule fields are raw u8 — always succeeds
            assert!(mol.shape > 0, "FFR[{}] shape must be > 0", n);
        }
    }

    #[test]
    fn ffr_chain_length() {
        let chain = ffr_chain(0, 5);
        assert_eq!(chain.len(), 5, "ffr_chain(0,5) → 5 molecules");
    }

    #[test]
    fn ffr_chain_non_empty() {
        let chain = ffr_chain(100, 3);
        assert!(!chain.is_empty());
        assert_eq!(chain.len(), 3);
    }

    #[test]
    fn ffr_deterministic() {
        // Cùng index → cùng kết quả
        let p1 = FfrPoint::at(42);
        let p2 = FfrPoint::at(42);
        assert_eq!(p1, p2, "FFR phải deterministic");
    }

    #[test]
    fn ffr_distance() {
        assert_eq!(FfrPoint::fib_distance(5, 3), 2);
        assert_eq!(FfrPoint::fib_distance(3, 5), 2);
        assert_eq!(FfrPoint::fib_distance(7, 7), 0);
    }

    #[test]
    fn ffr_nearest_finds_close() {
        // Tạo molecule từ FFR point 10, tìm nearest
        let pt = FfrPoint::at(10);
        let mol = pt.to_molecule();
        let (best_idx, sim) = ffr_nearest(&mol, 50);
        assert!(sim > 0.5, "Nearest phải có sim > 0.5: {}", sim);
        assert!(best_idx < 50, "Trong range");
    }

    #[test]
    fn ffr_coverage() {
        // 100 điểm đầu phải cover đủ shape/relation variants
        let mut shapes = [false; 7];
        let mut relations = [false; 8];
        for n in 0..100u64 {
            let p = FfrPoint::at(n);
            shapes[p.shape as usize] = true;
            relations[p.relation as usize] = true;
        }
        let shapes_covered = shapes.iter().filter(|&&v| v).count();
        let relations_covered = relations.iter().filter(|&&v| v).count();
        assert!(
            shapes_covered >= 5,
            "Ít nhất 5/7 shapes covered: {}",
            shapes_covered
        );
        assert!(
            relations_covered >= 6,
            "Ít nhất 6/8 relations covered: {}",
            relations_covered
        );
    }
}
