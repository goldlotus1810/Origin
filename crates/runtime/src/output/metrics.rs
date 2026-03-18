//! # metrics — Observability cho HomeRuntime
//!
//! RuntimeMetrics: Silk density, Dream frequency, STM hit rate.
//! Snapshot tức thời — không giữ state riêng.

extern crate alloc;
use alloc::format;
use alloc::string::String;

/// Snapshot metrics cho observability.
#[derive(Debug, Clone)]
pub struct RuntimeMetrics {
    /// Số turns đã xử lý.
    pub turns: u64,
    /// Số observations trong STM.
    pub stm_observations: usize,
    /// Số Silk edges.
    pub silk_edges: usize,
    /// Silk density = edges / (nodes × (nodes-1) / 2). Giá trị 0..1.
    pub silk_density: f32,
    /// Tỷ lệ STM hit (fire_count > 1) / total observations.
    pub stm_hit_rate: f32,
    /// f(x) hiện tại — ConversationCurve.
    pub fx: f32,
    /// Tone hiện tại.
    pub tone: String,
    /// Dream cycles đã chạy (actual count).
    pub dream_cycles: u64,
    /// Bytes pending ghi disk.
    pub pending_bytes: usize,
    /// Saveable Silk edges (weight >= 0.30).
    pub saveable_edges: usize,
    /// Max fire_count trong STM — indicator of repeated patterns.
    pub stm_max_fire: u32,
    /// Total proposals approved by AAM across all dream cycles.
    pub dream_approved: u64,
    /// L3 concepts created from Dream consolidation.
    pub dream_l3_concepts: u64,
    /// Current Fibonacci dream interval (turns until next dream).
    pub dream_fib_interval: u32,
    /// KnowTree: total nodes.
    pub knowtree_nodes: u64,
    /// KnowTree: total edges.
    pub knowtree_edges: u64,
    /// KnowTree: L2 sentences.
    pub knowtree_sentences: u64,
    /// KnowTree: L3+ concepts.
    pub knowtree_concepts: u64,
}

impl RuntimeMetrics {
    /// Format metrics thành text.
    pub fn summary(&self) -> String {
        format!(
            "Metrics ○\n\
             turns          : {}\n\
             stm_obs        : {}\n\
             stm_hit_rate   : {:.1}%\n\
             stm_max_fire   : {}\n\
             silk_edges     : {}\n\
             silk_density   : {:.4}\n\
             saveable_edges : {}\n\
             f(x)           : {:.3}\n\
             tone           : {}\n\
             dream_cycles   : {}\n\
             dream_approved : {}\n\
             dream_l3       : {}\n\
             dream_next_in  : {} turns\n\
             knowtree_nodes : {}\n\
             knowtree_edges : {}\n\
             knowtree_L2    : {}\n\
             knowtree_L3    : {}\n\
             pending_bytes  : {}",
            self.turns,
            self.stm_observations,
            self.stm_hit_rate * 100.0,
            self.stm_max_fire,
            self.silk_edges,
            self.silk_density,
            self.saveable_edges,
            self.fx,
            self.tone,
            self.dream_cycles,
            self.dream_approved,
            self.dream_l3_concepts,
            self.dream_fib_interval,
            self.knowtree_nodes,
            self.knowtree_edges,
            self.knowtree_sentences,
            self.knowtree_concepts,
            self.pending_bytes,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_summary_format() {
        let m = RuntimeMetrics {
            turns: 42,
            stm_observations: 15,
            silk_edges: 30,
            silk_density: 0.123,
            stm_hit_rate: 0.6,
            fx: -0.35,
            tone: String::from("Supportive"),
            dream_cycles: 5,
            pending_bytes: 256,
            saveable_edges: 12,
            stm_max_fire: 7,
            dream_approved: 3,
            dream_l3_concepts: 1,
            dream_fib_interval: 8,
            knowtree_nodes: 28,
            knowtree_edges: 349,
            knowtree_sentences: 28,
            knowtree_concepts: 1,
        };
        let s = m.summary();
        assert!(s.contains("turns"), "summary có turns");
        assert!(s.contains("silk_density"), "summary có silk_density");
        assert!(s.contains("stm_hit_rate"), "summary có stm_hit_rate");
        assert!(s.contains("dream_cycles"), "summary có dream_cycles");
        assert!(s.contains("knowtree_nodes"), "summary có knowtree_nodes");
    }
}
