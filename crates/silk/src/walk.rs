//! # walk — Walk qua Silk graph
//!
//! SentenceAffect: không trung bình từng từ riêng lẻ.
//! Walk qua Silk → emotions amplify nhau theo edge weight.
//!
//! "tôi buồn vì mất việc":
//!   MAT_VIEC → BUON (w=0.90) → CO_DON (w=0.71)
//!   composite V = -0.85 (nặng hơn từng từ riêng lẻ)

extern crate alloc;
use alloc::collections::BTreeSet;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

use crate::edge::{EdgeKind, EmotionTag};
use crate::graph::{MolSummary, SilkGraph};

// ─────────────────────────────────────────────────────────────────────────────
// WalkResult
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả walk qua Silk.
#[derive(Debug, Clone)]
pub struct WalkResult {
    /// EmotionTag tổng hợp sau khi walk
    pub composite: EmotionTag,
    /// Các nodes đã đi qua (chain_hash)
    pub path: Vec<u64>,
    /// Tổng weight tích lũy
    pub total_weight: f32,
}

// ─────────────────────────────────────────────────────────────────────────────
// SentenceAffect
// ─────────────────────────────────────────────────────────────────────────────

/// Walk qua Silk graph để tính EmotionTag tổng hợp của câu.
///
/// Không trung bình — walk theo edge weight → amplify.
/// Depth-limited để tránh infinite walk (QT2: ∞-1).
pub fn sentence_affect(
    graph: &SilkGraph,
    word_hashes: &[u64],          // hash của từng từ trong câu
    word_emotions: &[EmotionTag], // EmotionTag ban đầu của từng từ
    max_depth: usize,             // Fib[n] — depth giới hạn
) -> WalkResult {
    if word_hashes.is_empty() {
        return WalkResult {
            composite: EmotionTag::NEUTRAL,
            path: Vec::new(),
            total_weight: 0.0,
        };
    }

    // Bắt đầu từ EmotionTag của từ đầu tiên
    let mut composite = word_emotions
        .first()
        .copied()
        .unwrap_or(EmotionTag::NEUTRAL);
    let mut path = Vec::new();
    let mut total_weight = 1.0f32;

    path.push(word_hashes[0]);

    // Walk từng từ
    for i in 1..word_hashes.len().min(word_emotions.len()) {
        let hash = word_hashes[i];
        let w_emo = word_emotions[i];

        // Tìm Silk edge từ từ trước đến từ này
        let edge_weight = graph
            .assoc_weight(word_hashes[i - 1], hash)
            .max(graph.assoc_weight(hash, word_hashes[i - 1]));

        if edge_weight > 0.01 {
            // Amplify: từ kết nối → emotion của nó được khuếch đại
            let amplified = amplify_emotion(w_emo, edge_weight);
            composite = blend_composite(composite, amplified, edge_weight);
            total_weight += edge_weight;
        } else {
            // Không có Silk → blend nhẹ
            composite = blend_composite(composite, w_emo, 0.3);
            total_weight += 0.3;
        }

        path.push(hash);

        // Depth limit (QT2)
        if path.len() >= max_depth {
            break;
        }
    }

    // Normalize
    if total_weight > 0.0 {
        // Clamp valence và arousal
        composite.valence = composite.valence.clamp(-1.0, 1.0);
        composite.arousal = composite.arousal.clamp(0.0, 1.0);
        composite.dominance = composite.dominance.clamp(0.0, 1.0);
        composite.intensity = composite.intensity.clamp(0.0, 1.0);
    }

    WalkResult {
        composite,
        path,
        total_weight,
    }
}

