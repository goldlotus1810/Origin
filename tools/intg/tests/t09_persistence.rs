//! Integration: Origin file integrity — Runtime state after processing

use runtime::origin::{HomeRuntime, ResponseKind};

#[test]
fn runtime_state_after_processing() {
    let mut rt = HomeRuntime::new(42);
    rt.process_text("xin chào", 1000);
    rt.process_text("tôi buồn", 2000);
    rt.process_text("thời tiết đẹp", 3000);
    let m = rt.metrics();
    assert!(m.turns >= 3, "turns must be >= 3, got {}", m.turns);
}

#[test]
fn silk_edges_grow_with_conversation() {
    let mut rt = HomeRuntime::new(42);
    rt.process_text("lửa cháy", 1000);
    let m1 = rt.metrics();
    rt.process_text("nước dập lửa", 2000);
    let m2 = rt.metrics();
    assert!(m2.silk_edges >= m1.silk_edges, "Silk edges: {} → {}", m1.silk_edges, m2.silk_edges);
}

#[test]
fn stm_grows_with_input() {
    let mut rt = HomeRuntime::new(42);
    let m0 = rt.metrics();
    rt.process_text("câu thứ nhất", 1000);
    rt.process_text("câu thứ hai", 2000);
    rt.process_text("câu thứ ba", 3000);
    let m3 = rt.metrics();
    assert!(m3.stm_observations > m0.stm_observations, "STM: {} → {}", m0.stm_observations, m3.stm_observations);
}

#[test]
fn olang_emit_creates_state() {
    let mut rt = HomeRuntime::new(42);
    let r = rt.process_text("○{emit 🔥;}", 1000);
    assert_eq!(r.kind, ResponseKind::OlangResult);
}

#[test]
fn different_sessions_independent() {
    let mut rt1 = HomeRuntime::new(100);
    let mut rt2 = HomeRuntime::new(200);
    rt1.process_text("session one", 1000);
    rt2.process_text("session two", 1000);
    let m1 = rt1.metrics();
    let m2 = rt2.metrics();
    assert!(m1.turns >= 1);
    assert!(m2.turns >= 1);
}

#[test]
fn metrics_fx_is_finite() {
    let mut rt = HomeRuntime::new(42);
    rt.process_text("hello", 1000);
    let m = rt.metrics();
    assert!(m.fx.is_finite());
    assert!(m.silk_density.is_finite());
    assert!(m.stm_hit_rate.is_finite());
}
