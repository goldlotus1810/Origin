// homeos/learning.ol — Learning pipeline orchestration
// Merged: Origin (Rust) process_one + origin.olang STM/Dream/QR/Immune/Decode
// Gate → Encode → Instinct → STM → Silk → Curve

// ═══ ORIGIN RUST API (object-based pipeline) ═══

pub fn process_one(text, emotion, context) {
  // 1. SecurityGate (LUÔN chạy trước — bất biến)
  let gate = gate_check(text);
  if gate.action == "crisis" {
    return { ok: false, response: gate.response, action: "crisis" };
  }
  if gate.action == "block" {
    return { ok: false, response: "Nội dung không phù hợp.", action: "block" };
  }

  // 2. Encode text → molecular observation
  let mol = text_to_mol(text, emotion);
  let chain_hash = hash_str(text);
  let observation = { mol: mol, hash: chain_hash, text: text };

  // 3. Run instincts
  let instinct = run_instincts(observation, context);
  if instinct.action == "silence" {
    return { ok: true, response: "", action: "silence" };
  }

  // 4. Push to STM
  stm_push(context.stm, chain_hash, mol, emotion);

  // 5. Co-activate Silk (words that appear together)
  co_activate_text(context.silk, text, emotion);

  // 6. Walk emotion through Silk (amplification — NOT averaging)
  let amplified = walk_emotion(context.silk, chain_hash, emotion, 3);

  // 7. Update ConversationCurve
  curve_push(context.curve, amplified);

  // 8. Get response tone
  let tone = curve_tone(context.curve);

  return {
    ok: true,
    emotion: amplified,
    tone: tone,
    instinct: instinct,
    action: "learned"
  };
}

// ═══ Text → Molecule (packed u16) ═══
// [S:4][R:4][V:3][A:3][T:2] = 16 bits

fn text_to_mol(text, emotion) {
  let s = 1;  // default Sphere
  if ends_with(text, "?") { s = 5; }  // Circle for questions
  if starts_with(text, "!") || starts_with(text, "○{") { s = 4; }  // Triangle for commands

  let r = 1;  // default Member
  let v = clamp(round((emotion.v + 1.0) / 2.0 * 7), 0, 7);
  let a = clamp(round(emotion.a * 7), 0, 7);

  let t = 1;  // Medium
  if len(text) < 10 { t = 2; }   // Short = Fast
  if len(text) > 100 { t = 0; }  // Long = Slow

  return mol_new(s, r, v, a, t);
}

fn co_activate_text(silk, text, emotion) {
  let words = split_words(text);
  let n = len(words);
  if n < 2 { return; }

  // Adjacent pairs
  let i = 0;
  while i < n - 1 {
    let hash_a = hash_str(words[i]);
    let hash_b = hash_str(words[i + 1]);
    co_activate(silk, hash_a, hash_b, emotion);
    i = i + 1;
  }

  // Proximity decay window (φ⁻¹ = 0.618)
  i = 0;
  while i < n {
    let j = i + 2;
    while j < n && j < i + 5 {
      let decay = 0.618 / (j - i);
      let hash_a = hash_str(words[i]);
      let hash_b = hash_str(words[j]);
      let scaled_emotion = {
        v: emotion.v * decay,
        a: emotion.a * decay,
        d: emotion.d,
        i: emotion.i * decay
      };
      co_activate(silk, hash_a, hash_b, scaled_emotion);
      j = j + 1;
    }
    i = i + 1;
  }
}

// ═══ ORIGIN.OLANG STM — 32 slots with eviction scoring ═══
// (from origin.olang learning.ol G7)

let __ls_text = __array_with_cap(64);
let __ls_mol = __array_with_cap(64);
let __ls_v = __array_with_cap(64);
let __ls_a = __array_with_cap(64);
let __ls_access = __array_with_cap(64);
let __stm_max = 32;
let __stm_turn = [0];

