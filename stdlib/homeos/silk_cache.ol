// stdlib/homeos/silk_cache.ol — Similarity score cache for Silk 5D comparisons
// PLAN 5.2.3: Cache expensive 5D similarity computations.
// Key = order-independent pair (min_hash, max_hash).
// Invalidate when node evolves (5D position changed).

let MAX_ENTRIES = 256;

pub fn sim_cache_new() {
  return { keys: [], values: [], size: 0 };
}

pub fn sim_cache_get(cache, ha, hb) {
  let key = combine_hash(ha, hb);
  let i = 0;
  while i < cache.size {
    if cache.keys[i] == key {
      return cache.values[i];
    }
    i = i + 1;
  }
  return -1.0;  // sentinel: cache miss (similarity is always >= 0)
}

pub fn sim_cache_put(cache, ha, hb, score) {
  let key = combine_hash(ha, hb);
  // Check if already cached (update)
  let i = 0;
  while i < cache.size {
    if cache.keys[i] == key {
      cache.values[i] = score;
      return;
    }
    i = i + 1;
  }
  // New entry
  if cache.size >= MAX_ENTRIES {
    // Evict oldest (index 0)
    let j = 0;
    while j < cache.size - 1 {
      cache.keys[j] = cache.keys[j + 1];
      cache.values[j] = cache.values[j + 1];
      j = j + 1;
    }
    cache.keys[cache.size - 1] = key;
    cache.values[cache.size - 1] = score;
  } else {
    push(cache.keys, key);
    push(cache.values, score);
    cache.size = cache.size + 1;
  }
}

pub fn sim_cache_invalidate_node(cache, h) {
  // Remove all entries containing hash h
  // Since key = combine_hash(a,b), we need to check both directions
  let i = 0;
  while i < cache.size {
    let key = cache.keys[i];
    // Extract ha, hb from combined key
    // combine_hash always puts min first, so check if h could be either part
    let lo = key - floor(key / 4294967296) * 4294967296;
    let hi = floor(key / 4294967296);
    if lo == h || hi == h {
      // Remove this entry (shift left)
      let j = i;
      while j < cache.size - 1 {
        cache.keys[j] = cache.keys[j + 1];
        cache.values[j] = cache.values[j + 1];
        j = j + 1;
      }
      cache.size = cache.size - 1;
      pop(cache.keys);
      pop(cache.values);
      // Don't increment i — check same index again (shifted)
    } else {
      i = i + 1;
    }
  }
}

pub fn sim_cache_clear(cache) {
  cache.keys = [];
  cache.values = [];
  cache.size = 0;
}

fn combine_hash(ha, hb) {
  // Order-independent: always put smaller hash first
  if ha < hb { return ha * 4294967296 + hb; }
  return hb * 4294967296 + ha;
}
