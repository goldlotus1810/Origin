// homeos/curve.ol — ConversationCurve + tone detection
// f(x) = 0.6 × f_conv(t) + 0.4 × f_dn(nodes)

pub fn curve_new() {
  return { history: [], window: 8 };
}

pub fn curve_push(curve, emotion) {
  push(curve.history, { v: emotion.v, a: emotion.a });
  if len(curve.history) > 32 {
    curve.history = skip(curve.history, len(curve.history) - 32);
  }
}

pub fn curve_tone(curve) {
  let n = len(curve.history);
  if n < 3 { return "neutral"; }

  let v0 = curve.history[n - 3].v;
  let v1 = curve.history[n - 2].v;
  let v2 = curve.history[n - 1].v;

  let f_prime = v2 - v1;
  let f_double = v2 - 2.0 * v1 + v0;

  // Emotional instability check
  if n >= 5 {
    let var = curve_variance(curve, 5);
    if var > 0.3 && f_prime * (v1 - v0) < 0 {
      return "gentle";  // instability → gentle
    }
  }

  if f_prime < -0.15 { return "supportive"; }
  if f_double < -0.25 { return "pause"; }
  if f_prime > 0.15 { return "reinforcing"; }
  if f_double > 0.25 && v2 > 0.0 { return "celebratory"; }
  if v2 < -0.20 { return "gentle"; }

  return "neutral";
}

pub fn curve_variance(curve, window) {
  let n = len(curve.history);
  let start = max(0, n - window);
  let count = n - start;
  if count < 2 { return 0.0; }

  // Mean
  let sum = 0.0;
  let i = start;
  while i < n {
    sum = sum + curve.history[i].v;
    i = i + 1;
  }
  let mean = sum / count;

  // Variance
  let var_sum = 0.0;
  i = start;
  while i < n {
    let d = curve.history[i].v - mean;
    var_sum = var_sum + d * d;
    i = i + 1;
  }
  return var_sum / count;
}

pub fn curve_trend(curve) {
  let n = len(curve.history);
  if n < 2 { return 0.0; };
  return curve.history[n - 1].v - curve.history[n - 2].v;
}

pub fn curve_current_v(curve) {
  let n = len(curve.history);
  if n == 0 { return 0.0; };
  return curve.history[n - 1].v;
}

// f(x) = 0.6 × f_conv(t) + 0.4 × f_dn(nodes)
// f_conv = mean valence over window
// f_dn = node density (how many unique nodes in recent turns)
pub fn curve_fx(curve, node_count) {
  let f_conv = curve_mean_v(curve, curve.window);
  let f_dn = if node_count > 0 { min(node_count / 10.0, 1.0) } else { 0.0 };
  return 0.6 * f_conv + 0.4 * f_dn;
}

pub fn curve_mean_v(curve, window) {
  let n = len(curve.history);
  let start = max(0, n - window);
  let count = n - start;
  if count == 0 { return 0.0; };
  let sum = 0.0;
  let i = start;
  while i < n {
    let sum = sum + curve.history[i].v;
    let i = i + 1;
  };
  return sum / count;
}

pub fn curve_turn_count(curve) {
  return len(curve.history);
}

fn max(a, b) { if a > b { return a; }; return b; }
fn min(a, b) { if a < b { return a; }; return b; }