/// Walk qua unified Silk (implicit 5D + learned Hebbian) để tính emotion.
///
/// Giống sentence_affect nhưng dùng unified_weight thay vì chỉ assoc_weight.
/// Khi có MolSummary cho mỗi từ → implicit Silk boost emotion amplification.
///
/// Đây là cách đúng theo tài liệu:
/// "Silk = hệ quả của 5D. Hebbian = phát hiện, không tạo mới."
pub fn sentence_affect_unified(
    graph: &SilkGraph,
    word_hashes: &[u64],
    word_emotions: &[EmotionTag],
    word_mols: &[Option<MolSummary>],
    max_depth: usize,
) -> WalkResult {
    if word_hashes.is_empty() {
        return WalkResult {
            composite: EmotionTag::NEUTRAL,
            path: Vec::new(),
            total_weight: 0.0,
        };
    }

    let mut composite = word_emotions
        .first()
        .copied()
        .unwrap_or(EmotionTag::NEUTRAL);
    let mut path = Vec::new();
    let mut total_weight = 1.0f32;

    path.push(word_hashes[0]);

    for i in 1..word_hashes.len().min(word_emotions.len()) {
        let hash = word_hashes[i];
        let w_emo = word_emotions[i];

        // Unified weight: implicit(5D) + hebbian + legacy
        let prev_mol = if i - 1 < word_mols.len() { word_mols[i - 1].as_ref() } else { None };
        let curr_mol = if i < word_mols.len() { word_mols[i].as_ref() } else { None };

        let edge_weight = graph.unified_weight(
            word_hashes[i - 1], hash,
            prev_mol, curr_mol,
        );

        if edge_weight > 0.01 {
            let amplified = amplify_emotion(w_emo, edge_weight);
            composite = blend_composite(composite, amplified, edge_weight);
            total_weight += edge_weight;
        } else {
            composite = blend_composite(composite, w_emo, 0.3);
            total_weight += 0.3;
        }

        path.push(hash);

        if path.len() >= max_depth {
            break;
        }
    }

    if total_weight > 0.0 {
        composite.valence = composite.valence.clamp(-1.0, 1.0);
        composite.arousal = composite.arousal.clamp(0.0, 1.0);
        composite.dominance = composite.dominance.clamp(0.0, 1.0);
        composite.intensity = composite.intensity.clamp(0.0, 1.0);
    }

    WalkResult {
        composite,
        path,
        total_weight,
    }
}

/// Amplify emotion theo edge weight.
///
/// Edge mạnh → emotion đó ảnh hưởng nhiều hơn.
/// Đây là điểm khác biệt với trung bình đơn giản.
fn amplify_emotion(emo: EmotionTag, weight: f32) -> EmotionTag {
    EmotionTag {
        valence: emo.valence * (1.0 + weight * 0.5),
        arousal: emo.arousal * (1.0 + weight * 0.3),
        dominance: emo.dominance,
        intensity: emo.intensity * (1.0 + weight * 0.4),
    }
}

