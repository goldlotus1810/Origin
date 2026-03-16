//! # metrics — Observability cho HomeRuntime
//!
//! RuntimeMetrics: Silk density, Dream frequency, STM hit rate.
//! Snapshot tức thời — không giữ state riêng.

extern crate alloc;
use alloc::string::String;
use alloc::format;

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
    /// Dream cycles đã chạy (ước tính từ turn count / interval).
    pub dream_cycles_est: u64,
    /// Bytes pending ghi disk.
    pub pending_bytes: usize,
    /// Saveable Silk edges (weight >= 0.30).
    pub saveable_edges: usize,
    /// Max fire_count trong STM — indicator of repeated patterns.
    pub stm_max_fire: u32,
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
             dream_est      : {}\n\
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
            self.dream_cycles_est,
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
            dream_cycles_est: 5,
            pending_bytes: 256,
            saveable_edges: 12,
            stm_max_fire: 7,
        };
        let s = m.summary();
        assert!(s.contains("turns"), "summary có turns");
        assert!(s.contains("silk_density"), "summary có silk_density");
        assert!(s.contains("stm_hit_rate"), "summary có stm_hit_rate");
    }
}