pub fn kt_stm_push(_text) {
  let _mol = _kt_real_mol(_text);
  let _v = mol_get_dim(_mol, 2);
  let _a = mol_get_dim(_mol, 3);
  if len(__ls_text) >= __stm_max {
    // Evict lowest score: recency*0.3 + emotion*0.4 + frequency*0.3
    let _min_score = [999999];
    let _min_idx = [0];
    let _i = 0;
    while _i < len(__ls_text) {
      let _turns_ago = __array_get(__stm_turn, 0) - _i;
      let _recency = 1000;
      if _turns_ago > 0 { _recency = __floor(618000 / (1000 + (_turns_ago * 382))); }
      let _emo = _kt_abs(__array_get(__ls_v, _i) - 4) * __array_get(__ls_a, _i);
      let _score = (__array_get(__ls_access, _i) * 300) + (_emo * 400) + (_recency * 300);
      _score = __floor(_score / 1000);
      if _score < __array_get(_min_score, 0) {
        let _ = __set_at(_min_score, 0, _score);
        let _ = __set_at(_min_idx, 0, _i);
      }
      _i = _i + 1;
    }
    let _evict = __array_get(_min_idx, 0);
    let _ = __set_at(__ls_text, _evict, _text);
    let _ = __set_at(__ls_mol, _evict, _mol);
    let _ = __set_at(__ls_v, _evict, _v);
    let _ = __set_at(__ls_a, _evict, _a);
    let _ = __set_at(__ls_access, _evict, 1);
  } else {
    push(__ls_text, _text);
    push(__ls_mol, _mol);
    push(__ls_v, _v);
    push(__ls_a, _a);
    push(__ls_access, 1);
  }
  let _ = __set_at(__stm_turn, 0, __array_get(__stm_turn, 0) + 1);
  // Auto silk fire with recent entry
  let _n = len(__ls_mol);
  if _n >= 2 { kt_silk_fire(_mol, __array_get(__ls_mol, _n - 2)); }
}

pub fn kt_stm_count() { return len(__ls_text); }
pub fn kt_stm_mol_at(_i) { return __array_get(__ls_mol, _i); }
pub fn kt_stm_text_at(_i) { return __array_get(__ls_text, _i); }

// ═══ G7: Dream — cross-group consolidation ═══

let __dream_count = [0];

pub fn dream() {
  let _n = kt_stm_count();
  if _n < 2 { return; }
  let _i = 0;
  while _i < _n {
    let _j = _i + 1;
    while _j < _n {
      let _mi = kt_stm_mol_at(_i);
      let _mj = kt_stm_mol_at(_j);
      // Different S or R bucket = cross-group → strengthen silk
      let _si = (__floor(_mi / 4096)) % 16;
      let _sj = (__floor(_mj / 4096)) % 16;
      let _ri = (__floor(_mi / 256)) % 16;
      let _rj = (__floor(_mj / 256)) % 16;
      if _si != _sj { kt_silk_fire(_mi, _mj); }
      if _ri != _rj { kt_silk_fire(_mi, _mj); }
      _j = _j + 1;
    }
    _i = _i + 1;
  }
  // LCA: compose cross-group pairs → new concept
  let _new_concepts = [0];
  let _i2 = 0;
  while _i2 < _n {
    let _j2 = _i2 + 1;
    while _j2 < _n {
      let _mi2 = kt_stm_mol_at(_i2);
      let _mj2 = kt_stm_mol_at(_j2);
      if _mi2 != _mj2 {
        let _lca = compose([_mi2, _mj2]);
        let _text_i = kt_stm_text_at(_i2);
        let _text_j = kt_stm_text_at(_j2);
        if len(_text_i) > 0 {
          if len(_text_j) > 0 {
            let _concept = _text_i + " + " + _text_j;
            kt_learn(_concept);
            let _ = __set_at(_new_concepts, 0, __array_get(_new_concepts, 0) + 1);
          }
        }
      }
      _j2 = _j2 + 1;
    }
    _i2 = _i2 + 1;
  }
  // Decay all silk weights
  kt_silk_decay();
  let _ = __set_at(__dream_count, 0, __array_get(__dream_count, 0) + 1);
}

// ═══ G16: Chain Recombination — SINH nội dung mới ═══

pub fn generate(_query) {
  let _mol = _kt_real_mol(_query);
  let _path = kt_silk_walk(_mol, 3, 50);
  if len(_path) < 2 {
    let _n1 = kt_nearest(_mol);
    if len(_n1) > 0 { return _n1; }
    return "";
  }
  return decode_path(_path);
}

// ═══ G12: Immune Selection — 3 branches ═══

pub fn immune_select(_mol) {
  let _dim0 = mol_dominant_dim(_mol);
  let _p0 = kt_silk_walk(_mol, 3, 100);
  let _p1 = kt_silk_walk(_mol, 3, 100);
  let _text = kt_find("", 1);
  if len(_p0) >= len(_p1) { return _p0; }
  return _p1;
}

// ═══ G4: Decode — path → text ═══

pub fn decode_path(_path) {
  let _out = "";
  let _i = 0;
  while _i < len(_path) {
    let _mol = __array_get(_path, _i);
    let _text = kt_nearest(_mol);
    if len(_text) > 0 {
      if len(_out) > 0 { _out = _out + ". "; }
      _out = _out + _text;
    }
    _i = _i + 1;
  }
  return _out;
}

// ═══ QR — append-only proven knowledge ═══

