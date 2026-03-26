// stdlib/knowtree.ol — KnowTree: 5-Branch Knowledge Tree for HomeOS
// Structure: KnowTree[5 branches][max 65536 each]
// Branch layout follows human knowledge hierarchy:
//   0: Quy luat Goc    (Khoa hoc Tu nhien)
//   1: Co May Con Nguoi (Khoa hoc Suc khoe & Tam tri)
//   2: Song Chung       (Khoa hoc Xa hoi)
//   3: Cong Cu & Sang Tao (Ky thuat & Nghe thuat)
//   4: Cau Hoi Lon      (Triet hoc & Tam linh)
//
// Each branch has sub-categories (sub_id).
// Each entry: { key, mol(u16), sub_id }
// Lookup: branch_id → sub_id → entries

// ── Branch IDs ─────────────────────────────────────────────
let BR_NATURE   = 0;  // Khoa hoc Tu nhien
let BR_HUMAN    = 1;  // Suc khoe & Tam tri
let BR_SOCIAL   = 2;  // Khoa hoc Xa hoi
let BR_CRAFT    = 3;  // Ky thuat & Nghe thuat
let BR_BEYOND   = 4;  // Triet hoc & Tam linh

// ── Sub-category IDs per branch ────────────────────────────
// Branch 0: Quy luat Goc
let SUB_MATH    = 0;  // Toan hoc
let SUB_PHYSICS = 1;  // Vat ly
let SUB_CHEM    = 2;  // Hoa hoc
let SUB_BIO     = 3;  // Sinh hoc

// Branch 1: Co May Con Nguoi
let SUB_MED     = 0;  // Y sinh
let SUB_PSY     = 1;  // Tam ly hoc
let SUB_NEURO   = 2;  // Than kinh hoc

// Branch 2: Song Chung
let SUB_LANG    = 0;  // Ngon ngu hoc
let SUB_HIST    = 1;  // Lich su & Khao co
let SUB_ECON    = 2;  // Kinh te & Chinh tri
let SUB_LAW     = 3;  // Luat phap & Dao duc

// Branch 3: Cong Cu & Sang Tao
let SUB_TECH    = 0;  // Ky thuat & Cong nghe
let SUB_ARCH    = 1;  // Kien truc
let SUB_ART     = 2;  // Nghe thuat (Hoi hoa, Am nhac, Van hoc)

// Branch 4: Cau Hoi Lon
let SUB_PHIL    = 0;  // Triet hoc
let SUB_THEO    = 1;  // Than hoc
let SUB_COSMO   = 2;  // Vu tru hoc

// ── Create empty branch ────────────────────────────────────
pub fn branch_new(id, name) {
  let keys = [];
  let mols = [];
  let sub_ids = [];
  return { id: id, name: name, keys: keys, mols: mols, sub_ids: sub_ids, count: 0 };
}

// ── Create KnowTree with 5 branches ───────────────────────
pub fn knowtree_new() {
  let branches = [];
  __push(branches, branch_new(0, "Quy luat Goc"));
  __push(branches, branch_new(1, "Co May Con Nguoi"));
  __push(branches, branch_new(2, "Song Chung"));
  __push(branches, branch_new(3, "Cong Cu & Sang Tao"));
  __push(branches, branch_new(4, "Cau Hoi Lon"));
  return { branches: branches, total: 0 };
}

// ── Insert entry into branch ───────────────────────────────
pub fn knowtree_insert(tree, branch_id, sub_id, key, mol) {
  let br = __array_get(tree.branches, branch_id);
  __push(br.keys, key);
  __push(br.mols, mol);
  __push(br.sub_ids, sub_id);
  let _ = __set_at(br, "count", br.count + 1);
  let _ = __set_at(tree, "total", tree.total + 1);
  return tree;
}

// ── Get branch by ID ──────────────────────────────────────
pub fn knowtree_branch(tree, branch_id) {
  return __array_get(tree.branches, branch_id);
}

// ── Count entries in a branch ──────────────────────────────
pub fn knowtree_branch_count(tree, branch_id) {
  let br = __array_get(tree.branches, branch_id);
  return br.count;
}

