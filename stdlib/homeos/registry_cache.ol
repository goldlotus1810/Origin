// stdlib/homeos/registry_cache.ol — LRU cache for Registry lookups
// PLAN 5.2.2: Cache top-N hot nodes to avoid repeated HashMap lookup.
// Cache size = 55 (Fib[10]) per QT⑰.

let CACHE_SIZE = 55;

pub fn cache_new() {
  return { entries: [], hits: 0, misses: 0 };
}

pub fn cache_get(cache, hash) {
  // Linear search (small N → fast enough)
  let i = 0;
  let n = len(cache.entries);
  while i < n {
    let entry = cache.entries[i];
    if entry.hash == hash {
      // HIT: move to front (LRU promotion)
      cache.hits = cache.hits + 1;
      if i > 0 {
        // Remove from current position
        let promoted = cache.entries[i];
        let j = i;
        while j > 0 {
          cache.entries[j] = cache.entries[j - 1];
          j = j - 1;
        }
        cache.entries[0] = promoted;
      }
      return entry.node;
    }
    i = i + 1;
  }
  // MISS
  cache.misses = cache.misses + 1;
  return 0;
}

pub fn cache_put(cache, hash, node) {
  // Check if already cached (update + promote)
  let i = 0;
  let n = len(cache.entries);
  while i < n {
    if cache.entries[i].hash == hash {
      cache.entries[i].node = node;
      // Promote to front
      if i > 0 {
        let promoted = cache.entries[i];
        let j = i;
        while j > 0 {
          cache.entries[j] = cache.entries[j - 1];
          j = j - 1;
        }
        cache.entries[0] = promoted;
      }
      return;
    }
    i = i + 1;
  }
  // New entry: insert at front
  let entry = { hash: hash, node: node };
  // Shift all entries right by 1
  push(cache.entries, entry);  // extend array
  let j = len(cache.entries) - 1;
  while j > 0 {
    cache.entries[j] = cache.entries[j - 1];
    j = j - 1;
  }
  cache.entries[0] = entry;
  // Evict last if over capacity
  if len(cache.entries) > CACHE_SIZE {
    pop(cache.entries);
  }
}

pub fn cache_invalidate(cache, hash) {
  // Remove specific entry (called on amend/evolve)
  let i = 0;
  let n = len(cache.entries);
  while i < n {
    if cache.entries[i].hash == hash {
      // Shift left to fill gap
      let j = i;
      while j < n - 1 {
        cache.entries[j] = cache.entries[j + 1];
        j = j + 1;
      }
      pop(cache.entries);
      return;
    }
    i = i + 1;
  }
}

pub fn cache_clear(cache) {
  cache.entries = [];
  cache.hits = 0;
  cache.misses = 0;
}

pub fn cache_stats(cache) {
  let total = cache.hits + cache.misses;
  let ratio = 0.0;
  if total > 0 { ratio = cache.hits / total; }
  return { hits: cache.hits, misses: cache.misses,
           total: total, hit_ratio: ratio,
           size: len(cache.entries) };
}