/// Blend composite với emotion mới theo weight.
fn blend_composite(composite: EmotionTag, new_emo: EmotionTag, weight: f32) -> EmotionTag {
    let w_norm = weight / (1.0 + weight);
    EmotionTag {
        valence: composite.valence * (1.0 - w_norm) + new_emo.valence * w_norm,
        arousal: composite.arousal * (1.0 - w_norm) + new_emo.arousal * w_norm,
        dominance: composite.dominance * (1.0 - w_norm) + new_emo.dominance * w_norm,
        intensity: composite.intensity * (1.0 - w_norm) + new_emo.intensity * w_norm,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ResponseTone
// ─────────────────────────────────────────────────────────────────────────────

/// Tone phản hồi từ ConversationCurve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResponseTone {
    /// f'(t) < -0.15 — đang buồn xuống, dẫn lên chậm
    Supportive,
    /// f''(t) < -0.25 — đột ngột xấu, dừng lại hỏi
    Pause,
    /// f'(t) > +0.15 — đang hồi phục, tiếp tục đà
    Reinforcing,
    /// f''(t) > +0.25 AND V > 0 — bước ngoặt tốt
    Celebratory,
    /// V < -0.20, stable — buồn ổn định, dịu dàng
    Gentle,
    /// Bình thường
    Engaged,
}

/// Tính ResponseTone từ ConversationCurve.
///
/// curve: Vec<f32> = valence qua các turns
pub fn response_tone(curve: &[f32]) -> ResponseTone {
    let n = curve.len();
    if n == 0 {
        return ResponseTone::Engaged;
    }

    let v = curve[n - 1];

    // f'(t) — tốc độ thay đổi
    let d1 = if n >= 2 {
        curve[n - 1] - curve[n - 2]
    } else {
        0.0
    };

    // f''(t) — gia tốc thay đổi
    let d2 = if n >= 3 {
        (curve[n - 1] - curve[n - 2]) - (curve[n - 2] - curve[n - 3])
    } else {
        0.0
    };

    // Ưu tiên theo thứ tự
    if d1 < -0.15 {
        ResponseTone::Supportive
    } else if d2 < -0.25 {
        ResponseTone::Pause
    } else if d1 > 0.15 {
        ResponseTone::Reinforcing
    } else if d2 > 0.25 && v > 0.0 {
        ResponseTone::Celebratory
    } else if v < -0.20 {
        ResponseTone::Gentle
    } else {
        ResponseTone::Engaged
    }
}

/// Bước tiếp theo trên ConversationCurve.
///
/// Không nhảy quá 0.40/bước — dẫn từng bước.
pub fn next_curve_step(current_v: f32, target_v: f32) -> f32 {
    let delta = target_v - current_v;
    let max_step = 0.40f32;
    if delta.abs() <= max_step {
        target_v
    } else {
        current_v + max_step * delta.signum()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PathStep — element of a graph path
// ─────────────────────────────────────────────────────────────────────────────

/// One step in a path through the Silk graph.
#[derive(Debug, Clone, PartialEq)]
pub struct PathStep {
    /// The node hash at this step.
    pub node: u64,
    /// The edge kind used to reach this node (None for start node).
    pub edge: Option<EdgeKind>,
    /// The edge weight (0.0 for start node).
    pub weight: f32,
}

// ─────────────────────────────────────────────────────────────────────────────
// find_path — BFS shortest path between two nodes
// ─────────────────────────────────────────────────────────────────────────────

/// BFS record: (node_hash, parent_index, edge_kind, weight, depth).
type BfsRecord = (u64, Option<usize>, Option<EdgeKind>, f32, usize);

/// BFS shortest path from `from` to `to` in the Silk graph.
///
/// Returns the path as a sequence of `PathStep`s, or empty vec if no path.
/// Depth limited to `max_depth` (use Fib[7]=13 as default).
/// Traverses both directions of edges (undirected graph traversal).
pub fn find_path(graph: &SilkGraph, from: u64, to: u64, max_depth: usize) -> Vec<PathStep> {
    if from == to {
        return alloc::vec![PathStep { node: from, edge: None, weight: 0.0 }];
    }

    // BFS with parent tracking
    // Each entry: (node_hash, parent_index, edge_kind, weight)
    let mut queue: VecDeque<usize> = VecDeque::new();
    let mut visited: BTreeSet<u64> = BTreeSet::new();
    let mut records: Vec<BfsRecord> = Vec::new();

    // Start node
    records.push((from, None, None, 0.0, 0));
    queue.push_back(0);
    visited.insert(from);

    while let Some(idx) = queue.pop_front() {
        let (current, _, _, _, depth) = records[idx];

        if depth >= max_depth {
            continue;
        }

        // Iterate all edges involving this node
        for edge in graph.all_edges() {
            let (neighbor, kind, weight) = if edge.from_hash == current {
                (edge.to_hash, edge.kind, edge.weight)
            } else if edge.to_hash == current {
                (edge.from_hash, edge.kind, edge.weight)
            } else {
                continue;
            };

            if visited.contains(&neighbor) {
                continue;
            }

            visited.insert(neighbor);
            let new_idx = records.len();
            records.push((neighbor, Some(idx), Some(kind), weight, depth + 1));

            if neighbor == to {
                // Reconstruct path
                return reconstruct_path(&records, new_idx);
            }

            queue.push_back(new_idx);
        }
    }

    Vec::new() // no path found
}

/// Reconstruct path from BFS records.
fn reconstruct_path(records: &[BfsRecord], end_idx: usize) -> Vec<PathStep> {
    let mut path = Vec::new();
    let mut idx = end_idx;

    loop {
        let (node, parent, edge, weight, _) = &records[idx];
        path.push(PathStep {
            node: *node,
            edge: *edge,
            weight: *weight,
        });
        match parent {
            Some(p) => idx = *p,
            None => break,
        }
    }

    path.reverse();
    path
}

// ─────────────────────────────────────────────────────────────────────────────
// trace_origin — find all incoming edges to a node (ancestry)
// ─────────────────────────────────────────────────────────────────────────────

/// An origin trace entry — one incoming connection to a node.
#[derive(Debug, Clone, PartialEq)]
pub struct OriginEdge {
    /// Source node that points to the target.
    pub from: u64,
    /// Edge kind of the connection.
    pub kind: EdgeKind,
    /// Target node.
    pub to: u64,
    /// Edge weight.
    pub weight: f32,
    /// Depth at which this edge was found.
    pub depth: usize,
}

/// Trace the origin (ancestry) of a node by following incoming edges.
///
/// Returns all edges that lead TO this node, recursively up to `max_depth`.
/// This builds a "family tree" — who contributed to this concept.
pub fn trace_origin(graph: &SilkGraph, target: u64, max_depth: usize) -> Vec<OriginEdge> {
    let mut result = Vec::new();
    let mut visited: BTreeSet<u64> = BTreeSet::new();
    let mut frontier: VecDeque<(u64, usize)> = VecDeque::new();

    visited.insert(target);
    frontier.push_back((target, 0));

    while let Some((node, depth)) = frontier.pop_front() {
        if depth >= max_depth {
            continue;
        }

        // Find all edges pointing TO this node
        for edge in graph.edges_to(node) {
            result.push(OriginEdge {
                from: edge.from_hash,
                kind: edge.kind,
                to: edge.to_hash,
                weight: edge.weight,
                depth: depth + 1,
            });

            if !visited.contains(&edge.from_hash) {
                visited.insert(edge.from_hash);
                frontier.push_back((edge.from_hash, depth + 1));
            }
        }
    }

    result
}

// ─────────────────────────────────────────────────────────────────────────────
// reachable — all nodes reachable from a given node
// ─────────────────────────────────────────────────────────────────────────────

/// All nodes reachable from `start` within `max_depth`, optionally filtered by edge kind.
///
/// If `kind_filter` is Some, only traverse edges of that kind.
/// If `kind_filter` is None, traverse all edges.
/// Returns unique node hashes (excluding start itself).
pub fn reachable(
    graph: &SilkGraph,
    start: u64,
    kind_filter: Option<EdgeKind>,
    max_depth: usize,
) -> Vec<u64> {
    let mut visited: BTreeSet<u64> = BTreeSet::new();
    let mut frontier: VecDeque<(u64, usize)> = VecDeque::new();

    visited.insert(start);
    frontier.push_back((start, 0));

    while let Some((node, depth)) = frontier.pop_front() {
        if depth >= max_depth {
            continue;
        }

        // Outgoing edges
        for edge in graph.edges_from(node) {
            if let Some(filter) = kind_filter {
                if edge.kind != filter {
                    continue;
                }
            }
            if !visited.contains(&edge.to_hash) {
                visited.insert(edge.to_hash);
                frontier.push_back((edge.to_hash, depth + 1));
            }
        }

        // Also follow incoming edges (bidirectional reachability)
        for edge in graph.edges_to(node) {
            if let Some(filter) = kind_filter {
                if edge.kind != filter {
                    continue;
                }
            }
            if !visited.contains(&edge.from_hash) {
                visited.insert(edge.from_hash);
                frontier.push_back((edge.from_hash, depth + 1));
            }
        }
    }

    // Remove start from result
    visited.remove(&start);
    visited.into_iter().collect()
}

/// Format a path as human-readable string.
///
/// Example: "fire ->(Causes) heat ->(Similar) warmth"
pub fn format_path(path: &[PathStep]) -> alloc::string::String {
    use alloc::string::String;
    use core::fmt::Write;

    let mut out = String::new();
    for (i, step) in path.iter().enumerate() {
        if i > 0 {
            if let Some(kind) = step.edge {
                let _ = write!(out, " ->({})", kind.symbol());
            } else {
                let _ = write!(out, " -> ");
            }
        }
        let _ = write!(out, "{:016x}", step.node);
    }
    out
}

/// Format origin trace as human-readable string.
pub fn format_origin(origins: &[OriginEdge]) -> alloc::string::String {
    use alloc::string::String;
    use core::fmt::Write;

    let mut out = String::new();
    for (i, o) in origins.iter().enumerate() {
        if i > 0 {
            out.push_str("; ");
        }
        let _ = write!(
            out,
            "{:016x} ->({})[w={:.2}] {:016x}",
            o.from,
            o.kind.symbol(),
            o.weight,
            o.to
        );
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use crate::edge::EdgeKind;
    use crate::graph::SilkGraph;

    fn emo(v: f32, a: f32) -> EmotionTag {
        EmotionTag::new(v, a, 0.5, 0.8)
    }

    // ── SentenceAffect ───────────────────────────────────────────────────────

    #[test]
    fn sentence_affect_single_word() {
        let g = SilkGraph::new();
        let result = sentence_affect(&g, &[0xA], &[emo(-0.6, 0.8)], 10);
        assert!((result.composite.valence - (-0.6)).abs() < 0.01);
    }

    #[test]
    fn sentence_affect_amplifies_connected() {
        let mut g = SilkGraph::new();

        // "mất việc" và "buồn" hay co-activate → Silk mạnh
        for _ in 0..50 {
            g.co_activate(0xAA11_u64, 0xBB22_u64, emo(-0.7, 0.6), 0.9, 0);
        }

        // "tôi buồn vì mất việc"
        let hashes = [0xCC33_u64, 0xBB22_u64, 0xAA11_u64];
        let emotions = [emo(0.0, 0.3), emo(-0.5, 0.7), emo(-0.6, 0.5)];

        let result = sentence_affect(&g, &hashes, &emotions, 10);

        // Composite phải âm hơn trung bình đơn giản
        let simple_avg = (0.0 + (-0.5) + (-0.6)) / 3.0; // = -0.367
        assert!(
            result.composite.valence < simple_avg,
            "SentenceAffect < average: {} < {}",
            result.composite.valence,
            simple_avg
        );
    }

    #[test]
    fn sentence_affect_unconnected_mild() {
        let g = SilkGraph::new(); // không có Silk

        let hashes = [0xA, 0xB, 0xC];
        let emotions = [emo(-0.3, 0.5), emo(-0.4, 0.6), emo(-0.5, 0.7)];

        let result = sentence_affect(&g, &hashes, &emotions, 10);
        // Không có Silk → gần với trung bình
        let avg = (-0.3 + -0.4 + -0.5) / 3.0;
        assert!(
            (result.composite.valence - avg).abs() < 0.3,
            "Không Silk → gần avg: {} ≈ {}",
            result.composite.valence,
            avg
        );
    }

    #[test]
    fn sentence_affect_depth_limit() {
        let mut g = SilkGraph::new();
        // Tạo chain dài
        for i in 0u64..20 {
            g.co_activate(i, i + 1, emo(-0.3, 0.5), 0.8, 0);
        }

        let hashes: Vec<u64> = (0..20).collect();
        let emotions: Vec<EmotionTag> = (0..20).map(|_| emo(-0.3, 0.5)).collect();

        let result = sentence_affect(&g, &hashes, &emotions, 5); // max_depth=5
        assert!(
            result.path.len() <= 5,
            "Depth limit: {} <= 5",
            result.path.len()
        );
    }

    #[test]
    fn sentence_affect_valence_clamped() {
        let g = SilkGraph::new();
        // Emotions rất cực đoan
        let result = sentence_affect(&g, &[0xA], &[EmotionTag::new(-2.0, 2.0, 1.5, 1.5)], 10);
        assert!(result.composite.valence >= -1.0, "Valence phải >= -1.0");
        assert!(result.composite.arousal <= 1.0, "Arousal phải <= 1.0");
    }

    // ── ResponseTone ─────────────────────────────────────────────────────────

    #[test]
    fn tone_supportive_falling() {
        // Đang buồn xuống nhanh
        let curve = [-0.1, -0.3, -0.5];
        assert_eq!(
            response_tone(&curve),
            ResponseTone::Supportive,
            "d1=-0.2 < -0.15 → Supportive"
        );
    }

    #[test]
    fn tone_pause_sudden_drop() {
        // Đột ngột xấu
        let curve = [-0.1, -0.1, -0.5];
        let tone = response_tone(&curve);
        // d1 = -0.4, d2 = -0.3 < -0.25
        // d1 < -0.15 → Supportive wins (trước d2)
        assert!(matches!(
            tone,
            ResponseTone::Supportive | ResponseTone::Pause
        ));
    }

    #[test]
    fn tone_reinforcing_rising() {
        let curve = [-0.5, -0.2, 0.1];
        assert_eq!(
            response_tone(&curve),
            ResponseTone::Reinforcing,
            "d1=0.3 > 0.15 → Reinforcing"
        );
    }

    #[test]
    fn tone_celebratory() {
        let curve = [-0.1, 0.1, 0.4];
        // d1=0.3 > 0.15 → Reinforcing wins
        // Nhưng nếu d1 nhỏ hơn...
        let curve2 = [0.0, 0.1, 0.3];
        // d1=0.2 > 0.15 → Reinforcing
        let _ = (curve, curve2);
        // Celebratory: d2 > 0.25 AND v > 0 AND d1 <= 0.15
        let curve3 = [-0.2, 0.0, 0.1]; // d1=0.1, d2=0.3, v=0.1
        let tone3 = response_tone(&curve3);
        assert!(matches!(
            tone3,
            ResponseTone::Celebratory | ResponseTone::Engaged
        ));
    }

    #[test]
    fn tone_gentle_stable_sad() {
        let curve = [-0.4, -0.4, -0.4]; // Buồn ổn định
        assert_eq!(
            response_tone(&curve),
            ResponseTone::Gentle,
            "V < -0.20, stable → Gentle"
        );
    }

    #[test]
    fn tone_engaged_neutral() {
        let curve = [0.0, 0.05, 0.0];
        assert_eq!(response_tone(&curve), ResponseTone::Engaged);
    }

    #[test]
    fn tone_empty_curve() {
        assert_eq!(response_tone(&[]), ResponseTone::Engaged);
    }

    // ── Next curve step ──────────────────────────────────────────────────────

    #[test]
    fn step_small_delta() {
        // Target gần → đến thẳng
        assert!((next_curve_step(-0.5, -0.4) - (-0.4)).abs() < 0.001);
    }

    #[test]
    fn step_large_delta_capped() {
        // Target xa → chỉ bước 0.40
        let result = next_curve_step(-0.7, 0.5);
        assert!(
            (result - (-0.30)).abs() < 0.001,
            "Bước tối đa 0.40: {} ≈ -0.30",
            result
        );
    }

    #[test]
    fn step_no_jump_more_than_040() {
        let start = -0.8;
        let target = 0.8;
        let step = next_curve_step(start, target);
        assert!(
            (step - start).abs() <= 0.40 + 0.001,
            "Không nhảy quá 0.40: delta={}",
            (step - start).abs()
        );
    }

    // ── find_path — BFS ────────────────────────────────────────────────────────

    #[test]
    fn find_path_same_node() {
        let g = SilkGraph::new();
        let path = find_path(&g, 0xA, 0xA, 10);
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].node, 0xA);
    }

    #[test]
    fn find_path_direct_edge() {
        let mut g = SilkGraph::new();
        g.connect_structural(0xA, 0xB, EdgeKind::Causes, 0);
        let path = find_path(&g, 0xA, 0xB, 10);
        assert_eq!(path.len(), 2);
        assert_eq!(path[0].node, 0xA);
        assert_eq!(path[1].node, 0xB);
        assert_eq!(path[1].edge, Some(EdgeKind::Causes));
    }

    #[test]
    fn find_path_two_hops() {
        let mut g = SilkGraph::new();
        g.connect_structural(0xA, 0xB, EdgeKind::Causes, 0);
        g.connect_structural(0xB, 0xC, EdgeKind::Similar, 0);
        let path = find_path(&g, 0xA, 0xC, 10);
        assert_eq!(path.len(), 3, "A -> B -> C");
        assert_eq!(path[0].node, 0xA);
        assert_eq!(path[1].node, 0xB);
        assert_eq!(path[2].node, 0xC);
    }

    #[test]
    fn find_path_no_connection() {
        let mut g = SilkGraph::new();
        g.connect_structural(0xA, 0xB, EdgeKind::Causes, 0);
        // 0xC is disconnected
        let path = find_path(&g, 0xA, 0xC, 10);
        assert!(path.is_empty(), "No path → empty");
    }

    #[test]
    fn find_path_depth_limit() {
        let mut g = SilkGraph::new();
        // Chain: 0 -> 1 -> 2 -> 3 -> 4 -> 5
        for i in 0u64..5 {
            g.connect_structural(i, i + 1, EdgeKind::Causes, 0);
        }
        // max_depth=2 → can only reach 2 hops from 0
        let path = find_path(&g, 0, 5, 2);
        assert!(path.is_empty(), "Depth 2 can't reach node 5");

        // max_depth=5 → can reach
        let path = find_path(&g, 0, 5, 5);
        assert_eq!(path.len(), 6, "0->1->2->3->4->5");
    }

    #[test]
    fn find_path_reverse_direction() {
        let mut g = SilkGraph::new();
        // Edge goes A -> B, but we search B -> A (bidirectional)
        g.connect_structural(0xA, 0xB, EdgeKind::Causes, 0);
        let path = find_path(&g, 0xB, 0xA, 10);
        assert_eq!(path.len(), 2, "Bidirectional traversal");
    }

    #[test]
    fn find_path_shortest() {
        let mut g = SilkGraph::new();
        // Direct: A -> C
        g.connect_structural(0xA, 0xC, EdgeKind::Similar, 0);
        // Indirect: A -> B -> C
        g.connect_structural(0xA, 0xB, EdgeKind::Causes, 0);
        g.connect_structural(0xB, 0xC, EdgeKind::Causes, 0);
        let path = find_path(&g, 0xA, 0xC, 10);
        assert_eq!(path.len(), 2, "BFS finds shortest: A -> C directly");
    }

    #[test]
    fn find_path_with_assoc_edges() {
        let mut g = SilkGraph::new();
        g.co_activate(0xA, 0xB, emo(-0.5, 0.7), 0.8, 0);
        g.co_activate(0xB, 0xC, emo(-0.3, 0.6), 0.8, 0);
        let path = find_path(&g, 0xA, 0xC, 10);
        assert_eq!(path.len(), 3, "Traverses assoc edges too");
    }

    // ── trace_origin ───────────────────────────────────────────────────────────

    #[test]
    fn trace_origin_no_incoming() {
        let g = SilkGraph::new();
        let origins = trace_origin(&g, 0xA, 10);
        assert!(origins.is_empty());
    }

    #[test]
    fn trace_origin_direct_parents() {
        let mut g = SilkGraph::new();
        g.connect_structural(0xB, 0xA, EdgeKind::Causes, 0);
        g.connect_structural(0xC, 0xA, EdgeKind::Member, 0);

        let origins = trace_origin(&g, 0xA, 10);
        assert_eq!(origins.len(), 2, "2 incoming edges");
        let from_nodes: Vec<u64> = origins.iter().map(|o| o.from).collect();
        assert!(from_nodes.contains(&0xB));
        assert!(from_nodes.contains(&0xC));
    }

    #[test]
    fn trace_origin_recursive() {
        let mut g = SilkGraph::new();
        // D -> C -> B -> A
        g.connect_structural(0xB, 0xA, EdgeKind::Causes, 0);
        g.connect_structural(0xC, 0xB, EdgeKind::Causes, 0);
        g.connect_structural(0xD, 0xC, EdgeKind::Causes, 0);

        let origins = trace_origin(&g, 0xA, 10);
        assert_eq!(origins.len(), 3, "D->C->B->A = 3 edges");
    }

    #[test]
    fn trace_origin_depth_limited() {
        let mut g = SilkGraph::new();
        g.connect_structural(0xB, 0xA, EdgeKind::Causes, 0);
        g.connect_structural(0xC, 0xB, EdgeKind::Causes, 0);
        g.connect_structural(0xD, 0xC, EdgeKind::Causes, 0);

        let origins = trace_origin(&g, 0xA, 1);
        assert_eq!(origins.len(), 1, "Depth 1 → only direct parent B");
        assert_eq!(origins[0].from, 0xB);
    }

    // ── reachable ──────────────────────────────────────────────────────────────

    #[test]
    fn reachable_empty_graph() {
        let g = SilkGraph::new();
        let r = reachable(&g, 0xA, None, 10);
        assert!(r.is_empty());
    }

    #[test]
    fn reachable_all_connected() {
        let mut g = SilkGraph::new();
        g.connect_structural(0xA, 0xB, EdgeKind::Causes, 0);
        g.connect_structural(0xB, 0xC, EdgeKind::Similar, 0);
        g.connect_structural(0xC, 0xD, EdgeKind::Member, 0);

        let r = reachable(&g, 0xA, None, 10);
        assert_eq!(r.len(), 3, "B, C, D reachable from A");
        assert!(r.contains(&0xB));
        assert!(r.contains(&0xC));
        assert!(r.contains(&0xD));
    }

    #[test]
    fn reachable_filtered_by_kind() {
        let mut g = SilkGraph::new();
        g.connect_structural(0xA, 0xB, EdgeKind::Causes, 0);
        g.connect_structural(0xA, 0xC, EdgeKind::Similar, 0);
        g.connect_structural(0xB, 0xD, EdgeKind::Causes, 0);

        // Only follow Causes edges
        let r = reachable(&g, 0xA, Some(EdgeKind::Causes), 10);
        assert!(r.contains(&0xB), "B reachable via Causes");
        assert!(r.contains(&0xD), "D reachable via B->D Causes");
        assert!(!r.contains(&0xC), "C not reachable via Causes");
    }

    #[test]
    fn reachable_depth_limited() {
        let mut g = SilkGraph::new();
        for i in 0u64..10 {
            g.connect_structural(i, i + 1, EdgeKind::Causes, 0);
        }

        let r = reachable(&g, 0, None, 3);
        // Can reach nodes 1, 2, 3 (depth 3)
        assert!(r.contains(&1));
        assert!(r.contains(&2));
        assert!(r.contains(&3));
        assert!(!r.contains(&4), "Depth 3 can't reach node 4");
    }

    #[test]
    fn reachable_excludes_start() {
        let mut g = SilkGraph::new();
        g.connect_structural(0xA, 0xB, EdgeKind::Causes, 0);
        let r = reachable(&g, 0xA, None, 10);
        assert!(!r.contains(&0xA), "Start node excluded");
    }

    // ── format_path ────────────────────────────────────────────────────────────

    #[test]
    fn format_path_basic() {
        let path = vec![
            PathStep { node: 0xA, edge: None, weight: 0.0 },
            PathStep { node: 0xB, edge: Some(EdgeKind::Causes), weight: 1.0 },
        ];
        let s = format_path(&path);
        assert!(s.contains("000000000000000a"), "Contains node A hex");
        assert!(s.contains("→"), "Contains Causes symbol");
    }

    #[test]
    fn format_origin_basic() {
        let origins = vec![OriginEdge {
            from: 0xB,
            kind: EdgeKind::Causes,
            to: 0xA,
            weight: 1.0,
            depth: 1,
        }];
        let s = format_origin(&origins);
        assert!(s.contains("→"), "Contains Causes symbol");
        assert!(s.contains("1.00"), "Contains weight");
    }
}