// ── Search by key within a branch ──────────────────────────
// Returns index or -1
pub fn branch_find(tree, branch_id, key) {
  let br = __array_get(tree.branches, branch_id);
  let i = 0;
  while i < br.count {
    if __array_get(br.keys, i) == key { return i; }
    i = i + 1;
  }
  return -1;
}

// ── Search by key across ALL branches ──────────────────────
// Returns { branch_id, index } or { branch_id: -1, index: -1 }
pub fn knowtree_find(tree, key) {
  let b = 0;
  while b < 5 {
    let idx = branch_find(tree, b, key);
    if idx >= 0 { return { branch_id: b, index: idx }; }
    b = b + 1;
  }
  return { branch_id: -1, index: -1 };
}

// ── Get mol at branch + index ──────────────────────────────
pub fn knowtree_get_mol(tree, branch_id, index) {
  let br = __array_get(tree.branches, branch_id);
  return __array_get(br.mols, index);
}

// ── Get sub_id at branch + index ───────────────────────────
pub fn knowtree_get_sub(tree, branch_id, index) {
  let br = __array_get(tree.branches, branch_id);
  return __array_get(br.sub_ids, index);
}

// ── List all entries in a sub-category ─────────────────────
// Returns array of indices matching sub_id within branch
pub fn branch_list_sub(tree, branch_id, sub_id) {
  let br = __array_get(tree.branches, branch_id);
  let result = [];
  let i = 0;
  while i < br.count {
    if __array_get(br.sub_ids, i) == sub_id {
      __push(result, i);
    }
    i = i + 1;
  }
  return result;
}

// ── Search by mol similarity within branch ─────────────────
// Returns array of { index, key } where mol matches exactly
pub fn branch_find_mol(tree, branch_id, target_mol) {
  let br = __array_get(tree.branches, branch_id);
  let result = [];
  let i = 0;
  while i < br.count {
    if __array_get(br.mols, i) == target_mol {
      __push(result, { index: i, key: __array_get(br.keys, i) });
    }
    i = i + 1;
  }
  return result;
}

// ── Branch name lookup ─────────────────────────────────────
pub fn branch_name(tree, branch_id) {
  let br = __array_get(tree.branches, branch_id);
  return br.name;
}

// ── Sub-category name (hardcoded for clarity) ──────────────
pub fn sub_name(branch_id, sub_id) {
  // Branch 0: Nature
  if branch_id == 0 {
    if sub_id == 0 { return "Toan hoc"; }
    if sub_id == 1 { return "Vat ly"; }
    if sub_id == 2 { return "Hoa hoc"; }
    if sub_id == 3 { return "Sinh hoc"; }
  }
  // Branch 1: Human
  if branch_id == 1 {
    if sub_id == 0 { return "Y sinh"; }
    if sub_id == 1 { return "Tam ly hoc"; }
    if sub_id == 2 { return "Than kinh hoc"; }
  }
  // Branch 2: Social
  if branch_id == 2 {
    if sub_id == 0 { return "Ngon ngu hoc"; }
    if sub_id == 1 { return "Lich su & Khao co"; }
    if sub_id == 2 { return "Kinh te & Chinh tri"; }
    if sub_id == 3 { return "Luat phap & Dao duc"; }
  }
  // Branch 3: Craft
  if branch_id == 3 {
    if sub_id == 0 { return "Ky thuat & Cong nghe"; }
    if sub_id == 1 { return "Kien truc"; }
    if sub_id == 2 { return "Nghe thuat"; }
  }
  // Branch 4: Beyond
  if branch_id == 4 {
    if sub_id == 0 { return "Triet hoc"; }
    if sub_id == 1 { return "Than hoc"; }
    if sub_id == 2 { return "Vu tru hoc"; }
  }
  return "Unknown";
}

// ── Summary: print tree stats ──────────────────────────────
pub fn knowtree_summary(tree) {
  let b = 0;
  while b < 5 {
    let br = __array_get(tree.branches, b);
    emit br.name + ": " + to_string(br.count) + " entries";
    b = b + 1;
  }
  emit "Total: " + to_string(tree.total);
  return tree.total;
}
