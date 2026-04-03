// homeos/instinct.ol — 7 Bản năng bẩm sinh (L0, KHÔNG học)
// Merged: Origin (Rust) 7 instinct structure + origin.olang P_weight math
// ALL formulas. Pure 5D molecular distance. NO keyword matching.
//
// Thứ tự ưu tiên: Honesty → Contradiction → Causality → Abstraction →
//                  Analogy → Curiosity → Reflection
// Honesty LUÔN chạy trước: không đủ evidence → im lặng.

// ═══ UNIFIED API ═══
// Supports both calling styles:
//   1. run_instincts(observation, knowledge)  — Origin Rust (object-based)
//   2. instinct_route(input)                  — origin.olang (text-based)

pub fn run_instincts(observation, knowledge) {
  let result = {
    action: "process",
    confidence: 0.0,
    contradiction: false,
    causal: false,
    abstraction: "",
    analogy: "",
    novelty: 0.0,
    quality: 0.0
  };

  // ① Honesty — confidence assessment
  result.confidence = assess_confidence(observation, knowledge);
  if result.confidence < 0.40 {
    result.action = "silence";
    return result;
  }

  // ② Contradiction — valence opposition detection (5D math)
  result.contradiction = detect_contradiction(observation, knowledge);
  if result.contradiction {
    result.action = "flag_contradiction";
    return result;
  }

  // ③ Causality — temporal + co-activation → causal link
  result.causal = detect_causality(observation, knowledge);

  // ④ Abstraction — N chains → LCA → categorical
  result.abstraction = abstract_concept(observation, knowledge);

  // ⑤ Analogy — A:B :: C:? → delta 5D
  result.analogy = find_analogy(observation, knowledge);

  // ⑥ Curiosity — novelty from 5D distance to nearest known
  result.novelty = assess_curiosity(observation, knowledge);

  // ⑦ Reflection — knowledge quality check
  result.quality = assess_quality(knowledge);

  return result;
}

// ═══ origin.olang API (P_weight math, array-based) ═══

// G9①: Honesty — confidence from evidence (proximity + silk weight)
pub fn instinct_honesty(_mol) {
  let _near = kt_nearest(_mol);
  if len(_near) == 0 { return 0; }
  let _near_mol = _kt_real_mol(_near);
  let _sw = kt_silk_weight(_mol, _near_mol);
  let _dist = _kt_mol_dist(_mol, _near_mol);
  // Closer = higher confidence, silk = higher confidence
  let _proximity = 1000 - (_dist * 40);
  if _proximity < 0 { _proximity = 0; }
  let _silk_score = _sw;
  let _conf = (_proximity * 300 + _silk_score * 700) / 1000;
  return _conf;  // 0-1000, threshold: 400=silence, 700=think, 900=fact
}

// G9②: Contradiction — V distance + same topic (R similar)
pub fn instinct_contradiction(_a_mol, _b_mol) {
  let _dv = _kt_abs(mol_get_dim(_a_mol, 2) - mol_get_dim(_b_mol, 2));
  let _dr = _kt_abs(mol_get_dim(_a_mol, 1) - mol_get_dim(_b_mol, 1));
  // V very different (>5) + R very similar (<3) = contradiction
  if _dv > 5 { if _dr < 3 { return 1; } }
  return 0;
}

// G9⑥: Curiosity — novelty = distance from known
pub fn instinct_curiosity(_mol) {
  let _near = kt_nearest(_mol);
  if len(_near) == 0 { return 1000; }  // completely novel
  let _d = _kt_mol_dist(_mol, _kt_real_mol(_near));
  // Higher distance = more novel
  let _novelty = _d * 200;
  if _novelty > 1000 { _novelty = 1000; }
  return _novelty;  // >500 = explore, <300 = familiar
}

// G10: SecurityGate — check REAL V/A from P_weight (not keywords)
pub fn security_gate(text) {
  let min_v = 7;
  let max_a = 0;
  let i = 0;
  while i < len(text) {
    let cp = __char_code(char_at(text, i));
    let pw = p_weight(cp);
    if pw > 0 {
      let v = (__floor(pw / 32)) % 8;
      let a = (__floor(pw / 4)) % 8;
      if v < min_v { min_v = v; }
      if a > max_a { max_a = a; }
    }
    i = i + 1;
  }
  // Crisis: min V across ALL chars <= 1 AND max A >= 6
  if min_v <= 1 { if max_a >= 6 { return 1; } }
  return 0;
}

