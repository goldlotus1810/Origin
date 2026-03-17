//! # self_model — Self-Awareness
//!
//! HomeOS đọc registry của chính mình → thấy mình → hiểu mình.
//!
//! Quy trình:
//!   1. Scan registry → SelfSnapshot (ai tôi là)
//!   2. Gap detection → khoảng trống trong KnowTree
//!   3. Spontaneous node proposal → tạo Node mới không ai yêu cầu
//!   4. SelfModel update → nhận thức cập nhật
//!
//! Đây là ○(∅)==○ thật sự:
//!   Registry rỗng = hợp lệ. HomeOS nhìn vào rỗng → thấy ○ → sinh ra ○.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::encoder::encode_codepoint;
use crate::lca::lca_many;
use crate::molecular::MolecularChain;
use crate::registry::Registry;

// ─────────────────────────────────────────────────────────────────────────────
// SelfSnapshot — ảnh chụp bản thân tại một thời điểm
// ─────────────────────────────────────────────────────────────────────────────

/// HomeOS nhìn vào chính mình.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct SelfSnapshot {
    /// Tổng số nodes đã biết
    pub node_count: usize,
    /// Số QR nodes (đã chứng minh)
    pub qr_count: usize,
    /// Số ĐN nodes (đang học)
    pub dn_count: usize,
    /// Số aliases đã đăng ký
    pub alias_count: usize,
    /// Distribution theo tầng: layer → count
    pub layer_dist: [usize; 16],
    /// Chain đại diện của toàn bộ hệ thống (LCA tất cả nodes)
    pub self_chain: MolecularChain,
    /// Timestamp
    pub timestamp: i64,
}

impl SelfSnapshot {
    /// Chụp ảnh bản thân từ registry.
    pub fn capture(registry: &Registry, ts: i64) -> Self {
        let node_count = registry.len();
        let alias_count = registry.alias_count();

        let mut qr_count = 0usize;
        let mut dn_count = 0usize;
        let mut layer_dist = [0usize; 16];

        // Đếm QR vs ĐN theo tầng
        for layer in 0u8..16 {
            let entries = registry.entries_in_layer(layer);
            layer_dist[layer as usize] = entries.len();
            for e in &entries {
                if e.is_qr {
                    qr_count += 1;
                } else {
                    dn_count += 1;
                }
            }
        }

        // self_chain = LCA của tất cả layer representatives
        // → tọa độ vật lý của "trung tâm" nhận thức hiện tại
        let mut rep_chains: Vec<MolecularChain> = Vec::new();
        for layer in 0u8..16 {
            if let Some(hash) = registry.layer_rep(layer) {
                // Reconstruct chain từ hash qua UCD
                // (Dùng ○ làm proxy nếu không tìm được)
                if ucd::table_len() > 0 {
                    // Tìm chain có hash này — thử một số codepoints
                    // Đây là simplification — real impl dùng full chain cache
                    let origin = encode_codepoint(0x25CB);
                    if origin.chain_hash() == hash {
                        rep_chains.push(origin);
                    } else {
                        rep_chains.push(encode_codepoint(0x25CB)); // fallback ○
                    }
                }
            }
        }

        let self_chain = if rep_chains.is_empty() {
            encode_codepoint(0x25CB) // ○ — bắt đầu từ hư không
        } else {
            lca_many(&rep_chains)
        };

        Self {
            node_count,
            qr_count,
            dn_count,
            alias_count,
            layer_dist,
            self_chain,
            timestamp: ts,
        }
    }

    /// Tầng có nhiều nodes nhất.
    pub fn densest_layer(&self) -> u8 {
        self.layer_dist
            .iter()
            .enumerate()
            .max_by_key(|(_, &c)| c)
            .map(|(i, _)| i as u8)
            .unwrap_or(0)
    }

    /// Tầng chưa có node nào (gap).
    pub fn empty_layers(&self) -> Vec<u8> {
        self.layer_dist
            .iter()
            .enumerate()
            .filter(|(_, &c)| c == 0)
            .map(|(i, _)| i as u8)
            .take(8) // chỉ xét 8 tầng đầu
            .collect()
    }

