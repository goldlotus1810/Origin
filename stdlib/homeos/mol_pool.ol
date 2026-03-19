// stdlib/homeos/mol_pool.ol — Molecule slab allocator
// PLAN 5.3.3: Fixed-size pool for 5-byte Molecule objects.
// Molecule = 5 bytes + 3 bytes padding = 8 bytes per slot.
// Slab = 4096 slots = 32 KB.
// Alloc/Free = O(1) via free list.

let SLOT_SIZE = 8;      // 5 bytes molecule + 3 bytes metadata
let SLAB_SLOTS = 4096;  // slots per slab
let SLAB_SIZE = 32768;  // SLOT_SIZE * SLAB_SLOTS

pub fn pool_new() {
  // Initialize pool with one slab
  let pool = {
    slots: [],
    free_head: 0,
    allocated: 0,
    total_slots: SLAB_SLOTS,
    slab_count: 1
  };
  // Initialize free list: each slot points to next
  let i = 0;
  while i < SLAB_SLOTS {
    push(pool.slots, { mol: 0, next_free: i + 1, in_use: false });
    i = i + 1;
  }
  // Last slot's next_free = -1 (end of list)
  pool.slots[SLAB_SLOTS - 1].next_free = -1;
  return pool;
}

pub fn pool_alloc(pool) {
  // O(1): pop from free list head
  if pool.free_head == -1 {
    // Pool exhausted — grow
    pool_grow(pool);
  }
  let slot_idx = pool.free_head;
  let slot = pool.slots[slot_idx];
  pool.free_head = slot.next_free;
  slot.in_use = true;
  slot.next_free = -1;
  pool.allocated = pool.allocated + 1;
  return slot_idx;
}

pub fn pool_free(pool, slot_idx) {
  // O(1): push to free list head
  if slot_idx < 0 || slot_idx >= pool.total_slots { return; }
  let slot = pool.slots[slot_idx];
  if !slot.in_use { return; }  // double-free guard
  slot.in_use = false;
  slot.mol = 0;
  slot.next_free = pool.free_head;
  pool.free_head = slot_idx;
  pool.allocated = pool.allocated - 1;
}

pub fn pool_get(pool, slot_idx) {
  // Get molecule data from slot
  if slot_idx < 0 || slot_idx >= pool.total_slots { return 0; }
  return pool.slots[slot_idx].mol;
}

pub fn pool_set(pool, slot_idx, mol) {
  // Store molecule data in slot
  if slot_idx < 0 || slot_idx >= pool.total_slots { return; }
  pool.slots[slot_idx].mol = mol;
}

fn pool_grow(pool) {
  // Add another slab (double capacity)
  let old_total = pool.total_slots;
  let i = 0;
  while i < SLAB_SLOTS {
    push(pool.slots, { mol: 0, next_free: old_total + i + 1, in_use: false });
    i = i + 1;
  }
  pool.slots[old_total + SLAB_SLOTS - 1].next_free = -1;
  pool.free_head = old_total;
  pool.total_slots = old_total + SLAB_SLOTS;
  pool.slab_count = pool.slab_count + 1;
}

pub fn pool_stats(pool) {
  return {
    allocated: pool.allocated,
    free: pool.total_slots - pool.allocated,
    total_slots: pool.total_slots,
    slab_count: pool.slab_count,
    memory_kb: pool.total_slots * SLOT_SIZE / 1024
  };
}
