// homeos/silk_ops.ol — Implicit Silk operations
// Silk = implicit relationships from 5D molecular distance
// Hebbian learning: fire together → wire together

// ── Silk Graph (in-memory adjacency list) ──

pub fn silk_new() {
  return { edges: [], edge_count: 0 };
}

// Find edge between two hashes
fn find_edge(graph, hash_a, hash_b) {
  let i = 0;
  while i < graph.edge_count {
    let e = graph.edges[i];
    if (e.from == hash_a && e.to == hash_b) ||
       (e.from == hash_b && e.to == hash_a) {
      return i;
    }
    i = i + 1;
  }
  return -1;
}

// ── Implicit Silk (5D comparison, 0 bytes stored) ──

pub fn implicit_strength(mol_a, mol_b) {
  // 5D distance → relationship strength
  let d = distance_5d(mol_a, mol_b);
  if d < 0.05 { return 1.0; }
  if d > 0.8 { return 0.0; }
  // Smooth falloff using φ⁻¹
  return pow(0.618, d * 5.0);
}

// ── Hebbian Learning ──

pub fn co_activate(graph, hash_a, hash_b, emotion) {
  // Record co-activation between two concepts
  // Edge carries EmotionTag of the moment (QT⑬)
  let idx = find_edge(graph, hash_a, hash_b);

  if idx >= 0 {
    // Update existing edge
    let e = graph.edges[idx];
    e.weight = hebbian_update(e.weight, true);
    e.fire_count = e.fire_count + 1;
    e.emotion = emotion;  // emotion of co-activation moment
  } else {
    // Create new edge
    push(graph.edges, {
      from: hash_a,
      to: hash_b,
      weight: 0.1,
      fire_count: 1,
      emotion: emotion
    });
    graph.edge_count = graph.edge_count + 1;
  }
}

pub fn hebbian_update(weight, co_active) {
  // Fire together → wire together
  // Decay = φ⁻¹ ≈ 0.618 (optimal forgetting rate)
  let lr = 0.01;
  let decay = 0.618;
  if co_active {
    let new_w = weight + lr * (1.0 - weight * decay);
    if new_w > 1.0 { return 1.0; }
    return new_w;
  } else {
    return weight * decay;
  }
}

pub fn decay_all(graph) {
  // Apply decay to all edges (called periodically)
  let i = 0;
  while i < graph.edge_count {
    graph.edges[i].weight = hebbian_update(graph.edges[i].weight, false);
    i = i + 1;
  }
}

// ── Walk (emotion amplification) ──

pub fn walk_emotion(graph, start_hash, emotion, max_depth) {
  // BFS walk from start, accumulate emotion via Silk weights
  // AMPLIFY, not average (core rule)
  let visited = [start_hash];
  let composite = emotion;
  let frontier = neighbors(graph, start_hash);

  let depth = 0;
  while depth < max_depth && len(frontier) > 0 {
    let next_frontier = [];
    let i = 0;
    while i < len(frontier) {
      let edge = frontier[i];
      if index_of(visited, edge.to) < 0 {
        push(visited, edge.to);
        // Amplify: composite = composite * (1 + weight * 0.618)
        composite = amplify_emotion(composite, edge.weight);
        // Add neighbors of this node
        let nb = neighbors(graph, edge.to);
        let j = 0;
        while j < len(nb) {
          push(next_frontier, nb[j]);
          j = j + 1;
        }
      }
      i = i + 1;
    }
    frontier = next_frontier;
    depth = depth + 1;
  }
  return composite;
}

fn amplify_emotion(emo, weight) {
  let factor = 1.0 + weight * 0.618;
  return {
    v: emo.v * factor,
    a: clamp(emo.a * factor, 0.0, 1.0),
    d: emo.d,
    i: clamp(emo.i * factor, 0.0, 1.0)
  };
}

fn clamp(val, lo, hi) {
  if val < lo { return lo; }
  if val > hi { return hi; }
  return val;
}

fn neighbors(graph, hash) {
  let result = [];
  let i = 0;
  while i < graph.edge_count {
    let e = graph.edges[i];
    if e.from == hash {
      push(result, { to: e.to, weight: e.weight });
    } else {
      if e.to == hash {
        push(result, { to: e.from, weight: e.weight });
      }
    }
    i = i + 1;
  }
  return result;
}

// ── Query ──

pub fn strongest_edges(graph, hash, n) {
  let nb = neighbors(graph, hash);
  let sorted = sort_by(nb, fn(e) { return 0.0 - e.weight; });  // descending
  return take(sorted, n);
}

pub fn edge_weight(graph, hash_a, hash_b) {
  let idx = find_edge(graph, hash_a, hash_b);
  if idx >= 0 { return graph.edges[idx].weight; }
  return 0.0;
}