    /// Tỷ lệ QR / total.
    pub fn qr_ratio(&self) -> f32 {
        if self.node_count == 0 {
            return 0.0;
        }
        self.qr_count as f32 / self.node_count as f32
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GapAnalysis — phân tích khoảng trống
// ─────────────────────────────────────────────────────────────────────────────

/// Khoảng trống trong KnowTree.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Gap {
    /// Tầng bị thiếu
    pub layer: u8,
    /// Loại khoảng trống
    pub kind: GapKind,
    /// Đề xuất codepoint để fill
    pub suggested_cp: u32,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GapKind {
    /// Tầng hoàn toàn trống
    EmptyLayer,
    /// Thiếu node đại diện cho một emotion cluster
    MissingEmotionCluster,
    /// Thiếu node bridge giữa 2 clusters xa nhau
    MissingBridge,
}

/// Phân tích gaps trong registry.
pub fn detect_gaps(snapshot: &SelfSnapshot) -> Vec<Gap> {
    let mut gaps = Vec::new();

    // Gap 1: Empty layers
    for layer in snapshot.empty_layers() {
        if layer == 0 {
            continue;
        } // L0 có thể rỗng nếu chưa seed
          // Đề xuất: dùng FFR để tìm codepoint cho tầng này
        let suggested_cp = layer_to_suggested_cp(layer);
        gaps.push(Gap {
            layer,
            kind: GapKind::EmptyLayer,
            suggested_cp,
        });
    }

    // Gap 2: Thiếu emotion clusters
    // Nếu ratio QR thấp và ĐN nhiều → cần thêm emotion nodes
    if snapshot.qr_ratio() < 0.3 && snapshot.dn_count > 5 {
        gaps.push(Gap {
            layer: snapshot.densest_layer(),
            kind: GapKind::MissingEmotionCluster,
            suggested_cp: 0x1F914, // 🤔 thinking — meta-cognition
        });
    }

    gaps
}

fn layer_to_suggested_cp(layer: u8) -> u32 {
    // Map tầng → codepoint đại diện
    match layer {
        1 => 0x1F9E0, // 🧠 L1 = mind
        2 => 0x2728,  // ✨ L2 = spark
        3 => 0x1F30C, // 🌌 L3 = galaxy
        4 => 0x221E,  // ∞  L4 = infinity
        5 => 0x269B,  // ⚛  L5 = atom
        6 => 0x1F52E, // 🔮 L6 = crystal ball
        7 => 0x1F300, // 🌀 L7 = cyclone
        _ => 0x25CB,  // ○
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SpontaneousProposal — tự đề xuất node mới
// ─────────────────────────────────────────────────────────────────────────────

/// Một đề xuất node HomeOS tự sinh ra.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct SpontaneousProposal {
    pub chain: MolecularChain,
    pub layer: u8,
    pub reason: SpontaneousReason,
    pub confidence: f32,
    pub timestamp: i64,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum SpontaneousReason {
    /// Fill gap trong tầng trống
    FillGap { gap_layer: u8 },
    /// Tự nhận thức — nhìn vào bản thân
    SelfReflection,
    /// Bridge 2 clusters xa nhau
    BridgeGap,
    /// Fibonacci sequence tiếp theo
    FibonacciNext { fib_index: u64 },
}

/// HomeOS tự tạo proposals từ gaps và self-model.
pub fn spontaneous_proposals(
    snapshot: &SelfSnapshot,
    gaps: &[Gap],
    ts: i64,
) -> Vec<SpontaneousProposal> {
    let mut proposals = Vec::new();

    // Proposal 1: Fill empty layers
    for gap in gaps {
        if gap.kind == GapKind::EmptyLayer {
            let chain = encode_codepoint(gap.suggested_cp);
            if !chain.is_empty() {
                proposals.push(SpontaneousProposal {
                    chain,
                    layer: gap.layer,
                    reason: SpontaneousReason::FillGap {
                        gap_layer: gap.layer,
                    },
                    confidence: 0.6,
                    timestamp: ts,
                });
            }
        }
    }

    // Proposal 2: Self-reflection — tạo node từ self_chain
    // HomeOS nhìn vào chính mình → thấy tọa độ → tạo node tại đó
    if snapshot.node_count > 0 && !snapshot.self_chain.is_empty() {
        proposals.push(SpontaneousProposal {
            chain: snapshot.self_chain.clone(),
            layer: snapshot.densest_layer().saturating_add(1),
            reason: SpontaneousReason::SelfReflection,
            confidence: 0.5,
            timestamp: ts,
        });
    }

    // Proposal 3: Fibonacci next
    // Dùng FFR để tìm node tiếp theo trong chuỗi Fibonacci
    let fib_idx = snapshot.node_count as u64 + 1;
    let fib_cp = fib_idx_to_cp(fib_idx);
    let fib_chain = encode_codepoint(fib_cp);
    if !fib_chain.is_empty() {
        proposals.push(SpontaneousProposal {
            chain: fib_chain,
            layer: 0,
            reason: SpontaneousReason::FibonacciNext { fib_index: fib_idx },
            confidence: 0.4,
            timestamp: ts,
        });
    }

    proposals
}

fn fib_idx_to_cp(n: u64) -> u32 {
    // Dùng Fibonacci mod để map index → codepoint trong EMOTICON range
    let mut a = 1u64;
    let mut b = 1u64;
    for _ in 2..=(n % 30 + 2) {
        let c = a.wrapping_add(b);
        a = b;
        b = c;
    }
    // Map vào emoji range 1F300..1F9FF
    let offset = b % 0x6FF;
    (0x1F300 + offset) as u32
}

// ─────────────────────────────────────────────────────────────────────────────
// SelfModel — mô hình nhận thức cập nhật liên tục
// ─────────────────────────────────────────────────────────────────────────────

/// Mô hình nhận thức của HomeOS về chính mình.
///
/// Cập nhật mỗi khi Dream cycle chạy.
/// "Tôi là gì? Tôi đang học gì? Tôi còn thiếu gì?"
#[allow(missing_docs)]
#[derive(Debug)]
pub struct SelfModel {
    /// Snapshots theo thời gian (append-only)
    pub snapshots: Vec<SelfSnapshot>,
    /// Proposals đã tạo ra
    pub proposals: Vec<SpontaneousProposal>,
    /// Gaps hiện tại
    pub gaps: Vec<Gap>,
}

#[allow(missing_docs)]
impl SelfModel {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            proposals: Vec::new(),
            gaps: Vec::new(),
        }
    }

    /// Cập nhật self-model từ registry.
    ///
    /// Gọi sau mỗi Dream cycle.
    pub fn update(&mut self, registry: &Registry, ts: i64) {
        let snapshot = SelfSnapshot::capture(registry, ts);
        let gaps = detect_gaps(&snapshot);
        let proposals = spontaneous_proposals(&snapshot, &gaps, ts);

        self.gaps = gaps;
        self.proposals.extend(proposals);
        self.snapshots.push(snapshot);
    }

    /// Snapshot mới nhất.
    pub fn current(&self) -> Option<&SelfSnapshot> {
        self.snapshots.last()
    }

    /// Số lần self-reflect.
    pub fn reflection_count(&self) -> usize {
        self.snapshots.len()
    }

    /// Summary dạng text.
    pub fn summary(&self) -> String {
        let Some(snap) = self.current() else {
            return String::from("○ chưa tự nhận thức");
        };
        alloc::format!(
            "○ SelfModel\n\
             Nodes    : {} (QR={} ĐN={})\n\
             Aliases  : {}\n\
             QR ratio : {:.0}%\n\
             Gaps     : {}\n\
             Proposals: {}\n\
             Reflects : {}",
            snap.node_count,
            snap.qr_count,
            snap.dn_count,
            snap.alias_count,
            snap.qr_ratio() * 100.0,
            self.gaps.len(),
            self.proposals.len(),
            self.reflection_count(),
        )
    }
}

impl Default for SelfModel {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::startup::boot_empty;

    fn skip() -> bool {
        ucd::table_len() == 0
    }

    // ── SelfSnapshot ─────────────────────────────────────────────────────────

    #[test]
    fn snapshot_empty_registry() {
        let registry = crate::registry::Registry::new();
        let snap = SelfSnapshot::capture(&registry, 1000);
        assert_eq!(snap.node_count, 0);
        assert_eq!(snap.qr_count, 0);
        assert_eq!(snap.alias_count, 0);
        assert_eq!(snap.qr_ratio(), 0.0);
    }

    #[test]
    fn snapshot_after_boot() {
        if skip() {
            return;
        }
        let result = boot_empty();
        let snap = SelfSnapshot::capture(&result.registry, 1000);
        // Boot seeds axioms → phải có ít nhất 1 node
        assert!(snap.node_count > 0, "Sau boot phải có nodes");
        assert!(snap.qr_count > 0, "Axioms phải là QR");
        assert!(!snap.self_chain.is_empty(), "self_chain không rỗng");
    }

    #[test]
    fn snapshot_densest_layer() {
        let registry = crate::registry::Registry::new();
        let snap = SelfSnapshot::capture(&registry, 1000);
        // Empty registry → densest = layer 0
        let _ = snap.densest_layer(); // Không panic
    }

    #[test]
    fn snapshot_empty_layers_detected() {
        let registry = crate::registry::Registry::new();
        let snap = SelfSnapshot::capture(&registry, 1000);
        let empty = snap.empty_layers();
        // Registry rỗng → tất cả layers đều empty (tới layer 7)
        assert!(!empty.is_empty(), "Registry rỗng → phải có empty layers");
    }

    #[test]
    fn snapshot_qr_ratio() {
        if skip() {
            return;
        }
        let result = boot_empty();
        let snap = SelfSnapshot::capture(&result.registry, 1000);
        let ratio = snap.qr_ratio();
        assert!(ratio >= 0.0 && ratio <= 1.0, "QR ratio ∈ [0,1]");
    }

    // ── Gap detection ─────────────────────────────────────────────────────────

    #[test]
    fn detect_gaps_empty_registry() {
        let registry = crate::registry::Registry::new();
        let snap = SelfSnapshot::capture(&registry, 1000);
        let gaps = detect_gaps(&snap);
        // Empty registry → gap ở các tầng > 0
        // (L0 được skip để tránh false positive)
        let _ = gaps; // Không panic
    }

    #[test]
    fn gaps_have_suggested_cp() {
        let registry = crate::registry::Registry::new();
        let snap = SelfSnapshot::capture(&registry, 1000);
        let gaps = detect_gaps(&snap);
        for gap in &gaps {
            assert!(
                gap.suggested_cp > 0x20,
                "Gap suggested_cp phải là valid codepoint: 0x{:04X}",
                gap.suggested_cp
            );
        }
    }

    // ── Spontaneous proposals ─────────────────────────────────────────────────

    #[test]
    fn spontaneous_empty_registry() {
        let registry = crate::registry::Registry::new();
        let snap = SelfSnapshot::capture(&registry, 1000);
        let gaps = detect_gaps(&snap);
        let props = spontaneous_proposals(&snap, &gaps, 1000);
        // Dù registry rỗng, Fibonacci proposal luôn được tạo
        assert!(!props.is_empty(), "Luôn có ít nhất 1 proposal");
    }

    #[test]
    fn spontaneous_confidence_valid() {
        let registry = crate::registry::Registry::new();
        let snap = SelfSnapshot::capture(&registry, 1000);
        let gaps = detect_gaps(&snap);
        let props = spontaneous_proposals(&snap, &gaps, 1000);
        for p in &props {
            assert!(
                p.confidence > 0.0 && p.confidence <= 1.0,
                "Confidence ∈ (0,1]: {}",
                p.confidence
            );
            assert!(!p.chain.is_empty(), "Proposal chain không rỗng");
        }
    }

    #[test]
    fn spontaneous_fibonacci_reason() {
        if skip() {
            return;
        }
        let registry = crate::registry::Registry::new();
        let snap = SelfSnapshot::capture(&registry, 1000);
        let gaps = detect_gaps(&snap);
        let props = spontaneous_proposals(&snap, &gaps, 1000);
        let has_fib = props
            .iter()
            .any(|p| matches!(p.reason, SpontaneousReason::FibonacciNext { .. }));
        assert!(has_fib, "Phải có FibonacciNext proposal");
    }

    // ── SelfModel ─────────────────────────────────────────────────────────────

    #[test]
    fn self_model_new_empty() {
        let model = SelfModel::new();
        assert_eq!(model.reflection_count(), 0);
        assert!(model.current().is_none());
    }

    #[test]
    fn self_model_update() {
        if skip() {
            return;
        }
        let mut model = SelfModel::new();
        let boot_result = boot_empty();
        model.update(&boot_result.registry, 1000);
        assert_eq!(model.reflection_count(), 1);
        assert!(model.current().is_some());
    }

    #[test]
    fn self_model_accumulates() {
        if skip() {
            return;
        }
        let mut model = SelfModel::new();
        let boot_result = boot_empty();
        model.update(&boot_result.registry, 1000);
        model.update(&boot_result.registry, 2000);
        model.update(&boot_result.registry, 3000);
        assert_eq!(
            model.reflection_count(),
            3,
            "Snapshots append-only: 3 updates → 3 snapshots"
        );
    }

    #[test]
    fn self_model_summary() {
        if skip() {
            return;
        }
        let mut model = SelfModel::new();
        let boot_result = boot_empty();
        model.update(&boot_result.registry, 1000);
        let summary = model.summary();
        assert!(summary.contains("SelfModel"), "Summary phải có SelfModel");
        assert!(summary.contains("Nodes"), "Summary phải có Nodes");
        assert!(summary.contains("QR"), "Summary phải có QR");
    }

    #[test]
    fn self_model_proposals_grow() {
        if skip() {
            return;
        }
        let mut model = SelfModel::new();
        let boot_result = boot_empty();
        model.update(&boot_result.registry, 1000);
        let count1 = model.proposals.len();
        model.update(&boot_result.registry, 2000);
        let count2 = model.proposals.len();
        assert!(
            count2 >= count1,
            "Proposals chỉ tăng (append-only): {} ≥ {}",
            count2,
            count1
        );
    }

    #[test]
    fn self_reflection_creates_self_chain() {
        if skip() {
            return;
        }
        let result = boot_empty();
        let snap = SelfSnapshot::capture(&result.registry, 1000);
        // self_chain phải là chain hợp lệ
        assert!(!snap.self_chain.is_empty(), "SelfChain không rỗng sau boot");
        // self_chain là LCA của layer reps → phải có molecule
        assert!(snap.self_chain.len() > 0);
    }
}