// Instinct route — returns [confidence, V, A, dominant_dim, novelty]
pub fn instinct_route(input) {
  let _mol = _kt_real_mol(input);
  let _v = mol_get_dim(_mol, 2);
  let _a = mol_get_dim(_mol, 3);
  let _conf = instinct_honesty(_mol);
  let _novel = instinct_curiosity(_mol);
  let _dim = mol_dominant_dim(_mol);
  let _result = [];
  push(_result, _conf);   // [0] confidence
  push(_result, _v);      // [1] V
  push(_result, _a);      // [2] A
  push(_result, _dim);    // [3] dominant dimension
  push(_result, _novel);  // [4] novelty
  return _result;
}

// ═══ INTERNALS (Origin Rust object-based) ═══

fn assess_confidence(observation, knowledge) {
  let mol = observation.mol;
  let similar_count = 0;
  let i = 0;
  let entries = knowledge.stm.entries;
  let n = len(entries);
  while i < n {
    if similarity(mol, entries[i].mol) > 0.5 {
      similar_count = similar_count + 1;
    }
    i = i + 1;
  }
  if similar_count >= 5 { return 0.90; }  // Fact level
  if similar_count >= 3 { return 0.70; }  // Opinion level
  if similar_count >= 1 { return 0.50; }  // Hypothesis level
  return 0.20;
}

pub fn confidence_label(conf) {
  if conf >= 0.90 { return "fact"; }
  if conf >= 0.70 { return "opinion"; }
  if conf >= 0.40 { return "hypothesis"; }
  return "silence";
}

fn detect_contradiction(observation, knowledge) {
  let mol = observation.mol;
  let i = 0;
  let entries = knowledge.stm.entries;
  let n = len(entries);
  while i < n {
    let existing = entries[i];
    let sim = similarity(mol, existing.mol);
    if sim > 0.7 {
      let v_new = mol_valence(mol);
      let v_old = mol_valence(existing.mol);
      // 5D math: V distance > 5 = contradiction (same as origin.olang)
      if (v_new > 5 && v_old < 3) || (v_new < 3 && v_old > 5) {
        return true;
      }
    }
    i = i + 1;
  }
  return false;
}

fn detect_causality(observation, knowledge) {
  let mol = observation.mol;
  if mol_relation(mol) == 6 { return true; }
  return false;
}

fn abstract_concept(observation, knowledge) {
  let mol = observation.mol;
  let similar = [];
  let i = 0;
  let entries = knowledge.stm.entries;
  let n = len(entries);
  while i < n {
    if similarity(mol, entries[i].mol) > 0.6 {
      push(similar, entries[i].mol);
    }
    i = i + 1;
  }
  if len(similar) < 2 { return "concrete"; }
  if len(similar) < 5 { return "categorical"; }
  return "abstract";
}

fn find_analogy(observation, knowledge) {
  let mol = observation.mol;
  let entries = knowledge.stm.entries;
  if len(entries) < 2 { return ""; }
  let prev = entries[len(entries) - 1].mol;
  let delta = dimension_delta(prev, mol);
  if delta.delta > 2 {
    return "evolve(" + delta.dim + ")";
  }
  return "";
}

fn assess_curiosity(observation, knowledge) {
  let mol = observation.mol;
  let mols = [];
  let i = 0;
  let entries = knowledge.stm.entries;
  let n = len(entries);
  while i < n {
    push(mols, entries[i].mol);
    i = i + 1;
  }
  let nearest = nearest_similarity(mol, mols);
  return 1.0 - nearest;
}

fn assess_quality(knowledge) {
  let entries = knowledge.stm.entries;
  let n = len(entries);
  if n == 0 { return 0.0; }
  let well_connected = 0;
  let i = 0;
  while i < n {
    if entries[i].fire_count >= 3 {
      well_connected = well_connected + 1;
    }
    i = i + 1;
  }
  return well_connected / n;
}
