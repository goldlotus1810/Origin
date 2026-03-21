// homeos/instinct.ol — 7 Bản năng bẩm sinh (L0, KHÔNG học)
// Thứ tự ưu tiên: Honesty → Contradiction → Causality → Abstraction →
//                  Analogy → Curiosity → Reflection
// Honesty LUÔN chạy trước: không đủ evidence → im lặng.

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

  // ② Contradiction — valence opposition detection
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

  // ⑥ Curiosity — 1 - nearest_similarity → novelty score
  result.novelty = assess_curiosity(observation, knowledge);

  // ⑦ Reflection — knowledge quality check
  result.quality = assess_quality(knowledge);

  return result;
}

// ── ① Honesty ──

fn assess_confidence(observation, knowledge) {
  // Confidence based on:
  // - How many similar concepts exist in knowledge
  // - Fire count of related nodes
  // - Silk connectivity
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

  // More similar concepts → higher confidence
  if similar_count >= 5 { return 0.90; }  // Fact level
  if similar_count >= 3 { return 0.70; }  // Opinion level
  if similar_count >= 1 { return 0.50; }  // Hypothesis level
  return 0.20;  // Below silence threshold
}

// Honesty levels
pub fn confidence_label(conf) {
  if conf >= 0.90 { return "fact"; }
  if conf >= 0.70 { return "opinion"; }
  if conf >= 0.40 { return "hypothesis"; }
  return "silence";
}

// ── ② Contradiction ──

fn detect_contradiction(observation, knowledge) {
  // Check if new observation contradicts existing knowledge
  // Valence opposition: V > neutral vs V < neutral for same concept
  let mol = observation.mol;
  let i = 0;
  let entries = knowledge.stm.entries;
  let n = len(entries);

  while i < n {
    let existing = entries[i];
    let sim = similarity(mol, existing.mol);
    if sim > 0.7 {
      // Similar concept — check valence direction (V range 0-7, neutral=4)
      let v_new = mol_valence(mol);
      let v_old = mol_valence(existing.mol);
      // Opposition: one > 5 (positive) and one < 3 (negative)
      if (v_new > 5 && v_old < 3) || (v_new < 3 && v_old > 5) {
        return true;
      }
    }
    i = i + 1;
  }
  return false;
}

// ── ③ Causality ──

fn detect_causality(observation, knowledge) {
  // Temporal co-activation + Relation::Causes → causal link
  // Simplified: if two concepts frequently co-activate with Causes relation
  let mol = observation.mol;
  if mol_relation(mol) == 6 { return true; }  // Relation = Causes
  return false;
}

// ── ④ Abstraction ──

fn abstract_concept(observation, knowledge) {
  // Find LCA of similar concepts → categorical label
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

// ── ⑤ Analogy ──

fn find_analogy(observation, knowledge) {
  // A:B :: C:? → find D where delta(A,B) ≈ delta(C,D)
  // Simplified: find the dimension that changes most
  let mol = observation.mol;
  let entries = knowledge.stm.entries;
  if len(entries) < 2 { return ""; }

  // Compare with most recent entry
  let prev = entries[len(entries) - 1].mol;
  let delta = dimension_delta(prev, mol);

  if delta.delta > 2 {
    return "evolve(" + delta.dim + ")";
  }
  return "";
}

// ── ⑥ Curiosity ──

fn assess_curiosity(observation, knowledge) {
  // Novelty = 1 - nearest_similarity
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

// ── ⑦ Reflection ──

fn assess_quality(knowledge) {
  // Knowledge quality: ratio of well-connected nodes
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
