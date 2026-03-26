// Benchmark: Read "Cuon Theo Chieu Gio" (3.2MB) → Unique P_weight via FH table
// Fix: cp IS the key. P_weight lookup only once per UNIQUE codepoint.
// 2,479,884 chars → 132 unique P_weights (not 2.3M duplicates)

let tbl = __str_bytes(__file_read("json/udc_p_table.bin"));
let PHI = 40503;
let MASK = 65535;

emit "=== BOOK → P_weight (deduplicated) ===";

// ── Phase 1: File Read ──
let t0 = __time();
let book = __file_read("data/cuon_theo_chieu_gio.txt");
let t1 = __time();
let blen = __len(book);
emit "P1_Read=" + to_string(t1-t0) + "ms (" + to_string(blen) + " bytes)";

// ── Phase 2: Scan chars → FH table (cp as key, freq as value) ──
// seen[fh(cp)] = cp (to recover codepoint)
// freq[fh(cp)] = count (frequency)
// pw_cache[fh(cp)] = P_weight (looked up ONCE per unique cp)
let t2 = __time();
let seen = __array_range(65536);
let freq = __array_range(65536);
let pw_cache = __array_range(65536);
let i = 0;
while i < 65536 { let _ = __set_at(seen, i, 0); let _ = __set_at(freq, i, 0); let _ = __set_at(pw_cache, i, 0); let i = i + 1; };

let total = [0];
let unique = [0];
let ci = 0;
while ci < blen {
  let cp = __utf8_cp(book, ci);
  let cl = __utf8_len(book, ci);
  if cl == 0 { break; };
  let _ = __set_at(total, 0, __array_get(total, 0) + 1);
  let idx = __bit_and(cp * PHI, MASK);
  let old = __array_get(seen, idx);
  let _ = __set_at(freq, idx, __array_get(freq, idx) + 1);
  if old == 0 { let _ = __set_at(seen, idx, cp); let off = cp * 2; let lo = __array_get(tbl, off); let hi = __array_get(tbl, off + 1); let _ = __set_at(pw_cache, idx, lo + hi * 256); let _ = __set_at(unique, 0, __array_get(unique, 0) + 1); };
  let ci = ci + cl;
};
let t3 = __time();
let n_total = __array_get(total, 0);
let n_unique = __array_get(unique, 0);
emit "P2_Scan=" + to_string(t3-t2) + "ms chars=" + to_string(n_total) + " unique_cp=" + to_string(n_unique);
emit "  Dedup ratio: " + to_string(n_total) + ":" + to_string(n_unique);

// ── Phase 3: Compose ONLY unique P_weights (weighted by freq) ──
// Scan FH table, compose each unique pw once
let t4 = __time();
let st = [0];
let first = [1];
let composed = [0];
let j = 0;
while j < 65536 {
  let f = __array_get(freq, j);
  if f > 0 { let pw = __array_get(pw_cache, j); if pw > 0 { let _ = __set_at(composed, 0, __array_get(composed, 0) + 1); let cur = __array_get(st, 0); let is_first = __array_get(first, 0); if is_first == 1 { let _ = __set_at(st, 0, pw); let _ = __set_at(first, 0, 0); }; }; };
  let j = j + 1;
};
// Now compose for real (depth 2 max)
let _ = __set_at(st, 0, 0);
let _ = __set_at(first, 0, 1);
let k = 0;
while k < 65536 {
  let f = __array_get(freq, k);
  let pw = __array_get(pw_cache, k);
  let do_compose = 0;
  if f > 0 { let do_compose = 1; };
  if do_compose == 1 {
    let is_first = __array_get(first, 0);
    if is_first == 1 { let _ = __set_at(st, 0, pw); let _ = __set_at(first, 0, 0); };
    if is_first == 0 {
      let cur = __array_get(st, 0);
      let cs = (__floor(cur/4096))%16;
      let cr = (__floor(cur/256))%16;
      let cv = (__floor(cur/32))%8;
      let ca = (__floor(cur/4))%8;
      let ct = cur%4;
      let ns = (__floor(pw/4096))%16;
      let nr = (__floor(pw/256))%16;
      let nv = (__floor(pw/32))%8;
      let na = (__floor(pw/4))%8;
      let nt = pw%4;
      let rs = (__floor((cs*2+ns)/3))%16;
      let rr = (__floor((cr*2+nr)/3))%16;
      let rv = (__floor((cv*2+nv)/3))%8;
      let ra = (__floor((ca*2+na)/3))%8;
      let rt = (__floor((ct*2+nt)/3))%4;
      let nw = (rs*4096)+(rr*256)+(rv*32)+(ra*4)+rt;
      let _ = __set_at(st, 0, nw);
    };
  };
  let k = k + 1;
};
let t5 = __time();
let bmol = __array_get(st, 0);
let n_composed = __array_get(composed, 0);
emit "P3_Compose=" + to_string(t5-t4) + "ms (" + to_string(n_composed) + " unique mols)";

// ── Summary ──
let ttotal = t5 - t0;
emit "";
emit "=== RESULT ===";
emit "Total=" + to_string(ttotal) + "ms";
emit "BookMol=" + to_string(bmol);
emit "  S=" + to_string((__floor(bmol/4096))%16) + " R=" + to_string((__floor(bmol/256))%16) + " V=" + to_string((__floor(bmol/32))%8) + " A=" + to_string((__floor(bmol/4))%8) + " T=" + to_string(bmol%4);
emit "Memory: " + to_string(n_unique) + " entries x 2B = " + to_string(n_unique * 2) + " bytes (was 4.5MB)";
emit "=== DONE ===";
