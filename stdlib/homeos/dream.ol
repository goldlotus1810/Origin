// homeos/dream.ol — Offline consolidation (STM → cluster → promote)
// Dream = neuron analog: dendrites consolidate during sleep

// ── STM (Short-Term Memory) ──

pub fn stm_new() {
  return { entries: [], max_size: 1000 };
}

pub fn stm_push(stm, chain_hash, mol, emotion) {
  push(stm.entries, {
    hash: chain_hash,
    mol: mol,
    emotion: emotion,
    fire_count: 1,
    weight: 0.0
  });
  // Cap size
  if len(stm.entries) > stm.max_size {
    stm.entries = skip(stm.entries, len(stm.entries) - stm.max_size);
  }
}

pub fn stm_fire(stm, chain_hash) {
  // Increment fire count for existing entry
  let i = 0;
  while i < len(stm.entries) {
    if stm.entries[i].hash == chain_hash {
      stm.entries[i].fire_count = stm.entries[i].fire_count + 1;
      return;
    }
    i = i + 1;
  }
}

// ── Dream Cycle ──

pub fn dream_cycle(stm, silk_graph) {
  // STM → cluster → score → propose QR promotion
  let entries = stm.entries;
  if len(entries) < 3 { return []; }

  // 1. Cluster by 5D similarity
  let clusters = cluster_5d(entries);

  // 2. Score each cluster
  let scored = [];
  let i = 0;
  while i < len(clusters) {
    let c = clusters[i];
    if len(c) >= 2 {  // minimum cluster size
      let score = dream_score(c, silk_graph);
      push(scored, { cluster: c, score: score });
    }
    i = i + 1;
  }

  // 3. Sort by score (descending)
  let sorted = sort_by(scored, fn(s) { return 0.0 - s.score; });

  // 4. Return top 3 proposals
  return take(sorted, 3);
}

fn dream_score(cluster, silk_graph) {
  // α=0.3 (cohesion) + β=0.4 (fire_count) + γ=0.3 (connectivity)
  let cohesion = avg_pairwise_similarity(cluster);
  let fire = avg_fire_count(cluster);
  let connectivity = avg_connectivity(cluster, silk_graph);

  return 0.3 * cohesion + 0.4 * fire + 0.3 * connectivity;
}

fn avg_pairwise_similarity(cluster) {
  let n = len(cluster);
  if n < 2 { return 0.0; }
  let total = 0.0;
  let pairs = 0;
  let i = 0;
  while i < n {
    let j = i + 1;
    while j < n {
      total = total + similarity(cluster[i].mol, cluster[j].mol);
      pairs = pairs + 1;
      j = j + 1;
    }
    i = i + 1;
  }
  if pairs == 0 { return 0.0; }
  return total / pairs;
}

fn avg_fire_count(cluster) {
  let total = 0.0;
  let i = 0;
  let n = len(cluster);
  while i < n {
    total = total + cluster[i].fire_count;
    i = i + 1;
  }
  // Normalize: fire_count / fib threshold
  return (total / n) / 8.0;  // Fib[6] = 8
}

fn avg_connectivity(cluster, silk_graph) {
  let total = 0.0;
  let i = 0;
  let n = len(cluster);
  while i < n {
    let nb = neighbors(silk_graph, cluster[i].hash);
    total = total + len(nb);
    i = i + 1;
  }
  if n == 0 { return 0.0; }
  // Normalize: max 20 connections = 1.0
  return min(total / n / 20.0, 1.0);
}

// ── Clustering (simple greedy, threshold = φ⁻¹) ──

fn cluster_5d(entries) {
  let threshold = 0.382;  // φ⁻¹ ≈ 0.382
  let clusters = [];
  let assigned = [];  // track which entries are assigned

  let i = 0;
  let n = len(entries);
  while i < n {
    push(assigned, false);
    i = i + 1;
  }

  i = 0;
  while i < n {
    if assigned[i] { i = i + 1; continue; }

    // Start new cluster with entry i
    let cluster = [entries[i]];
    assigned[i] = true;

    // Find similar entries
    let j = i + 1;
    while j < n {
      if !assigned[j] {
        let sim = similarity(entries[i].mol, entries[j].mol);
        if sim >= threshold {
          push(cluster, entries[j]);
          assigned[j] = true;
        }
      }
      j = j + 1;
    }

    push(clusters, cluster);
    i = i + 1;
  }

  return clusters;
}

// ── Promote proposal ──

pub fn should_promote(entry, fib_threshold) {
  // Maturity check: fire_count >= fib(depth) AND weight >= 0.854
  return entry.fire_count >= fib_threshold && entry.weight >= 0.854;
}

fn fib(n) {
  if n <= 1 { return n; }
  let a = 0;
  let b = 1;
  let i = 2;
  while i <= n {
    let tmp = a + b;
    a = b;
    b = tmp;
    i = i + 1;
  }
  return b;
}