let _qr_facts = [];
let __qr_proven = [];
let __dn_fire = [];

pub fn dn_observe(fact) {
  push(_qr_facts, fact);
  kt_learn(fact);
  kt_stm_push(fact);
  return "DN (fire=" + __to_string(len(_qr_facts)) + ")";
}

pub fn dn_count() { return len(_qr_facts); }
pub fn qr_count() { return len(__qr_proven); }

pub fn learning_status() {
  return "DN:" + __to_string(len(_qr_facts))
       + " QR:" + __to_string(len(__qr_proven))
       + " STM:" + __to_string(kt_stm_count())
       + " Dream:" + __to_string(__array_get(__dream_count, 0));
}

// ═══ G14: Negative Knowledge — prohibited space ═══

let __nac_prohibited = [];

pub fn nac_mark(_text) {
  push(__nac_prohibited, _kt_real_mol(_text));
}

pub fn nac_check(_mol) {
  let _i = 0;
  while _i < len(__nac_prohibited) {
    if _kt_mol_dist(_mol, __array_get(__nac_prohibited, _i)) < 3 { return 1; }
    _i = _i + 1;
  }
  return 0;
}

// ═══ G14: QR promotion — fire count → Fibonacci threshold ═══

pub fn dn_fire_check() {
  let _i = 0;
  while _i < len(_qr_facts) {
    if _i < len(__dn_fire) {
      let _fc = __array_get(__dn_fire, _i);
      // Fibonacci threshold: 2,3,5,8,13...
      if _fc >= 5 {
        push(__qr_proven, __array_get(_qr_facts, _i));
        let _ = __set_at(__dn_fire, _i, 0);
      }
    }
    _i = _i + 1;
  }
}

// ═══ G19: Persistence — save/load ═══

pub fn kt_save_state(_path) {
  let _out = "";
  let _i = 0;
  while _i < len(__kt_facts) {
    _out = _out + __array_get(__kt_facts, _i) + "\n";
    _i = _i + 1;
  }
  __file_write(_path, _out);
  return "Saved " + __to_string(len(__kt_facts)) + " facts to " + _path;
}

pub fn kt_load_state(_path) {
  let _c = __file_read(_path);
  if len(_c) == 0 { return "empty"; }
  let _count = [0];
  let _start = [0];
  let _i = 0;
  while _i < len(_c) {
    if __char_code(char_at(_c, _i)) == 10 {
      let _line = substr(_c, __array_get(_start, 0), _i);
      if len(_line) > 3 {
        kt_learn(_line);
        let _ = __set_at(_count, 0, __array_get(_count, 0) + 1);
      }
      let _ = __set_at(_start, 0, _i + 1);
    }
    _i = _i + 1;
  }
  __heap_pin();
  return "Loaded " + __to_string(__array_get(_count, 0)) + " facts from " + _path;
}

// ═══ G22: Goal System — self-directed learning ═══

let __goals = [];

pub fn goal_add(_domain, _priority) {
  push(__goals, _domain);
  push(__goals, _priority);
}

pub fn goal_top() {
  if len(__goals) < 2 { return ""; }
  let _best = [""]; let _bp = [0];
  let _i = 0;
  while _i < len(__goals) {
    let _p = __array_get(__goals, _i + 1);
    if _p > __array_get(_bp, 0) {
      let _ = __set_at(_bp, 0, _p);
      let _ = __set_at(_best, 0, __array_get(__goals, _i));
    }
    _i = _i + 2;
  }
  return __array_get(_best, 0);
}

pub fn goal_count() { return __floor(len(__goals) / 2); }

// ═══ Context factory (Origin Rust) ═══

pub fn context_new() {
  return {
    stm: stm_new(),
    silk: silk_new(),
    curve: curve_new(),
  };
}

pub fn context_dream(context) {
  return dream_cycle(context.stm, context.silk);
}

// ═══ Utility ═══

fn split_words(text) {
  let words = [];
  let current = "";
  let i = 0;
  let n = len(text);
  while i < n {
    let ch = char_at(text, i);
    if ch == " " || ch == "\n" || ch == "\t" {
      if len(current) > 0 {
        push(words, current);
        current = "";
      }
    } else {
      current = current + ch;
    }
    i = i + 1;
  }
  if len(current) > 0 {
    push(words, current);
  }
  return words;
}

fn starts_with(s, prefix) {
  if len(s) < len(prefix) { return false; }
  return substr(s, 0, len(prefix)) == prefix;
}

fn ends_with(s, suffix) {
  if len(s) < len(suffix) { return false; }
  return substr(s, len(s) - len(suffix), len(suffix)) == suffix;
}
