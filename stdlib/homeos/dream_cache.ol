// stdlib/homeos/dream_cache.ol — Dream cluster score memoization
// PLAN 5.2.4: Cache cluster scores, only recalculate when nodes change.
// Avoids full recalculation every Dream cycle.

pub fn memo_new() {
  return { scores: {}, versions: {}, current_ver: 0 };
}

pub fn memo_bump_version(memo) {
  memo.current_ver = memo.current_ver + 1;
  return memo.current_ver;
}

pub fn memo_invalidate(memo, cluster_id) {
  // Mark cluster as dirty — will be recalculated next access
  memo.versions[cluster_id] = -1;
}

pub fn memo_get_score(memo, cluster_id) {
  // Return cached score if valid, -1 if stale/missing
  let ver = memo.versions[cluster_id];
  if ver == memo.current_ver {
    return memo.scores[cluster_id];
  }
  return -1.0;  // sentinel: needs recalculation
}

pub fn memo_set_score(memo, cluster_id, score) {
  memo.scores[cluster_id] = score;
  memo.versions[cluster_id] = memo.current_ver;
}

pub fn memo_invalidate_all(memo) {
  // Called when significant changes occur (e.g., many new observations)
  memo.scores = {};
  memo.versions = {};
}

pub fn memo_stats(memo) {
  let n = 0;
  let valid = 0;
  // Count entries (approximate — dict iteration not available)
  return { version: memo.current_ver };
}
