//! # vm — OlangVM Stack Machine
//!
//! Execute OlangProgram.
//! Stack-based: mọi thứ là MolecularChain.
//!
//! VM không có side effects trực tiếp — trả về Vec<VmEvent>
//! để caller (HomeRuntime) xử lý (ghi registry, trigger dream...).

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ir::{OlangProgram, Op};
use crate::lca::lca;
use crate::molecular::{Molecule, MolecularChain};

// ─────────────────────────────────────────────────────────────────────────────
// VmEvent — side effects VM muốn thực hiện
// ─────────────────────────────────────────────────────────────────────────────

/// Zero-allocation string ordering comparison.
/// Compares lower 8 bits (byte values) of each molecule lexicographically.
#[inline]
fn chain_cmp_bytes(a: &MolecularChain, b: &MolecularChain) -> core::cmp::Ordering {
    for (ma, mb) in a.0.iter().zip(b.0.iter()) {
        let ba = (ma & 0xFF) as u8;
        let bb = (mb & 0xFF) as u8;
        match ba.cmp(&bb) {
            core::cmp::Ordering::Equal => continue,
            ord => return ord,
        }
    }
    a.0.len().cmp(&b.0.len())
}

/// Fast check: is this chain a string-encoded chain? (shape=2, rel=1 marker)
/// String molecules have top byte = 0x21 (shape=2 in bits[15:12], rel=1 in bits[11:8]).
/// Must check ALL molecules to reject mixed chains (string+separator arrays).
#[inline]
fn is_string_chain(chain: &MolecularChain) -> bool {
    !chain.is_empty() && chain.0.iter().all(|&bits| bits & 0xFF00 == 0x2100)
}

/// Extract readable text from a string-encoded MolecularChain.
/// String chains use shape=0x02, relation=0x01, with each byte stored in valence.
/// Returns None if the chain doesn't look like a string encoding.
pub fn chain_to_string(chain: &MolecularChain) -> Option<String> {
    if chain.is_empty() {
        return Some(String::new());
    }
    // Check if it looks like a string chain: shape=2, relation=1 (quantized 4-bit values)
    let is_string = chain.0.iter().all(|&bits| {
        let m = Molecule::from_u16(bits);
        m.shape() == 2 && m.relation() == 1
    });
    if is_string {
        // Decode bytes from lower 8 bits [V:3][A:3][T:2] of each molecule
        let bytes: Vec<u8> = chain.0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
        match String::from_utf8(bytes) {
            Ok(s) => Some(s),
            Err(e) => {
                // Lossy fallback for invalid UTF-8 sequences
                Some(String::from_utf8_lossy(e.as_bytes()).into_owned())
            }
        }
    } else {
        None
    }
}

/// Encode a string as a MolecularChain (each byte → 1 molecule).
/// Inverse of chain_to_string.
///
/// String molecule marker: shape=2, relation=1 (quantized 4-bit values).
/// Valence holds the byte value (quantized 3-bit, so only 0-7 range for marker check).
/// The actual byte is stored in the full V+A+T bits (11 bits = 0-2047, enough for u8 0-255).
/// Encode a single byte as a string molecule u16.
///
/// Format: `[S=2:4bit][R=1:4bit][V:3bit][A:3bit][T:2bit]` where lower 8 bits = byte value.
/// Used by all string builtins for consistent encoding (bypass Molecule::raw quantization).
#[inline]
fn str_byte_mol(b: u8) -> u16 {
    let v3 = ((b >> 5) & 0x7) as u16;
    let a3 = ((b >> 2) & 0x7) as u16;
    let t2 = (b & 0x3) as u16;
    (2u16 << 12) | (1u16 << 8) | (v3 << 5) | (a3 << 2) | t2
}

/// Encode a string as a MolecularChain (each byte → 1 molecule).
/// Inverse of chain_to_string.
///
/// String molecule marker: shape=2, relation=1 (quantized 4-bit values).
/// The actual byte is stored in the lower 8 bits `[V:3][A:3][T:2]`.
pub fn string_to_chain(s: &str) -> MolecularChain {
    let mols: Vec<u16> = s.bytes().map(|b| str_byte_mol(b)).collect();
    MolecularChain(mols)
}

// Keep original implementation as reference (unused):
#[cfg(any())]
fn _string_to_chain_original(s: &str) -> MolecularChain {
    let mols: Vec<u16> = s.bytes().map(|b| {
        let s4: u16 = 2;
        let r4: u16 = 1;
        let v3 = ((b >> 5) & 0x7) as u16;
        let a3 = ((b >> 2) & 0x7) as u16;
        let t2 = (b & 0x3) as u16;
        (s4 << 12) | (r4 << 8) | (v3 << 5) | (a3 << 2) | t2
    }).collect();
    MolecularChain(mols)
}

/// Format a chain for human-readable display.
/// Tries string decoding first, then number, then raw molecule info.
pub fn format_chain_display(chain: &MolecularChain) -> String {
    if chain.is_empty() {
        return "(empty)".into();
    }
    // Try string
    if let Some(s) = chain_to_string(chain) {
        return s;
    }
    // Try number
    if let Some(n) = chain.to_number() {
        return if n == (n as i64 as f64) {
            alloc::format!("{}", n as i64)
        } else {
            alloc::format!("{}", n)
        };
    }
    // Fallback: molecule count + hash
    alloc::format!("[chain: {} molecules, hash={:#x}]", chain.len(), chain.chain_hash())
}

/// Event từ VM → caller xử lý.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum VmEvent {
    /// Output chain (từ EMIT)
    Output(MolecularChain),
    /// Cần lookup alias trong Registry
    LookupAlias(String),
    /// Cần tạo Silk edge
    CreateEdge { from: u64, to: u64, rel: u8 },
    /// Cần query relation
    QueryRelation { hash: u64, rel: u8 },
    /// Trigger Dream cycle
    TriggerDream,
    /// Request system stats
    RequestStats,
    /// Error trong VM
    Error(VmError),
    // ── Reasoning & Debug ────────────────────────────────────────────────
    /// Execution trace step (opcode name, stack depth, pc)
    TraceStep {
        op_name: &'static str,
        stack_depth: usize,
        pc: usize,
    },
    /// Inspect chain structure (molecules count, hash, byte size)
    InspectChain {
        hash: u64,
        molecule_count: usize,
        byte_size: usize,
        is_empty: bool,
    },
    /// Assert failed — chain was empty
    AssertFailed,
    /// Type classification of a chain's molecules
    TypeInfo {
        hash: u64,
        classification: String,
    },
    /// Why: explain connection between two chains
    WhyConnection { from: u64, to: u64 },
    /// Explain: trace origin of a chain
    ExplainOrigin { hash: u64 },

    // ── Device I/O events ────────────────────────────────────────────────────
    // VM emit — Runtime xử lý → gọi HAL → phần cứng thật.

    /// Ghi giá trị ra thiết bị. Runtime gọi HAL.device_write().
    DeviceWrite {
        /// Device ID (VD: "gpio_relay", "light_0")
        device_id: String,
        /// Giá trị ghi (molecular dimension: 0x00=off, 0xFF=max)
        value: u8,
    },
    /// Đọc giá trị từ thiết bị. Runtime gọi HAL.device_read().
    DeviceRead {
        /// Device ID
        device_id: String,
    },
    /// Liệt kê thiết bị. Runtime gọi HAL.scan_devices().
    DeviceListRequest,

    // ── FFI & System I/O events ──────────────────────────────────────────────

    /// Gọi foreign function. Runtime dispatch → extern fn → inject result.
    FfiCall {
        /// Function name (VD: "gpio_write", "http_get")
        name: String,
        /// Arguments (popped from stack before event)
        args: Vec<MolecularChain>,
    },
    /// Đọc file. Runtime gọi HAL.read_file().
    FileReadRequest {
        /// File path (extracted from chain)
        path: String,
    },
    /// Ghi file. Runtime gọi HAL.write_file().
    FileWriteRequest {
        /// File path
        path: String,
        /// Data to write
        data: Vec<u8>,
    },
    /// Append file. Runtime gọi HAL.write_file() ở chế độ append.
    FileAppendRequest {
        /// File path
        path: String,
        /// Data to append
        data: Vec<u8>,
    },
    /// Spawn request. Runtime tạo ISL message hoặc async task.
    SpawnRequest {
        /// Number of opcodes in the spawned block
        body_ops_count: usize,
    },
    /// Module import request. Runtime loads + executes the module file.
    UseModule {
        /// Module name or path
        name: String,
    },
    /// Selective module import: load specific symbols from a module.
    UseModuleSelective {
        /// Module name or path
        name: String,
        /// Specific symbols to import
        imports: Vec<String>,
    },
    /// Module declaration.
    ModDecl {
        /// Module path (dot-separated)
        path: String,
    },
    /// Early return from ? operator (Err/None propagation).
    /// VM should execute Ret to return from current function.
    EarlyReturn,
}

// ─────────────────────────────────────────────────────────────────────────────
// VmError
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum VmError {
    StackUnderflow,
    StackOverflow,
    InfiniteLoop,
    InvalidJump(usize),
    MaxStepsExceeded,
    MaxCallDepthExceeded,
    DivisionByZero,
    /// Runtime error with custom message (panic, assert failures, etc.)
    RuntimeError(String),
    /// Phase 6G: Molecular constraint violation at function call site
    ConstraintViolation(String),
}

impl core::fmt::Display for VmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::StackUnderflow => write!(f, "Stack underflow — pop from empty stack"),
            Self::StackOverflow => write!(f, "Stack overflow — exceeded {} entries", STACK_MAX),
            Self::InfiniteLoop => write!(f, "Infinite loop detected (QT2: ∞ is wrong)"),
            Self::InvalidJump(target) => write!(f, "Invalid jump to position {}", target),
            Self::MaxStepsExceeded => write!(f, "Max steps exceeded ({}) — program too long", STEPS_MAX),
            Self::MaxCallDepthExceeded => write!(f, "Max call depth exceeded — too many nested scopes"),
            Self::DivisionByZero => write!(f, "Division by zero"),
            Self::RuntimeError(msg) => write!(f, "{}", msg),
            Self::ConstraintViolation(msg) => write!(f, "Constraint violation: {}", msg),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// VmStack
// ─────────────────────────────────────────────────────────────────────────────

const STACK_MAX: usize = 256;
const STEPS_MAX: u32 = 500_000;

/// Pop 1 chain từ stack, break nếu underflow.
macro_rules! vm_pop {
    ($stack:expr, $events:expr) => {
        match $stack.pop() {
            Ok(c) => c,
            Err(e) => {
                $events.push(VmEvent::Error(e));
                break;
            }
        }
    };
}

struct VmStack {
    data: Vec<MolecularChain>,
}

impl VmStack {
    fn new() -> Self {
        Self {
            data: Vec::with_capacity(32),
        }
    }

    fn push(&mut self, c: MolecularChain) -> Result<(), VmError> {
        if self.data.len() >= STACK_MAX {
            return Err(VmError::StackOverflow);
        }
        self.data.push(c);
        Ok(())
    }

    fn pop(&mut self) -> Result<MolecularChain, VmError> {
        self.data.pop().ok_or(VmError::StackUnderflow)
    }

    fn peek(&self) -> Option<&MolecularChain> {
        self.data.last()
    }

    fn _is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// OlangVM
// ─────────────────────────────────────────────────────────────────────────────

/// Classify chain by dominant molecule characteristics.
///
/// Simple glob pattern matching: * = any chars, ? = any single char.
fn glob_match(text: &str, pattern: &str) -> bool {
    let (t, p) = (text.as_bytes(), pattern.as_bytes());
    let (mut ti, mut pi) = (0usize, 0usize);
    let (mut star_pi, mut star_ti) = (usize::MAX, 0usize);
    while ti < t.len() {
        if pi < p.len() && (p[pi] == b'?' || p[pi] == t[ti]) {
            ti += 1; pi += 1;
        } else if pi < p.len() && p[pi] == b'*' {
            star_pi = pi; star_ti = ti; pi += 1;
        } else if star_pi != usize::MAX {
            pi = star_pi + 1; star_ti += 1; ti = star_ti;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == b'*' { pi += 1; }
    pi == p.len()
}

/// Iterator-level separator molecule — distinct from array element separator AND closure markers.
/// Uses raw u16 0xFE01 to avoid collision with closure marker tag (0xF000) and array sep (0xF000).
fn iter_sep() -> Molecule {
    Molecule::from_u16(0xFE01)
}

/// Split an iterator-encoded MolecularChain by iterator separator molecules (shape=0xFE).
/// This preserves array-element separators (shape=0) within each section intact.
fn split_iter_chain(chain: &MolecularChain) -> Vec<MolecularChain> {
    if chain.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut current = Vec::new();
    let sep_bits = iter_sep().bits;
    for &bits in &chain.0 {
        if bits == sep_bits {
            result.push(MolecularChain(core::mem::take(&mut current)));
        } else {
            current.push(bits);
        }
    }
    result.push(MolecularChain(current));
    result
}

/// Maps ShapeBase to Unicode group categories:
/// Split an array-encoded MolecularChain by separator molecules (shape=0, relation=0).
/// Split array chain by separator molecules (shape=0, relation=0, v=0, a=0, t=0).
// Heap reference markers — direct u16 bit packing (bypass Molecule::raw quantization)
// Format: [marker:4bit][unused:4bit][index:8bit] = 16 bits
const HEAP_DICT_MARKER: u16 = 0xFC00;   // shape nibble = 0xF, relation nibble = 0xC
const HEAP_ARRAY_MARKER: u16 = 0xFD00;  // shape nibble = 0xF, relation nibble = 0xD
const HEAP_MARKER_MASK: u16 = 0xFF00;   // top 8 bits = marker
const HEAP_INDEX_MASK: u16 = 0x00FF;    // bottom 8 bits = index (0..255)

/// Create a heap-reference chain for a dict.
fn make_dict_ref(idx: usize) -> MolecularChain {
    MolecularChain::single(Molecule::from_u16(HEAP_DICT_MARKER | (idx as u16 & HEAP_INDEX_MASK)))
}

/// Create a heap-reference chain for an array.
fn make_array_ref(idx: usize) -> MolecularChain {
    MolecularChain::single(Molecule::from_u16(HEAP_ARRAY_MARKER | (idx as u16 & HEAP_INDEX_MASK)))
}

/// Check if chain is a dict heap reference.
fn as_dict_ref(chain: &MolecularChain) -> Option<usize> {
    if chain.0.len() == 1 && (chain.0[0] & HEAP_MARKER_MASK) == HEAP_DICT_MARKER {
        return Some((chain.0[0] & HEAP_INDEX_MASK) as usize);
    }
    None
}

/// Check if chain is an array heap reference.
fn as_array_ref(chain: &MolecularChain) -> Option<usize> {
    if chain.0.len() == 1 && (chain.0[0] & HEAP_MARKER_MASK) == HEAP_ARRAY_MARKER {
        return Some((chain.0[0] & HEAP_INDEX_MASK) as usize);
    }
    None
}

/// Materialize a heap ref into a flat chain for external consumption (e.g. Emit).
/// Recursively materializes nested heap refs.
fn materialize_heap_value(
    chain: &MolecularChain,
    array_heap: &[Vec<MolecularChain>],
    dict_heap: &[Vec<(MolecularChain, MolecularChain)>],
) -> MolecularChain {
    if let Some(arr_idx) = as_array_ref(chain) {
        if arr_idx < array_heap.len() {
            let sep = array_sep();
            let mut result = MolecularChain(Vec::new());
            for (i, elem) in array_heap[arr_idx].iter().enumerate() {
                if i > 0 { result.0.push(sep.bits); }
                let mat = materialize_heap_value(elem, array_heap, dict_heap);
                result.0.extend(mat.0.iter().copied());
            }
            return result;
        }
    }
    if let Some(dict_idx) = as_dict_ref(chain) {
        if dict_idx < dict_heap.len() {
            let sep = Molecule::raw(0, 0, 0, 0, 0);
            let mut result = MolecularChain(Vec::new());
            for (i, (k, v)) in dict_heap[dict_idx].iter().enumerate() {
                if i > 0 { result.0.push(sep.bits); }
                let mk = materialize_heap_value(k, array_heap, dict_heap);
                result.0.extend(mk.0.iter().copied());
                result.0.push(sep.bits);
                let mv = materialize_heap_value(v, array_heap, dict_heap);
                result.0.extend(mv.0.iter().copied());
            }
            return result;
        }
    }
    chain.clone()
}

/// Check if a molecule is a null separator (used by dicts: key|null|val|null|key|null|val).
fn is_null_separator(bits: &u16) -> bool {
    let mol = Molecule::from_u16(*bits);
    mol.shape_u8() == 0 && mol.relation_u8() == 0
        && mol.valence_u8() == 0 && mol.arousal_u8() == 0
        && mol.time_u8() == 0
}

/// Array-element separator molecule (shape_u8 = 0xF0 after quantization of 0xFE).
/// Distinct from null separator (all zeros) and iter separator (same bits, but used
/// at a different level). We match on the quantized value that `Molecule::raw(0xFE, 0, 0, 0, 0)`
/// actually produces.
fn array_sep() -> Molecule {
    Molecule::raw(0xFE, 0, 0, 0, 0)
}

/// Check if a molecule is an array element separator.
/// `Molecule::raw(0xFE, 0, 0, 0, 0)` quantizes shape 0xFE → 0x0F (4 bits),
/// so `shape_u8()` = 0xF0.  We check the raw bits directly for speed.
fn is_array_separator(bits: &u16) -> bool {
    *bits == array_sep().bits
}

/// Split a chain by null separators (used for dicts AND legacy arrays without 0xFE separators).
pub fn split_array_chain(chain: &MolecularChain) -> Vec<MolecularChain> {
    if chain.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut current = Vec::new();
    // Skip the 0xFD tag molecule used by __array_push to track element count
    let start = if !chain.0.is_empty() && Molecule::from_u16(chain.0[0]).shape_u8() == 0xFD { 1 } else { 0 };
    // Check if this array uses 0xFE array separators (modern format)
    let has_array_seps = chain.0[start..].iter().any(|m| is_array_separator(m));
    for mol in &chain.0[start..] {
        if has_array_seps {
            // Modern array: split only on 0xFE, keep internal null separators
            if is_array_separator(mol) {
                result.push(MolecularChain(core::mem::take(&mut current)));
            } else {
                current.push(*mol);
            }
        } else {
            // Legacy: split on null separators
            if is_null_separator(mol) {
                result.push(MolecularChain(core::mem::take(&mut current)));
            } else {
                current.push(*mol);
            }
        }
    }
    // Last element (no trailing separator)
    result.push(MolecularChain(current));
    result
}

/// Split an enum-tagged chain (Option/Result) into [tag, payload].
/// The tag is always a string chain (all molecules have bits & 0xFF00 == 0x2100).
/// After the tag there is one null separator, then the payload (arbitrary encoding).
/// Unlike split_array_chain, this finds the FIRST null separator that immediately
/// follows a string molecule, so it won't be confused by 0x0000 inside numeric payloads.
fn split_enum_parts(chain: &MolecularChain) -> Vec<MolecularChain> {
    if chain.is_empty() {
        return Vec::new();
    }
    // Find the boundary: last string molecule followed by null separator
    let mut boundary = None;
    for i in 0..chain.0.len() {
        if chain.0[i] == 0x0000 {
            // Check that everything before this is a string molecule
            if i > 0 && chain.0[..i].iter().all(|&b| b & 0xFF00 == 0x2100) {
                boundary = Some(i);
                break;
            }
        }
    }
    if let Some(sep_idx) = boundary {
        let tag = MolecularChain(chain.0[..sep_idx].to_vec());
        let payload = MolecularChain(chain.0[sep_idx + 1..].to_vec());
        alloc::vec![tag, payload]
    } else {
        // No separator found — return whole chain as single element (e.g. Option::None with no payload)
        alloc::vec![chain.clone()]
    }
}

/// Split a chain by null separators only (for dict key-value extraction).
/// This always splits on null (0,0,0,0,0) regardless of array separators.
fn split_dict_chain(chain: &MolecularChain) -> Vec<MolecularChain> {
    if chain.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut current = Vec::new();
    for mol in &chain.0 {
        if is_null_separator(mol) {
            result.push(MolecularChain(core::mem::take(&mut current)));
        } else {
            current.push(*mol);
        }
    }
    result.push(MolecularChain(current));
    result
}

/// Get a field value from a dict chain. Keys at even indices, values at odd.
fn get_dict_field(dict: &MolecularChain, key: &MolecularChain) -> MolecularChain {
    let elements = split_dict_chain(dict);
    let mut i = 0;
    while i + 1 < elements.len() {
        if elements[i].0 == key.0 {
            return elements[i + 1].clone();
        }
        i += 2;
    }
    MolecularChain::empty()
}

/// - SDF shapes (Sphere●, Capsule▬, Box■, Cone▲) → geometric primitives
/// - CSG ops (Torus○, Union∪, Intersect∩, Subtract∖) → mathematical composition
/// - High emotion valence → emoticon-like
///
/// Returns "SDF", "EMOTICON", or "Mixed(SDF+EMOTICON)".
fn classify_chain(chain: &MolecularChain) -> String {
    if chain.is_empty() {
        return "Empty".into();
    }
    let (mut sdf, mut emo) = (0u32, 0u32);
    for &bits in &chain.0 {
        let mol = Molecule::from_u16(bits);
        // v2: all 18 ShapeBase variants are SDF primitives
        let _shape = mol.shape_base();
        sdf += 1;
        // Extreme valence → emoticon category
        let v = mol.valence_u8();
        if !(80..=176).contains(&v) {
            emo += 1;
        }
    }
    let total = chain.len() as u32;
    // v2: all shapes are SDF primitives, MATH category no longer applies
    let dominant = [("SDF", sdf), ("EMOTICON", emo)];
    let mut parts: Vec<&str> = dominant
        .iter()
        .filter(|(_, c)| *c * 2 >= total) // ≥50% of molecules
        .map(|(name, _)| *name)
        .collect();
    if parts.is_empty() {
        parts.push("Mixed");
    }
    if parts.len() == 1 {
        parts[0].into()
    } else {
        let mut s = String::from("Mixed(");
        for (i, p) in parts.iter().enumerate() {
            if i > 0 {
                s.push('+');
            }
            s.push_str(p);
        }
        s.push(')');
        s
    }
}

/// Create a closure marker MolecularChain that stores `body_pc` losslessly.
///
/// Layout: `[tag_molecule, pc_low_u16, pc_high_u16]`
/// - tag_molecule has shape=0xFF (quantized to 0x0F via `shape()`)
/// - body_pc is split into two raw u16 words so it survives round-trip
///   without the lossy 3-bit quantization of valence/arousal fields.
fn make_closure_marker(param_count: u8, body_pc: usize) -> MolecularChain {
    let tag = Molecule::raw(0xFF, param_count, 0, 0, 1);
    let mut chain = MolecularChain::single(tag);
    chain.push_raw(body_pc as u16);
    chain.push_raw((body_pc >> 16) as u16);
    chain
}

/// Extract body_pc from a closure marker created by [`make_closure_marker`].
/// Returns `None` if the chain is not a valid closure marker.
fn closure_body_pc(chain: &MolecularChain) -> Option<usize> {
    if chain.0.len() >= 3 {
        let mol = Molecule::from_u16(chain.0[0]);
        if mol.shape() == (0xFF >> 4) {
            let lo = chain.0[1] as usize;
            let hi = chain.0[2] as usize;
            return Some(lo | (hi << 16));
        }
    }
    None
}

/// Execute a closure inline with given arguments.
/// Used by higher-order array methods (map, filter, fold, etc.).
/// Returns the result left on stack after closure body executes.
fn call_closure_inline(
    prog: &OlangProgram,
    body_pc: usize,
    args: &[MolecularChain],
    parent_scopes: &[Vec<(String, MolecularChain)>],
    steps: &mut u32,
    max_steps: u32,
) -> MolecularChain {
    let mut local_stack = VmStack::new();
    // Push args (reversed so body can Store them in order)
    for arg in args.iter().rev() {
        let _ = local_stack.push(arg.clone());
    }
    // Create scope with parent's variables visible
    let mut scopes: Vec<Vec<(String, MolecularChain)>> = Vec::new();
    // Copy parent scope for variable access (capture by value)
    if let Some(last) = parent_scopes.last() {
        scopes.push(last.clone());
    } else {
        scopes.push(Vec::new());
    }
    // New scope for closure locals
    scopes.push(Vec::new());

    let mut local_pc = body_pc;
    let mut local_events = Vec::new();
    while local_pc < prog.ops.len() && *steps < max_steps {
        let op = &prog.ops[local_pc];
        local_pc += 1;
        *steps += 1;
        if matches!(op, Op::Ret) {
            break;
        }
        match op {
            Op::Store(name) => {
                let val = local_stack.pop().unwrap_or_default();
                if let Some(scope) = scopes.last_mut() {
                    if let Some(entry) = scope.iter_mut().find(|(n, _)| n == name) {
                        entry.1 = val;
                    } else {
                        scope.push((name.clone(), val));
                    }
                }
            }
            Op::LoadLocal(name) => {
                let val = scopes.iter().rev()
                    .find_map(|s| s.iter().rev().find(|(n, _)| n == name).map(|(_, c)| c.clone()))
                    .unwrap_or_else(MolecularChain::empty);
                let _ = local_stack.push(val);
            }
            Op::PushNum(n) => { let _ = local_stack.push(MolecularChain::from_number(*n)); }
            Op::Push(chain) => { let _ = local_stack.push(chain.clone()); }
            Op::Call(fname) => {
                match fname.as_str() {
                    "__hyp_add" | "__hyp_sub" | "__hyp_mul" | "__hyp_div"
                    | "__hyp_mod" | "__phys_add" | "__phys_sub" => {
                        let b = local_stack.pop().unwrap_or_default();
                        let a = local_stack.pop().unwrap_or_default();
                        let fa = a.to_number().unwrap_or(0.0);
                        let fb = b.to_number().unwrap_or(0.0);
                        let result = match fname.as_str() {
                            "__hyp_add" | "__phys_add" => fa + fb,
                            "__hyp_sub" | "__phys_sub" => fa - fb,
                            "__hyp_mul" => fa * fb,
                            "__hyp_div" => if fb.abs() > f64::EPSILON { fa / fb } else { 0.0 },
                            "__hyp_mod" => if fb.abs() > f64::EPSILON { fa % fb } else { 0.0 },
                            _ => 0.0,
                        };
                        let _ = local_stack.push(MolecularChain::from_number(result));
                    }
                    "__cmp_lt" | "__cmp_gt" | "__cmp_le" | "__cmp_ge" | "__cmp_ne" => {
                        let b = local_stack.pop().unwrap_or_default();
                        let a = local_stack.pop().unwrap_or_default();
                        let result = if is_string_chain(&a) || is_string_chain(&b) {
                            let ord = chain_cmp_bytes(&a, &b);
                            match fname.as_str() {
                                "__cmp_lt" => ord == core::cmp::Ordering::Less,
                                "__cmp_gt" => ord == core::cmp::Ordering::Greater,
                                "__cmp_le" => ord != core::cmp::Ordering::Greater,
                                "__cmp_ge" => ord != core::cmp::Ordering::Less,
                                "__cmp_ne" => ord != core::cmp::Ordering::Equal,
                                _ => false,
                            }
                        } else if let (Some(fa), Some(fb)) =
                            (a.to_number(), b.to_number())
                        {
                            match fname.as_str() {
                                "__cmp_lt" => fa < fb,
                                "__cmp_gt" => fa > fb,
                                "__cmp_le" => fa <= fb,
                                "__cmp_ge" => fa >= fb,
                                "__cmp_ne" => (fa - fb).abs() >= f64::EPSILON,
                                _ => false,
                            }
                        } else {
                            match fname.as_str() {
                                "__cmp_ne" => a != b,
                                _ => false,
                            }
                        };
                        let _ = local_stack.push(if result {
                            MolecularChain::from_number(1.0)
                        } else {
                            MolecularChain::empty()
                        });
                    }
                    _ => {
                        local_events.push(VmEvent::LookupAlias(fname.clone()));
                    }
                }
            }
            Op::Lca => {
                let b = local_stack.pop().unwrap_or_default();
                let a = local_stack.pop().unwrap_or_default();
                let _ = local_stack.push(lca(&a, &b));
            }
            Op::Pop => { let _ = local_stack.pop(); }
            Op::Dup => {
                if let Some(top) = local_stack.peek() {
                    let _ = local_stack.push(top.clone());
                }
            }
            Op::Swap => {
                let b = if let Ok(v) = local_stack.pop() { v } else { break };
                let a = if let Ok(v) = local_stack.pop() { v } else { break };
                let _ = local_stack.push(b);
                let _ = local_stack.push(a);
            }
            Op::Emit => {
                // In closure context, emit means "this is the return value"
                // Don't actually emit; leave on stack
            }
            _ => {}
        }
    }
    // Return top of stack
    local_stack.pop().unwrap_or_else(|_| MolecularChain::empty())
}

/// OlangVM — stack machine thực thi OlangProgram.
pub struct OlangVM {
    /// Max steps để tránh infinite loop (QT2: ∞-1)
    pub max_steps: u32,
    /// Max call depth để tránh stack overflow từ recursion
    pub max_call_depth: u32,
}

#[allow(missing_docs)]
impl OlangVM {
    pub fn new() -> Self {
        Self {
            max_steps: STEPS_MAX,
            max_call_depth: 512, // Bootstrap parser needs deep scope nesting
        }
    }

    pub fn with_max_steps(n: u32) -> Self {
        Self {
            max_steps: n,
            max_call_depth: 512,
        }
    }

    /// Execute program → Vec<VmEvent>.
    ///
    /// VM không access registry trực tiếp.
    /// LOAD → emit LookupAlias event → caller inject chain.
    /// Sau đó caller gọi resume_with(chain) để tiếp tục.
    pub fn execute(&self, prog: &OlangProgram) -> VmResult {
        let mut stack = VmStack::new();
        // Scope stack: each Vec is a scope frame with local variables.
        // ScopeBegin pushes new frame, ScopeEnd pops it.
        // LoadLocal searches from innermost scope outward.
        let mut scopes: Vec<Vec<(alloc::string::String, MolecularChain)>> = Vec::new();
        scopes.push(Vec::new()); // root scope
        let mut events = Vec::new();
        let mut steps = 0u32;
        let mut pc = 0usize;
        let mut call_depth = 0u32;
        // Call stack for CallClosure: (saved_pc, scope_depth, stack_depth, param_count)
        let mut closure_call_stack: Vec<(usize, usize, usize, usize)> = Vec::new();
        // Loop stack: (jump_back_pc, remaining_iterations)
        let mut loop_stack: Vec<(usize, u32)> = Vec::new();
        // Try/catch stack: catch handler PC targets
        let mut try_stack: Vec<usize> = Vec::new();
        // Channel store: id → queue of messages (cooperative concurrency)
        let mut channels: Vec<Vec<MolecularChain>> = Vec::new();
        let mut next_channel_id: u64 = 1;
        // Heap for dict/array objects (avoids in-band separator nesting issues)
        // Dict: Vec<(key_chain, value_chain)>
        // Array: Vec<element_chain>
        let mut dict_heap: Vec<Vec<(MolecularChain, MolecularChain)>> = Vec::new();
        let mut array_heap: Vec<Vec<MolecularChain>> = Vec::new();
        // Compiler pipeline caches (for __parse → __lower → __encode_bytecode)
        let mut parse_cache: Option<Vec<crate::lang::syntax::Stmt>> = None;
        let mut lower_cache: Option<crate::exec::ir::OlangProgram> = None;

        while pc < prog.ops.len() {
            // Batch step check: only compare max_steps every 256 iterations
            steps += 1;
            if steps & 0xFF == 0 && steps >= self.max_steps {
                // If in try block, jump to catch instead of halting
                if let Some(catch_pc) = try_stack.pop() {
                    pc = catch_pc;
                    continue;
                }
                events.push(VmEvent::Error(VmError::MaxStepsExceeded));
                break;
            }

            let op = &prog.ops[pc];
            pc += 1;

            match op {
                Op::Nop => {}

                Op::Push(chain) => {
                    if let Err(e) = stack.push(chain.clone()) {
                        events.push(VmEvent::Error(e));
                        break;
                    }
                }

                Op::PushNum(n) => {
                    if let Err(e) = stack.push(MolecularChain::from_number(*n)) {
                        events.push(VmEvent::Error(e));
                        break;
                    }
                }

                Op::PushMol(bits) => {
                    // v2: push packed u16 as 1-link chain.
                    // Bytecode: [0x19][lo][hi] = 3 bytes.
                    let chain = MolecularChain(alloc::vec![*bits]);
                    if let Err(e) = stack.push(chain) {
                        events.push(VmEvent::Error(e));
                        break;
                    }
                }

                Op::Load(alias) => {
                    // Emit event — caller sẽ inject chain
                    events.push(VmEvent::LookupAlias(alias.clone()));
                    // Push empty placeholder — real impl dùng callback
                    let _ = stack.push(MolecularChain::empty());
                }

                Op::Lca => {
                    let b = vm_pop!(stack, events);
                    let a = vm_pop!(stack, events);
                    let result = if a.is_empty() || b.is_empty() {
                        if !a.is_empty() {
                            a
                        } else {
                            b
                        }
                    } else {
                        lca(&a, &b)
                    };
                    if let Err(e) = stack.push(result) {
                        events.push(VmEvent::Error(e));
                        break;
                    }
                }

                Op::Edge(rel) => {
                    let b = vm_pop!(stack, events);
                    let a = vm_pop!(stack, events);
                    events.push(VmEvent::CreateEdge {
                        from: a.chain_hash(),
                        to: b.chain_hash(),
                        rel: *rel,
                    });
                    // Giữ lại b trên stack (kết quả của relation)
                    let _ = stack.push(b);
                }

                Op::Query(rel) => {
                    let a = vm_pop!(stack, events);
                    events.push(VmEvent::QueryRelation {
                        hash: a.chain_hash(),
                        rel: *rel,
                    });
                    // Push empty — caller sẽ inject results
                    let _ = stack.push(MolecularChain::empty());
                }

                Op::Emit => {
                    let c = vm_pop!(stack, events);
                    // Materialize heap refs before emitting
                    let materialized = materialize_heap_value(&c, &array_heap, &dict_heap);
                    events.push(VmEvent::Output(materialized));
                }

                Op::Dup => {
                    let c = match stack.peek() {
                        Some(c) => c.clone(),
                        None => {
                            events.push(VmEvent::Error(VmError::StackUnderflow));
                            break;
                        }
                    };
                    let _ = stack.push(c);
                }

                Op::Pop => {
                    let _ = vm_pop!(stack, events);
                }

                Op::Swap => {
                    let b = vm_pop!(stack, events);
                    let a = vm_pop!(stack, events);
                    let _ = stack.push(b);
                    let _ = stack.push(a);
                }

                Op::Jmp(target) => {
                    if *target >= prog.ops.len() {
                        events.push(VmEvent::Error(VmError::InvalidJump(*target)));
                        break;
                    }
                    pc = *target;
                }

                Op::Jz(target) => {
                    let is_empty = stack.peek().map(|c| c.is_empty()).unwrap_or(true);
                    if is_empty {
                        if *target >= prog.ops.len() {
                            events.push(VmEvent::Error(VmError::InvalidJump(*target)));
                            break;
                        }
                        pc = *target;
                    }
                }

                Op::Loop(n) => {
                    // Loop(n): repeat next instruction block n times.
                    // The block runs from pc (current, after Loop opcode) to the next
                    // ScopeEnd or Halt. Uses loop_stack to track remaining iterations
                    // and the jump-back target.
                    // Max iterations capped at 1024 (QT2: ∞-1, không vô hạn).
                    let count = (*n).min(1024);
                    if count > 1 {
                        loop_stack.push((pc, count - 1)); // (jump_back_to, remaining)
                    }
                    // First iteration starts immediately (fall through)
                }

                Op::Call(name) => {
                    // Dispatch built-in functions first, otherwise emit lookup
                    match name.as_str() {
                        "__hyp_add" | "__hyp_sub" | "__hyp_mul" | "__hyp_div"
                        | "__phys_add" | "__phys_sub" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let nb = b.to_number().unwrap_or(0.0);
                            let result = match name.as_str() {
                                "__hyp_add" | "__phys_add" => na + nb,
                                "__hyp_sub" | "__phys_sub" => na - nb,
                                "__hyp_mul" => na * nb,
                                "__hyp_div" => {
                                    if nb.abs() < f64::EPSILON {
                                        events.push(VmEvent::Error(VmError::DivisionByZero));
                                        break;
                                    }
                                    na / nb
                                }
                                _ => 0.0,
                            };
                            let _ = stack.push(MolecularChain::from_number(result));
                        }
                        "__cmp_lt" | "__cmp_gt" | "__cmp_le" | "__cmp_ge" | "__cmp_ne" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            // String-first: check encoding BEFORE to_number() to prevent
                            // 4-char strings from being miscompared as denormalized f64.
                            let truthy = if is_string_chain(&a) || is_string_chain(&b) {
                                // Zero-alloc string comparison via raw u16 byte values
                                let ord = chain_cmp_bytes(&a, &b);
                                match name.as_str() {
                                    "__cmp_lt" => ord == core::cmp::Ordering::Less,
                                    "__cmp_gt" => ord == core::cmp::Ordering::Greater,
                                    "__cmp_le" => ord != core::cmp::Ordering::Greater,
                                    "__cmp_ge" => ord != core::cmp::Ordering::Less,
                                    "__cmp_ne" => ord != core::cmp::Ordering::Equal,
                                    _ => false,
                                }
                            } else if let (Some(na), Some(nb)) =
                                (a.to_number(), b.to_number())
                            {
                                match name.as_str() {
                                    "__cmp_lt" => na < nb,
                                    "__cmp_gt" => na > nb,
                                    "__cmp_le" => na <= nb,
                                    "__cmp_ge" => na >= nb,
                                    "__cmp_ne" => (na - nb).abs() >= f64::EPSILON,
                                    _ => false,
                                }
                            } else {
                                match name.as_str() {
                                    "__cmp_ne" => a != b,
                                    _ => false,
                                }
                            };
                            // true → non-empty chain (1.0), false → empty chain
                            // Jz checks is_empty() so empty = falsy
                            if truthy {
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__logic_not" => {
                            let a = vm_pop!(stack, events);
                            // Invert: empty → 1.0 (truthy), non-empty → empty (falsy)
                            if a.is_empty() {
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__assert_truth" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            // Truth: string-first check to avoid 4-char string/f64 collision
                            let is_true = if is_string_chain(&a) || is_string_chain(&b) {
                                a == b
                            } else if let (Some(na), Some(nb)) =
                                (a.to_number(), b.to_number())
                            {
                                (na - nb).abs() < f64::EPSILON
                            } else {
                                a == b
                            };
                            if is_true {
                                let _ = stack.push(a); // push back (truthy)
                            } else {
                                let _ = stack.push(MolecularChain::empty()); // falsy
                            }
                        }
                        "__match_type" => {
                            // Pop expected type name (from Load) and actual type chain
                            // The actual chain was DUPed then TypeOf → events emitted
                            // Stack has: [... type_info_chain, expected_name_chain]
                            let _expected = vm_pop!(stack, events);
                            let actual = vm_pop!(stack, events);
                            // classify_chain returns "SDF", "MATH", "EMOTICON", "Mixed(...)", "Empty"
                            let type_name = classify_chain(&actual);
                            // The expected pattern comes from Load("SDF") which triggers
                            // LookupAlias. For matching, we check if the type_name
                            // starts with common known types. The pattern name is stored
                            // in the Load op preceding this Call.
                            // Since the VM is stack-based and we already consumed the
                            // expected chain, we look at the previous Load op's name
                            // from the events (LookupAlias).
                            let expected_name = events.iter().rev().find_map(|e| {
                                if let VmEvent::LookupAlias(n) = e { Some(n.clone()) } else { None }
                            }).unwrap_or_default();
                            let matches = type_name.starts_with(&expected_name)
                                || (expected_name == "Mixed" && type_name.starts_with("Mixed"));
                            if matches {
                                let _ = stack.push(actual); // truthy
                            } else {
                                let _ = stack.push(MolecularChain::empty()); // falsy
                            }
                        }
                        "__match_enum" => {
                            // Match user-defined enum variant by tag string
                            // Stack: [subject, expected_tag_chain]
                            // Expected tag = "EnumName::Variant" string chain
                            let expected_tag = vm_pop!(stack, events);
                            let subject = vm_pop!(stack, events);
                            let expected_str = chain_to_string(&expected_tag).unwrap_or_default();

                            // Extract tag from subject: everything before first null separator
                            let mut tag_mols = Vec::new();
                            for &bits in &subject.0 {
                                let mol = Molecule::from_u16(bits);
                                if mol.shape() == 0 && mol.relation() == 0
                                    && mol.valence_u8() == 0 && mol.arousal_u8() == 0
                                    && mol.time() == 0
                                {
                                    break; // stop at first separator
                                }
                                tag_mols.push(bits);
                            }
                            let actual_tag = chain_to_string(&MolecularChain(tag_mols)).unwrap_or_default();

                            // Also check __type field for struct-tagged dicts
                            let matches = if actual_tag == expected_str {
                                true
                            } else if let Some(dict_idx) = as_dict_ref(&subject) {
                                // Heap-based dict: check __type field
                                let type_key = string_to_chain("__type");
                                if dict_idx < dict_heap.len() {
                                    dict_heap[dict_idx].iter()
                                        .find(|(k, _)| k.0 == type_key.0)
                                        .map(|(_, v)| chain_to_string(v).unwrap_or_default() == expected_str)
                                        .unwrap_or(false)
                                } else {
                                    false
                                }
                            } else {
                                // Legacy flat chain: check __type field
                                let type_key = string_to_chain("__type");
                                let type_val = get_dict_field(&subject, &type_key);
                                if type_val.is_empty() {
                                    false
                                } else {
                                    chain_to_string(&type_val).unwrap_or_default() == expected_str
                                }
                            };

                            if matches {
                                let _ = stack.push(subject); // truthy — keep subject for binding extraction
                            } else {
                                let _ = stack.push(MolecularChain::empty()); // falsy
                            }
                        }
                        "__enum_field" => {
                            // Extract nth payload field from enum variant
                            // Stack: [enum_chain, index_num]
                            let index_chain = vm_pop!(stack, events);
                            let enum_chain = vm_pop!(stack, events);
                            let index = index_chain.to_number().unwrap_or(0.0) as usize;

                            if let Some(dict_idx) = as_dict_ref(&enum_chain) {
                                // Heap dict (StructLiteral): skip __type, return nth other field's value
                                let type_key = string_to_chain("__type");
                                if dict_idx < dict_heap.len() {
                                    let mut field_i = 0usize;
                                    let mut found = false;
                                    for (k, v) in &dict_heap[dict_idx] {
                                        if k.0 == type_key.0 { continue; } // skip __type
                                        if field_i == index {
                                            let _ = stack.push(v.clone());
                                            found = true;
                                            break;
                                        }
                                        field_i += 1;
                                    }
                                    if !found {
                                        let _ = stack.push(MolecularChain::empty());
                                    }
                                } else {
                                    let _ = stack.push(MolecularChain::empty());
                                }
                            } else {
                                // Legacy flat chain: [tag][sep][field0][sep][field1]...
                                let parts = split_array_chain(&enum_chain);
                                let field_idx = index + 1; // +1 to skip tag
                                if field_idx < parts.len() {
                                    let _ = stack.push(parts[field_idx].clone());
                                } else {
                                    let _ = stack.push(MolecularChain::empty());
                                }
                            }
                        }
                        "__match_mol" => {
                            // Pop expected mol pattern and subject
                            let expected = vm_pop!(stack, events);
                            let actual = vm_pop!(stack, events);
                            // Compare molecule dimensions
                            let matches = if !actual.is_empty() && !expected.is_empty() {
                                let a = Molecule::from_u16(actual.0[0]);
                                let e = Molecule::from_u16(expected.0[0]);
                                a.shape() == e.shape()
                                    && a.relation() == e.relation()
                                    && a.valence_u8() == e.valence_u8()
                                    && a.arousal_u8() == e.arousal_u8()
                                    && a.time() == e.time()
                            } else {
                                actual.is_empty() && expected.is_empty()
                            };
                            if matches {
                                let _ = stack.push(actual); // truthy
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        // Phase 6: Molecular constraint pattern matching
                        // Stack: [subject, constraint0, constraint1, ..., constraintN-1, count]
                        // Each constraint = PushMol(dim_byte, op_byte, value, 0, 0)
                        // dim_byte: S=1 R=2 V=3 A=4 T=5
                        // op_byte: Eq=0 Gt=1 Lt=2 Ge=3 Le=4 Any=5
                        "__match_mol_constraint" => {
                            let count_chain = vm_pop!(stack, events);
                            let count = count_chain.to_number().unwrap_or(0.0) as usize;
                            // Pop constraint molecules
                            let mut constraints = Vec::new();
                            for _ in 0..count {
                                constraints.push(vm_pop!(stack, events));
                            }
                            constraints.reverse(); // order: first constraint first
                            // Pop subject
                            let actual = vm_pop!(stack, events);
                            let matches = if actual.is_empty() {
                                false
                            } else {
                                let mol = Molecule::from_u16(actual.0[0]);
                                constraints.iter().all(|c| {
                                    if c.is_empty() { return true; }
                                    let cm = Molecule::from_u16(c.0[0]);
                                    let dim_val = match cm.shape() {
                                        1 => mol.shape(),
                                        2 => mol.relation(),
                                        3 => mol.valence_u8(),
                                        4 => mol.arousal_u8(),
                                        5 => mol.time(),
                                        _ => return true,
                                    };
                                    let threshold = cm.valence_u8(); // value stored in V position
                                    match cm.relation() { // op stored in R position
                                        0 => dim_val == threshold,     // Eq
                                        1 => dim_val > threshold,      // Gt
                                        2 => dim_val < threshold,      // Lt
                                        3 => dim_val >= threshold,     // Ge
                                        4 => dim_val <= threshold,     // Le
                                        5 => true,                     // Any
                                        _ => true,
                                    }
                                })
                            };
                            if matches {
                                let _ = stack.push(actual); // truthy
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }

                        // Phase 6G: Runtime constraint check for function parameters
                        // Stack: [value, constraint0, ..., constraintN-1, count]
                        // Same encoding as __match_mol_constraint
                        // If constraint fails → emit VmError
                        "__check_constraint" => {
                            let count_chain = vm_pop!(stack, events);
                            let count = count_chain.to_number().unwrap_or(0.0) as usize;
                            let mut constraints = Vec::new();
                            for _ in 0..count {
                                constraints.push(vm_pop!(stack, events));
                            }
                            constraints.reverse();
                            let actual = vm_pop!(stack, events);
                            if !actual.is_empty() {
                                let mol = Molecule::from_u16(actual.0[0]);
                                for c in &constraints {
                                    if c.is_empty() { continue; }
                                    let cm = Molecule::from_u16(c.0[0]);
                                    let dim_name = match cm.shape() {
                                        1 => "S", 2 => "R", 3 => "V", 4 => "A", 5 => "T", _ => "?",
                                    };
                                    let dim_val = match cm.shape() {
                                        1 => mol.shape(), 2 => mol.relation(),
                                        3 => mol.valence_u8(), 4 => mol.arousal_u8(),
                                        5 => mol.time(), _ => continue,
                                    };
                                    let threshold = cm.valence_u8();
                                    let ok = match cm.relation() {
                                        0 => dim_val == threshold,
                                        1 => dim_val > threshold,
                                        2 => dim_val < threshold,
                                        3 => dim_val >= threshold,
                                        4 => dim_val <= threshold,
                                        5 => true,
                                        _ => true,
                                    };
                                    if !ok {
                                        let op_str = match cm.relation() {
                                            0 => "=", 1 => ">", 2 => "<", 3 => ">=", 4 => "<=", _ => "?",
                                        };
                                        events.push(VmEvent::Error(VmError::ConstraintViolation(
                                            alloc::format!("Constraint failed: {}={} but expected {}{}{}", dim_name, dim_val, dim_name, op_str, threshold)
                                        )));
                                    }
                                }
                            }
                            let _ = stack.push(actual); // pass through
                        }

                        "__array_new" => {
                            // Stack: [... elem0, elem1, ..., elemN-1, count]
                            let count_chain = vm_pop!(stack, events);
                            let count = count_chain.to_number().unwrap_or(0.0) as usize;
                            let mut elements = Vec::new();
                            for _ in 0..count {
                                elements.push(vm_pop!(stack, events));
                            }
                            elements.reverse();
                            // Store on array heap, push reference
                            let idx = array_heap.len();
                            array_heap.push(elements);
                            let _ = stack.push(make_array_ref(idx));
                        }
                        "__array_get" => {
                            // Stack: [array, index]
                            let idx_chain = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let idx = idx_chain.to_number().unwrap_or(0.0) as usize;
                            if let Some(arr_idx) = as_array_ref(&arr) {
                                // Heap-based array
                                if arr_idx < array_heap.len() && idx < array_heap[arr_idx].len() {
                                    let _ = stack.push(array_heap[arr_idx][idx].clone());
                                } else {
                                    let _ = stack.push(MolecularChain::empty());
                                }
                            } else {
                                // Legacy flat chain array
                                let elements = split_array_chain(&arr);
                                if idx < elements.len() {
                                    let _ = stack.push(elements[idx].clone());
                                } else {
                                    let _ = stack.push(MolecularChain::empty());
                                }
                            }
                        }
                        "__array_len" => {
                            let arr = vm_pop!(stack, events);
                            if let Some(arr_idx) = as_array_ref(&arr) {
                                // Heap-based array
                                let count = if arr_idx < array_heap.len() { array_heap[arr_idx].len() } else { 0 };
                                let _ = stack.push(MolecularChain::from_number(count as f64));
                            } else if arr.is_empty() {
                                let _ = stack.push(MolecularChain::from_number(0.0));
                            } else if !arr.0.is_empty() && Molecule::from_u16(arr.0[0]).shape() == (0xFD >> 4) {
                                // Tagged array (from push): count is stored in tag molecule
                                let count = Molecule::from_u16(arr.0[0]).valence_u8() as f64;
                                let _ = stack.push(MolecularChain::from_number(count));
                            } else if is_string_chain(&arr)
                            {
                                // Pure string chain: length = number of characters
                                let _ = stack.push(MolecularChain::from_number(arr.0.len() as f64));
                            } else {
                                let count = split_array_chain(&arr).len();
                                let _ = stack.push(MolecularChain::from_number(count as f64));
                            }
                        }
                        "__concat" => {
                            // Concatenate two chains: pop b, pop a, push a+b
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let mut result = MolecularChain(Vec::new());
                            result.0.extend(a.0.iter().copied());
                            result.0.extend(b.0.iter().copied());
                            let _ = stack.push(result);
                        }
                        "__head" => {
                            // First molecule of chain
                            let chain = vm_pop!(stack, events);
                            if let Some(mol) = chain.0.first() {
                                let _ = stack.push(MolecularChain(alloc::vec![*mol]));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__tail" => {
                            // Chain without first molecule
                            let chain = vm_pop!(stack, events);
                            if chain.0.len() > 1 {
                                let _ = stack.push(MolecularChain(chain.0[1..].to_vec()));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__array_push" => {
                            // Stack: [array, element]
                            let elem = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            if let Some(arr_idx) = as_array_ref(&arr) {
                                // Heap-based array: push in place
                                if arr_idx < array_heap.len() {
                                    array_heap[arr_idx].push(elem);
                                }
                                let _ = stack.push(arr); // return same ref
                            } else if arr.is_empty() {
                                // First push: create new heap array
                                let idx = array_heap.len();
                                array_heap.push(alloc::vec![elem]);
                                let _ = stack.push(make_array_ref(idx));
                            } else {
                                // Legacy flat chain array: convert to heap
                                let mut elements = split_array_chain(&arr);
                                elements.push(elem);
                                let idx = array_heap.len();
                                array_heap.push(elements);
                                let _ = stack.push(make_array_ref(idx));
                            }
                        }
                        "__dict_new" => {
                            // Stack: [key0, val0, key1, val1, ..., count]
                            let count_chain = vm_pop!(stack, events);
                            let count = count_chain.to_number().unwrap_or(0.0) as usize;
                            let mut pairs = Vec::new();
                            for _ in 0..count {
                                let val = vm_pop!(stack, events);
                                let key = vm_pop!(stack, events);
                                pairs.push((key, val));
                            }
                            pairs.reverse();
                            // Store on dict heap, push reference
                            let idx = dict_heap.len();
                            dict_heap.push(pairs);
                            let _ = stack.push(make_dict_ref(idx));
                        }
                        "__dict_get" => {
                            // Stack: [dict, key]
                            let key = vm_pop!(stack, events);
                            let dict = vm_pop!(stack, events);
                            if let Some(idx) = as_dict_ref(&dict) {
                                // Heap-based dict lookup
                                let mut found = false;
                                if idx < dict_heap.len() {
                                    for (k, v) in &dict_heap[idx] {
                                        if k.0 == key.0 {
                                            let _ = stack.push(v.clone());
                                            found = true;
                                            break;
                                        }
                                    }
                                }
                                if !found {
                                    let _ = stack.push(MolecularChain::empty());
                                }
                            } else {
                                // Legacy flat chain dict
                                let elements = split_dict_chain(&dict);
                                let mut found = false;
                                let mut i = 0;
                                while i + 1 < elements.len() {
                                    if elements[i].0 == key.0 {
                                        let _ = stack.push(elements[i + 1].clone());
                                        found = true;
                                        break;
                                    }
                                    i += 2;
                                }
                                if !found {
                                    let _ = stack.push(MolecularChain::empty());
                                }
                            }
                        }
                        "__dict_keys" => {
                            // Stack: [dict]
                            // Returns array of keys
                            let dict = vm_pop!(stack, events);
                            if let Some(idx) = as_dict_ref(&dict) {
                                let keys: Vec<MolecularChain> = if idx < dict_heap.len() {
                                    dict_heap[idx].iter().map(|(k, _)| k.clone()).collect()
                                } else {
                                    Vec::new()
                                };
                                let arr_idx = array_heap.len();
                                array_heap.push(keys);
                                let _ = stack.push(make_array_ref(arr_idx));
                            } else {
                                // Legacy flat chain dict
                                let elements = split_dict_chain(&dict);
                                let sep = Molecule::raw(0xFE, 0, 0, 0, 0);
                                let mut result = MolecularChain(Vec::new());
                                let mut key_idx = 0;
                                let mut i = 0;
                                while i < elements.len() {
                                    if key_idx > 0 {
                                        result.0.push(sep.bits);
                                    }
                                    result.0.extend(elements[i].0.iter().cloned());
                                    key_idx += 1;
                                    i += 2;
                                }
                                let _ = stack.push(result);
                            }
                        }
                        "__dict_set" => {
                            // Stack: [dict, key, value]
                            let value = vm_pop!(stack, events);
                            let key = vm_pop!(stack, events);
                            let dict = vm_pop!(stack, events);
                            if let Some(idx) = as_dict_ref(&dict) {
                                // Heap-based dict: mutate in place
                                if idx < dict_heap.len() {
                                    let mut found = false;
                                    for (k, v) in &mut dict_heap[idx] {
                                        if k.0 == key.0 {
                                            *v = value.clone();
                                            found = true;
                                            break;
                                        }
                                    }
                                    if !found {
                                        dict_heap[idx].push((key, value));
                                    }
                                }
                                let _ = stack.push(dict); // return same ref
                            } else {
                                // Legacy flat chain dict
                                let mut elements = split_dict_chain(&dict);
                                let mut found = false;
                                let mut i = 0;
                                while i + 1 < elements.len() {
                                    if elements[i].0 == key.0 {
                                        elements[i + 1] = value.clone();
                                        found = true;
                                        break;
                                    }
                                    i += 2;
                                }
                                if !found {
                                    elements.push(key);
                                    elements.push(value);
                                }
                                let sep = Molecule::raw(0, 0, 0, 0, 0);
                                let mut result = MolecularChain(Vec::new());
                                for (j, elem) in elements.into_iter().enumerate() {
                                    if j > 0 {
                                        result.0.push(sep.bits);
                                    }
                                    result.0.extend(elem.0.iter().cloned());
                                }
                                let _ = stack.push(result);
                            }
                        }
                        "__str_len" => {
                            // String length: count molecules in chain
                            let s = vm_pop!(stack, events);
                            let _ = stack.push(MolecularChain::from_number(s.0.len() as f64));
                        }
                        "__str_concat" => {
                            // Concatenate two chains
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let mut result = a.0.clone();
                            result.extend(b.0.iter().cloned());
                            let _ = stack.push(MolecularChain(result));
                        }
                        "__to_string" => {
                            // Convert value to string chain
                            let val = vm_pop!(stack, events);
                            // If already a string chain, keep as-is
                            if is_string_chain(&val) {
                                let _ = stack.push(val);
                            } else {
                                // Number → string chain
                                let n = val.to_number().unwrap_or(0.0);
                                let s = if n == (n as i64 as f64) {
                                    alloc::format!("{}", n as i64)
                                } else {
                                    alloc::format!("{}", n)
                                };
                                let _ = stack.push(string_to_chain(&s));
                            }
                        }
                        "__to_number" => {
                            // String chain → number: decode molecules back to digits
                            let val = vm_pop!(stack, events);
                            // Try as number first
                            if let Some(n) = val.to_number() {
                                let _ = stack.push(MolecularChain::from_number(n));
                            } else {
                                // Decode string bytes from valence
                                let s: String = val.0.iter()
                                    .map(|&bits| (bits & 0xFF) as u8 as char)
                                    .collect();
                                if let Ok(n) = s.parse::<f64>() {
                                    let _ = stack.push(MolecularChain::from_number(n));
                                } else {
                                    let _ = stack.push(MolecularChain::from_number(0.0));
                                }
                            }
                        }
                        "__print" => {
                            // Print: emit top of stack as output (same as Emit but via call)
                            let val = vm_pop!(stack, events);
                            events.push(VmEvent::Output(val));
                        }
                        "__println" => {
                            // Print with newline: emit value + newline as string chain
                            let val = vm_pop!(stack, events);
                            let text = format_chain_display(&val);
                            let with_nl = alloc::format!("{}\n", text);
                            events.push(VmEvent::Output(string_to_chain(&with_nl)));
                        }
                        "__hyp_mod" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let nb = b.to_number().unwrap_or(1.0);
                            let _ = stack.push(MolecularChain::from_number(na % nb));
                        }
                        "__hyp_neg" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(-na));
                        }
                        "__hyp_abs" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(na.abs()));
                        }
                        "__hyp_min" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let nb = b.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(na.min(nb)));
                        }
                        "__hyp_max" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let nb = b.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(na.max(nb)));
                        }
                        "__array_set" => {
                            // Stack: [array, index, value]
                            let value = vm_pop!(stack, events);
                            let idx_chain = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let idx = idx_chain.to_number().unwrap_or(0.0) as usize;
                            if let Some(arr_idx) = as_array_ref(&arr) {
                                if arr_idx < array_heap.len() && idx < array_heap[arr_idx].len() {
                                    array_heap[arr_idx][idx] = value;
                                }
                                let _ = stack.push(arr);
                            } else {
                                let mut elements = split_array_chain(&arr);
                                if idx < elements.len() {
                                    elements[idx] = value;
                                }
                                let aidx = array_heap.len();
                                array_heap.push(elements);
                                let _ = stack.push(make_array_ref(aidx));
                            }
                        }
                        "__array_slice" => {
                            // Stack: [array, start, end]
                            let end_chain = vm_pop!(stack, events);
                            let start_chain = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let start = start_chain.to_number().unwrap_or(0.0) as usize;
                            let end = end_chain.to_number().unwrap_or(0.0) as usize;
                            let elements: Vec<MolecularChain> = if let Some(arr_idx) = as_array_ref(&arr) {
                                if arr_idx < array_heap.len() {
                                    array_heap[arr_idx].iter().skip(start).take(end.saturating_sub(start)).cloned().collect()
                                } else { Vec::new() }
                            } else {
                                split_array_chain(&arr).into_iter().skip(start).take(end.saturating_sub(start)).collect()
                            };
                            let aidx = array_heap.len();
                            array_heap.push(elements);
                            let _ = stack.push(make_array_ref(aidx));
                        }
                        "__is_empty" => {
                            let val = vm_pop!(stack, events);
                            let result = if val.is_empty() { 1.0 } else { 0.0 };
                            let _ = stack.push(MolecularChain::from_number(result));
                        }
                        "__eq" => {
                            // Equality: string-first, then numeric, then deep chain compare.
                            // IMPORTANT: Check string encoding BEFORE to_number() because
                            // 4-char strings have exactly 4 u16 molecules which to_number()
                            // interprets as f64 bits, causing false equality between
                            // different strings of the same length.
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let is_equal = if is_string_chain(&a) || is_string_chain(&b) {
                                a.0 == b.0
                            } else if let (Some(na), Some(nb)) =
                                (a.to_number(), b.to_number())
                            {
                                (na - nb).abs() < f64::EPSILON
                            } else {
                                a.0 == b.0
                            };
                            if is_equal {
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        // ── String builtins ────────────────────────────────
                        "__str_split" => {
                            // Stack: [string_chain, delimiter_chain]
                            // Split string by delimiter, return array of sub-strings
                            let delim = vm_pop!(stack, events);
                            let s = vm_pop!(stack, events);
                            // Decode both to byte strings via valence
                            let s_bytes: Vec<u8> = s.0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
                            let d_bytes: Vec<u8> = delim.0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
                            if d_bytes.is_empty() {
                                let _ = stack.push(s); // no split on empty delim
                            } else {
                                // Split
                                let sep = Molecule::raw(0, 0, 0, 0 , 0);
                                let mut result = MolecularChain(Vec::new());
                                let mut start = 0;
                                let mut elem_idx = 0;
                                while start <= s_bytes.len() {
                                    // Find next occurrence of delimiter
                                    let found = if start + d_bytes.len() <= s_bytes.len() {
                                        s_bytes[start..].windows(d_bytes.len())
                                            .position(|w| w == d_bytes.as_slice())
                                    } else {
                                        None
                                    };
                                    let end = match found {
                                        Some(pos) => start + pos,
                                        None => s_bytes.len(),
                                    };
                                    if elem_idx > 0 { result.0.push(sep.bits); }
                                    for &b in &s_bytes[start..end] {
                                        result.0.push(str_byte_mol(b));
                                    }
                                    elem_idx += 1;
                                    if found.is_some() {
                                        start = end + d_bytes.len();
                                    } else {
                                        break;
                                    }
                                }
                                let _ = stack.push(result);
                            }
                        }
                        "__str_contains" => {
                            // Stack: [haystack, needle] → 1.0 if contains, empty if not
                            let needle = vm_pop!(stack, events);
                            let haystack = vm_pop!(stack, events);
                            let h_bytes: Vec<u8> = haystack.0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
                            let n_bytes: Vec<u8> = needle.0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
                            let found = if n_bytes.is_empty() {
                                true
                            } else {
                                h_bytes.windows(n_bytes.len()).any(|w| w == n_bytes.as_slice())
                            };
                            if found {
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__str_replace" => {
                            // Stack: [string, old_pattern, new_pattern] → replaced string
                            let new_pat = vm_pop!(stack, events);
                            let old_pat = vm_pop!(stack, events);
                            let s = vm_pop!(stack, events);
                            let s_bytes: Vec<u8> = s.0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
                            let old_bytes: Vec<u8> = old_pat.0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
                            let new_bytes: Vec<u8> = new_pat.0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
                            let mut result_bytes = Vec::new();
                            let mut i = 0;
                            if old_bytes.is_empty() {
                                result_bytes = s_bytes;
                            } else {
                                while i < s_bytes.len() {
                                    if i + old_bytes.len() <= s_bytes.len()
                                        && s_bytes[i..i + old_bytes.len()] == *old_bytes.as_slice()
                                    {
                                        result_bytes.extend_from_slice(&new_bytes);
                                        i += old_bytes.len();
                                    } else {
                                        result_bytes.push(s_bytes[i]);
                                        i += 1;
                                    }
                                }
                            }
                            let mut mols = Vec::new();
                            for b in result_bytes {
                                mols.push(str_byte_mol(b));
                            }
                            let _ = stack.push(MolecularChain(mols));
                        }
                        "__str_starts_with" => {
                            // Stack: [string, prefix] → 1.0 if starts with, empty if not
                            let prefix = vm_pop!(stack, events);
                            let s = vm_pop!(stack, events);
                            let s_bytes: Vec<u8> = s.0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
                            let p_bytes: Vec<u8> = prefix.0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
                            let starts = s_bytes.starts_with(&p_bytes);
                            if starts {
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__str_ends_with" => {
                            let suffix = vm_pop!(stack, events);
                            let s = vm_pop!(stack, events);
                            let s_bytes: Vec<u8> = s.0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
                            let x_bytes: Vec<u8> = suffix.0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
                            let ends = s_bytes.ends_with(&x_bytes);
                            if ends {
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__str_index_of" => {
                            // Stack: [haystack, needle] → index (number) or -1
                            let needle = vm_pop!(stack, events);
                            let haystack = vm_pop!(stack, events);
                            let h_bytes: Vec<u8> = haystack.0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
                            let n_bytes: Vec<u8> = needle.0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
                            let idx = if n_bytes.is_empty() {
                                0i64
                            } else {
                                h_bytes.windows(n_bytes.len())
                                    .position(|w| w == n_bytes.as_slice())
                                    .map(|i| i as i64)
                                    .unwrap_or(-1)
                            };
                            let _ = stack.push(MolecularChain::from_number(idx as f64));
                        }
                        "__str_trim" => {
                            let s = vm_pop!(stack, events);
                            // Trim leading/trailing whitespace (space=0x20, tab=0x09, etc)
                            let bytes: Vec<u8> = s.0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
                            let trimmed: &[u8] = {
                                let start = bytes.iter().position(|&b| b != b' ' && b != b'\t' && b != b'\n' && b != b'\r').unwrap_or(bytes.len());
                                let end = bytes.iter().rposition(|&b| b != b' ' && b != b'\t' && b != b'\n' && b != b'\r').map(|i| i + 1).unwrap_or(start);
                                &bytes[start..end]
                            };
                            let mut mols = Vec::new();
                            for &b in trimmed {
                                mols.push(str_byte_mol(b));
                            }
                            let _ = stack.push(MolecularChain(mols));
                        }
                        "__str_upper" => {
                            let s = vm_pop!(stack, events);
                            let mut mols = Vec::new();
                            for &bits in &s.0 {
                                let b = (bits & 0xFF) as u8;
                                let upper = if b.is_ascii_lowercase() { b - 32 } else { b };
                                mols.push(str_byte_mol(upper));
                            }
                            let _ = stack.push(MolecularChain(mols));
                        }
                        "__str_lower" => {
                            let s = vm_pop!(stack, events);
                            let mut mols = Vec::new();
                            for &bits in &s.0 {
                                let b = (bits & 0xFF) as u8;
                                let lower = if b.is_ascii_uppercase() { b + 32 } else { b };
                                mols.push(str_byte_mol(lower));
                            }
                            let _ = stack.push(MolecularChain(mols));
                        }
                        "__str_substr" => {
                            // Stack: [string, start, end]
                            // substr(s, start, end) → s[start..end]
                            let end_chain = vm_pop!(stack, events);
                            let start_chain = vm_pop!(stack, events);
                            let s = vm_pop!(stack, events);
                            let start = start_chain.to_number().unwrap_or(0.0) as usize;
                            let end = end_chain.to_number().unwrap_or(0.0) as usize;
                            let end = end.min(s.0.len());
                            let start = start.min(end);
                            let mols: Vec<u16> = s.0[start..end].to_vec();
                            let _ = stack.push(MolecularChain(mols));
                        }
                        // ── Math builtins ──────────────────────────────────
                        "__hyp_floor" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::floor(na)));
                        }
                        "__hyp_ceil" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::ceil(na)));
                        }
                        "__hyp_round" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::round(na)));
                        }
                        "__hyp_sqrt" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::sqrt(na)));
                        }
                        "__hyp_pow" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let nb = b.to_number().unwrap_or(1.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::pow(na, nb)));
                        }
                        "__hyp_log" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(1.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::log(na)));
                        }
                        "__hyp_sin" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::sin(na)));
                        }
                        "__hyp_cos" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::cos(na)));
                        }
                        // ── Dict builtins ──────────────────────────────────
                        "__dict_has_key" => {
                            // Stack: [dict, key] → 1.0 if key exists, empty if not
                            let key = vm_pop!(stack, events);
                            let dict = vm_pop!(stack, events);
                            let found = if let Some(idx) = as_dict_ref(&dict) {
                                idx < dict_heap.len() && dict_heap[idx].iter().any(|(k, _)| k.0 == key.0)
                            } else {
                                let elements = split_dict_chain(&dict);
                                let mut f = false;
                                let mut i = 0;
                                while i + 1 < elements.len() {
                                    if elements[i].0 == key.0 { f = true; break; }
                                    i += 2;
                                }
                                f
                            };
                            if found {
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__dict_values" => {
                            // Stack: [dict] → array of values (odd-indexed elements)
                            let dict = vm_pop!(stack, events);
                            let elements = split_array_chain(&dict);
                            let sep = Molecule::raw(0, 0, 0, 0 , 0);
                            let mut result = MolecularChain(Vec::new());
                            let mut val_idx = 0;
                            let mut i = 1; // start at first value
                            while i < elements.len() {
                                if val_idx > 0 { result.0.push(sep.bits); }
                                result.0.extend(elements[i].0.iter().cloned());
                                val_idx += 1;
                                i += 2;
                            }
                            let _ = stack.push(result);
                        }
                        "__dict_merge" => {
                            // Stack: [dict_a, dict_b] → merged dict (b overrides a)
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let mut a_elems = split_array_chain(&a);
                            let b_elems = split_array_chain(&b);
                            // Merge: for each key in b, update or append to a
                            let mut j = 0;
                            while j + 1 < b_elems.len() {
                                let bkey = &b_elems[j];
                                let bval = &b_elems[j + 1];
                                let mut found = false;
                                let mut k = 0;
                                while k + 1 < a_elems.len() {
                                    if a_elems[k].0 == bkey.0 {
                                        a_elems[k + 1] = bval.clone();
                                        found = true;
                                        break;
                                    }
                                    k += 2;
                                }
                                if !found {
                                    a_elems.push(bkey.clone());
                                    a_elems.push(bval.clone());
                                }
                                j += 2;
                            }
                            let sep = Molecule::raw(0, 0, 0, 0 , 0);
                            let mut result = MolecularChain(Vec::new());
                            for (idx, elem) in a_elems.into_iter().enumerate() {
                                if idx > 0 { result.0.push(sep.bits); }
                                result.0.extend(elem.0.iter().cloned());
                            }
                            let _ = stack.push(result);
                        }
                        "__dict_remove" => {
                            // Stack: [dict, key] → dict without that key
                            let key = vm_pop!(stack, events);
                            let dict = vm_pop!(stack, events);
                            let elements = split_array_chain(&dict);
                            let sep = Molecule::raw(0, 0, 0, 0 , 0);
                            let mut result = MolecularChain(Vec::new());
                            let mut out_idx = 0;
                            let mut i = 0;
                            while i + 1 < elements.len() {
                                if elements[i].0 != key.0 {
                                    if out_idx > 0 { result.0.push(sep.bits); }
                                    result.0.extend(elements[i].0.iter().cloned());
                                    result.0.push(sep.bits);
                                    result.0.extend(elements[i + 1].0.iter().cloned());
                                    out_idx += 1;
                                }
                                i += 2;
                            }
                            let _ = stack.push(result);
                        }
                        // ── Array builtins ─────────────────────────────────
                        "__array_pop" => {
                            // Stack: [array] → [array_without_last, last_element]
                            let arr = vm_pop!(stack, events);
                            if let Some(arr_idx) = as_array_ref(&arr) {
                                if arr_idx < array_heap.len() && !array_heap[arr_idx].is_empty() {
                                    let last = array_heap[arr_idx].pop().unwrap();
                                    let _ = stack.push(arr); // same ref (now shorter)
                                    let _ = stack.push(last);
                                } else {
                                    let _ = stack.push(MolecularChain::empty());
                                    let _ = stack.push(MolecularChain::empty());
                                }
                            } else {
                                let mut elements = split_array_chain(&arr);
                                if let Some(last) = elements.pop() {
                                    let aidx = array_heap.len();
                                    array_heap.push(elements);
                                    let _ = stack.push(make_array_ref(aidx));
                                    let _ = stack.push(last);
                                } else {
                                    let _ = stack.push(MolecularChain::empty());
                                    let _ = stack.push(MolecularChain::empty());
                                }
                            }
                        }
                        "__array_reverse" => {
                            let arr = vm_pop!(stack, events);
                            if let Some(arr_idx) = as_array_ref(&arr) {
                                if arr_idx < array_heap.len() {
                                    array_heap[arr_idx].reverse();
                                }
                                let _ = stack.push(arr);
                            } else {
                                let mut elements = split_array_chain(&arr);
                                elements.reverse();
                                let aidx = array_heap.len();
                                array_heap.push(elements);
                                let _ = stack.push(make_array_ref(aidx));
                            }
                        }
                        "__array_contains" => {
                            // Stack: [array, element] → 1.0 if found, empty if not
                            let elem = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let found = if let Some(arr_idx) = as_array_ref(&arr) {
                                arr_idx < array_heap.len() && array_heap[arr_idx].iter().any(|e| e.0 == elem.0)
                            } else {
                                let elements = split_array_chain(&arr);
                                elements.iter().any(|e| e.0 == elem.0)
                            };
                            if found {
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__array_join" => {
                            // Stack: [array, separator_string] → joined string
                            let sep_chain = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let elements: Vec<MolecularChain> = if let Some(arr_idx) = as_array_ref(&arr) {
                                if arr_idx < array_heap.len() { array_heap[arr_idx].clone() } else { Vec::new() }
                            } else {
                                split_array_chain(&arr)
                            };
                            let mut result = MolecularChain(Vec::new());
                            for (j, elem) in elements.into_iter().enumerate() {
                                if j > 0 {
                                    result.0.extend(sep_chain.0.iter().cloned());
                                }
                                result.0.extend(elem.0.iter().cloned());
                            }
                            let _ = stack.push(result);
                        }
                        "__array_map" => {
                            // Stack: [array_or_iter, closure]
                            // Apply closure to each element, collect results
                            let closure_marker = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            // Check if this is a lazy iterator → delegate to __iter_map
                            let first_part = split_iter_chain(&arr);
                            if first_part.len() >= 2 && chain_to_string(&first_part[0]).unwrap_or_default() == "__ITER__" {
                                // Lazy: append map transform to iterator
                                let isep = iter_sep();
                                let mut result = MolecularChain(Vec::new());
                                result.0.extend(arr.0.iter().copied());
                                result.0.push(isep.bits);
                                result.0.extend(string_to_chain("M").0.iter().copied());
                                result.0.push(isep.bits);
                                result.0.extend(closure_marker.0.iter().copied());
                                let _ = stack.push(result);
                            } else {
                            // Eager array map
                            let elements = if let Some(arr_idx) = as_array_ref(&arr) {
                                if arr_idx < array_heap.len() { array_heap[arr_idx].clone() } else { Vec::new() }
                            } else { split_array_chain(&arr) };
                            let mut mapped_elems: Vec<MolecularChain> = Vec::new();
                            if let Some(body_pc) = closure_body_pc(&closure_marker) {
                                {
                                    for elem in &elements {
                                        let mapped = call_closure_inline(
                                            prog, body_pc, core::slice::from_ref(elem),
                                            &scopes, &mut steps, self.max_steps,
                                        );
                                        mapped_elems.push(mapped);
                                    }
                                }
                            }
                            let idx = array_heap.len();
                            array_heap.push(mapped_elems);
                            let _ = stack.push(make_array_ref(idx));
                            } // end else (eager array map)
                        }
                        "__array_filter" => {
                            // Stack: [array_or_iter, closure]
                            // Keep elements where closure returns non-empty
                            let closure_marker = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            // Check if this is a lazy iterator → delegate
                            let first_part = split_iter_chain(&arr);
                            if first_part.len() >= 2 && chain_to_string(&first_part[0]).unwrap_or_default() == "__ITER__" {
                                let isep = iter_sep();
                                let mut result = MolecularChain(Vec::new());
                                result.0.extend(arr.0.iter().copied());
                                result.0.push(isep.bits);
                                result.0.extend(string_to_chain("F").0.iter().copied());
                                result.0.push(isep.bits);
                                result.0.extend(closure_marker.0.iter().copied());
                                let _ = stack.push(result);
                            } else {
                            // Eager array filter
                            let elements = if let Some(arr_idx) = as_array_ref(&arr) {
                                if arr_idx < array_heap.len() { array_heap[arr_idx].clone() } else { Vec::new() }
                            } else { split_array_chain(&arr) };
                            let mut filtered: Vec<MolecularChain> = Vec::new();
                            if let Some(body_pc) = closure_body_pc(&closure_marker) {
                                {
                                    for elem in &elements {
                                        let keep = call_closure_inline(
                                            prog, body_pc, core::slice::from_ref(elem),
                                            &scopes, &mut steps, self.max_steps,
                                        );
                                        if !keep.is_empty() {
                                            filtered.push(elem.clone());
                                        }
                                    }
                                }
                            }
                            let idx = array_heap.len();
                            array_heap.push(filtered);
                            let _ = stack.push(make_array_ref(idx));
                            } // end else (eager array filter)
                        }
                        "__array_fold" => {
                            // Stack: [array, init, closure]
                            // Fold (reduce) array with 2-arg closure: |acc, elem| { ... }
                            let closure_marker = vm_pop!(stack, events);
                            let init = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let elements = if let Some(arr_idx) = as_array_ref(&arr) {
                                if arr_idx < array_heap.len() { array_heap[arr_idx].clone() } else { Vec::new() }
                            } else { split_array_chain(&arr) };
                            let mut acc = init;
                            if let Some(body_pc) = closure_body_pc(&closure_marker) {
                                {
                                    for elem in &elements {
                                        acc = call_closure_inline(
                                            prog, body_pc, &[acc, elem.clone()],
                                            &scopes, &mut steps, self.max_steps,
                                        );
                                    }
                                }
                            }
                            let _ = stack.push(acc);
                        }
                        "__array_any" => {
                            // Stack: [array, closure]
                            // Returns 1.0 if any element satisfies predicate, else empty
                            let closure_marker = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let elements = if let Some(arr_idx) = as_array_ref(&arr) {
                                if arr_idx < array_heap.len() { array_heap[arr_idx].clone() } else { Vec::new() }
                            } else { split_array_chain(&arr) };
                            let mut found = false;
                            if let Some(body_pc) = closure_body_pc(&closure_marker) {
                                {
                                    for elem in &elements {
                                        let r = call_closure_inline(
                                            prog, body_pc, core::slice::from_ref(elem),
                                            &scopes, &mut steps, self.max_steps,
                                        );
                                        if !r.is_empty() { found = true; break; }
                                    }
                                }
                            }
                            let _ = stack.push(if found {
                                MolecularChain::from_number(1.0)
                            } else {
                                MolecularChain::empty()
                            });
                        }
                        "__array_all" => {
                            // Stack: [array, closure]
                            // Returns 1.0 if all elements satisfy predicate, else empty
                            let closure_marker = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let elements = if let Some(arr_idx) = as_array_ref(&arr) {
                                if arr_idx < array_heap.len() { array_heap[arr_idx].clone() } else { Vec::new() }
                            } else { split_array_chain(&arr) };
                            let mut all_pass = true;
                            if let Some(body_pc) = closure_body_pc(&closure_marker) {
                                {
                                    for elem in &elements {
                                        let r = call_closure_inline(
                                            prog, body_pc, core::slice::from_ref(elem),
                                            &scopes, &mut steps, self.max_steps,
                                        );
                                        if r.is_empty() { all_pass = false; break; }
                                    }
                                }
                            }
                            let _ = stack.push(if all_pass {
                                MolecularChain::from_number(1.0)
                            } else {
                                MolecularChain::empty()
                            });
                        }
                        "__array_find" => {
                            // Stack: [array, closure]
                            // Returns first element satisfying predicate, or empty
                            let closure_marker = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let elements = if let Some(arr_idx) = as_array_ref(&arr) {
                                if arr_idx < array_heap.len() { array_heap[arr_idx].clone() } else { Vec::new() }
                            } else { split_array_chain(&arr) };
                            let mut found = MolecularChain::empty();
                            if let Some(body_pc) = closure_body_pc(&closure_marker) {
                                {
                                    for elem in &elements {
                                        let r = call_closure_inline(
                                            prog, body_pc, core::slice::from_ref(elem),
                                            &scopes, &mut steps, self.max_steps,
                                        );
                                        if !r.is_empty() {
                                            found = elem.clone();
                                            break;
                                        }
                                    }
                                }
                            }
                            let _ = stack.push(found);
                        }
                        "__array_enumerate" => {
                            // Stack: [array]
                            // Returns array of [index, element] pairs (flattened as [0, e0, 1, e1, ...])
                            let arr = vm_pop!(stack, events);
                            let elements = if let Some(arr_idx) = as_array_ref(&arr) {
                                if arr_idx < array_heap.len() { array_heap[arr_idx].clone() } else { Vec::new() }
                            } else { split_array_chain(&arr) };
                            let sep = Molecule::raw(0, 0, 0, 0 , 0);
                            let mut result = MolecularChain(Vec::new());
                            for (i, elem) in elements.iter().enumerate() {
                                if i > 0 { result.0.push(sep.bits); }
                                // Each pair is [idx_chain + sep + elem_chain]
                                let idx_chain = MolecularChain::from_number(i as f64);
                                result.0.extend(idx_chain.0.iter().cloned());
                                result.0.push(sep.bits);
                                result.0.extend(elem.0.iter().cloned());
                            }
                            let _ = stack.push(result);
                        }
                        "__array_count" => {
                            // Stack: [array, closure]
                            // Count elements satisfying predicate
                            let closure_marker = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let elements = if let Some(arr_idx) = as_array_ref(&arr) {
                                if arr_idx < array_heap.len() { array_heap[arr_idx].clone() } else { Vec::new() }
                            } else { split_array_chain(&arr) };
                            let mut count = 0usize;
                            if let Some(body_pc) = closure_body_pc(&closure_marker) {
                                {
                                    for elem in &elements {
                                        let r = call_closure_inline(
                                            prog, body_pc, core::slice::from_ref(elem),
                                            &scopes, &mut steps, self.max_steps,
                                        );
                                        if !r.is_empty() { count += 1; }
                                    }
                                }
                            }
                            let _ = stack.push(MolecularChain::from_number(count as f64));
                        }
                        // ── ISL builtins ───────────────────────────────────
                        "__isl_send" => {
                            // Stack: [address_chain, payload_chain]
                            let payload = vm_pop!(stack, events);
                            let addr = vm_pop!(stack, events);
                            events.push(VmEvent::Output(MolecularChain(alloc::vec![
                                Molecule::raw(0x0A, 0x06, 0x01, 0 , 0x03 ).bits
                            ])));
                            // Emit ISL send event for runtime to handle
                            events.push(VmEvent::CreateEdge {
                                from: addr.chain_hash(),
                                to: payload.chain_hash(),
                                rel: 0x06, // Causes (send triggers receive)
                            });
                            let _ = stack.push(MolecularChain::from_number(1.0)); // success
                        }
                        "__isl_broadcast" => {
                            // Stack: [payload_chain]
                            let payload = vm_pop!(stack, events);
                            events.push(VmEvent::Output(payload));
                            let _ = stack.push(MolecularChain::from_number(1.0));
                        }
                        // ── Type conversion ────────────────────────────────
                        "__type_of" => {
                            let val = vm_pop!(stack, events);
                            let type_name = classify_chain(&val);
                            // Encode type name as string chain
                            let mut mols: Vec<u16> = Vec::new();
                            for b in type_name.bytes() {
                                mols.push(str_byte_mol(b));
                            }
                            let _ = stack.push(MolecularChain(mols));
                        }
                        "__chain_hash" => {
                            let val = vm_pop!(stack, events);
                            let hash = val.chain_hash();
                            let _ = stack.push(MolecularChain::from_number(hash as f64));
                        }
                        "__chain_len" => {
                            let val = vm_pop!(stack, events);
                            let _ = stack.push(MolecularChain::from_number(val.0.len() as f64));
                        }
                        // ── Device I/O builtins ──────────────────────────
                        "__device_write" => {
                            // Stack: [device_id_string, value_chain]
                            let value_chain = vm_pop!(stack, events);
                            let id_chain = vm_pop!(stack, events);
                            let device_id = chain_to_string(&id_chain).unwrap_or_default();
                            let value = value_chain.to_number().unwrap_or(0.0) as u8;
                            events.push(VmEvent::DeviceWrite { device_id, value });
                            let _ = stack.push(MolecularChain::from_number(1.0));
                        }
                        "__device_read" => {
                            // Stack: [device_id_string]
                            let id_chain = vm_pop!(stack, events);
                            let device_id = chain_to_string(&id_chain).unwrap_or_default();
                            events.push(VmEvent::DeviceRead { device_id });
                            let _ = stack.push(MolecularChain::empty()); // placeholder
                        }
                        "__device_list" => {
                            events.push(VmEvent::DeviceListRequest);
                            let _ = stack.push(MolecularChain::empty());
                        }
                        // ── FFI builtins ─────────────────────────────────
                        "__ffi" => {
                            // Stack: [function_name_string, ...args]
                            let name_chain = vm_pop!(stack, events);
                            let fn_name = chain_to_string(&name_chain).unwrap_or_default();
                            events.push(VmEvent::FfiCall { name: fn_name, args: Vec::new() });
                            let _ = stack.push(MolecularChain::empty());
                        }
                        // ── File I/O builtins ────────────────────────────
                        "__file_read" => {
                            let path_chain = vm_pop!(stack, events);
                            let path = chain_to_string(&path_chain).unwrap_or_default();
                            #[cfg(feature = "std")]
                            {
                                match std::fs::read(&path) {
                                    Ok(bytes) => {
                                        // Return as array of byte values
                                        let elements: Vec<MolecularChain> = bytes
                                            .iter()
                                            .map(|&b| MolecularChain::from_number(b as f64))
                                            .collect();
                                        let idx = array_heap.len();
                                        array_heap.push(elements);
                                        let _ = stack.push(make_array_ref(idx));
                                    }
                                    Err(_) => {
                                        let _ = stack.push(MolecularChain::empty());
                                    }
                                }
                            }
                            #[cfg(not(feature = "std"))]
                            {
                                events.push(VmEvent::FileReadRequest { path });
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__file_write" => {
                            let data_chain = vm_pop!(stack, events);
                            let path_chain = vm_pop!(stack, events);
                            let path = chain_to_string(&path_chain).unwrap_or_default();
                            #[cfg(feature = "std")]
                            {
                                // Collect bytes: if data is array ref, materialize byte values
                                let data = if let Some(arr_idx) = as_array_ref(&data_chain) {
                                    if arr_idx < array_heap.len() {
                                        array_heap[arr_idx]
                                            .iter()
                                            .map(|c| c.to_number().unwrap_or(0.0) as u8)
                                            .collect::<Vec<u8>>()
                                    } else {
                                        Vec::new()
                                    }
                                } else if let Some(s) = chain_to_string(&data_chain) {
                                    s.into_bytes()
                                } else {
                                    data_chain.to_tagged_bytes()
                                };
                                match std::fs::write(&path, &data) {
                                    Ok(()) => {
                                        let _ = stack.push(MolecularChain::from_number(1.0));
                                    }
                                    Err(_) => {
                                        let _ = stack.push(MolecularChain::from_number(0.0));
                                    }
                                }
                            }
                            #[cfg(not(feature = "std"))]
                            {
                                let data = if let Some(s) = chain_to_string(&data_chain) {
                                    s.into_bytes()
                                } else {
                                    data_chain.to_tagged_bytes()
                                };
                                events.push(VmEvent::FileWriteRequest { path, data });
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            }
                        }
                        "__file_append" => {
                            let data_chain = vm_pop!(stack, events);
                            let path_chain = vm_pop!(stack, events);
                            let path = chain_to_string(&path_chain).unwrap_or_default();
                            #[cfg(feature = "std")]
                            {
                                use std::io::Write;
                                let data = if let Some(arr_idx) = as_array_ref(&data_chain) {
                                    if arr_idx < array_heap.len() {
                                        array_heap[arr_idx]
                                            .iter()
                                            .map(|c| c.to_number().unwrap_or(0.0) as u8)
                                            .collect::<Vec<u8>>()
                                    } else {
                                        Vec::new()
                                    }
                                } else if let Some(s) = chain_to_string(&data_chain) {
                                    s.into_bytes()
                                } else {
                                    data_chain.to_tagged_bytes()
                                };
                                match std::fs::OpenOptions::new().append(true).create(true).open(&path) {
                                    Ok(mut f) => {
                                        let ok = f.write_all(&data).is_ok();
                                        let _ = stack.push(MolecularChain::from_number(if ok { 1.0 } else { 0.0 }));
                                    }
                                    Err(_) => {
                                        let _ = stack.push(MolecularChain::from_number(0.0));
                                    }
                                }
                            }
                            #[cfg(not(feature = "std"))]
                            {
                                let data = if let Some(s) = chain_to_string(&data_chain) {
                                    s.into_bytes()
                                } else {
                                    data_chain.to_tagged_bytes()
                                };
                                events.push(VmEvent::FileAppendRequest { path, data });
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            }
                        }
                        // ── Compiler builtins (for builder.ol / self-compile) ──
                        "__parse" => {
                            // Pop source string → parse → push array of Op descriptions
                            let src_chain = vm_pop!(stack, events);
                            if let Some(src) = chain_to_string(&src_chain) {
                                match crate::lang::syntax::parse(&src) {
                                    Ok(stmts) => {
                                        // Push count of statements as success indicator
                                        let _ = stack.push(MolecularChain::from_number(stmts.len() as f64));
                                        // Store stmts in parse_cache for __lower
                                        parse_cache = Some(stmts);
                                    }
                                    Err(_) => {
                                        let _ = stack.push(MolecularChain::empty());
                                    }
                                }
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__lower" => {
                            // Use cached parse result → lower → push op count
                            if let Some(ref stmts) = parse_cache {
                                let program = crate::lang::semantic::lower(stmts);
                                let _ = stack.push(MolecularChain::from_number(program.ops.len() as f64));
                                lower_cache = Some(program);
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__encode_bytecode" => {
                            // Use cached lower result → encode → push bytecode as byte array
                            if let Some(ref program) = lower_cache {
                                let bc = crate::exec::bytecode::encode_bytecode(&program.ops);
                                let elements: Vec<MolecularChain> = bc
                                    .iter()
                                    .map(|&b| MolecularChain::from_number(b as f64))
                                    .collect();
                                let idx = array_heap.len();
                                array_heap.push(elements);
                                let _ = stack.push(make_array_ref(idx));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__list_files" => {
                            // Pop dir path, pop extension filter → push array of file paths
                            let ext_chain = vm_pop!(stack, events);
                            let dir_chain = vm_pop!(stack, events);
                            let dir = chain_to_string(&dir_chain).unwrap_or_default();
                            let ext = chain_to_string(&ext_chain).unwrap_or_default();
                            #[cfg(feature = "std")]
                            {
                                let mut files: Vec<MolecularChain> = Vec::new();
                                if let Ok(entries) = std::fs::read_dir(&dir) {
                                    let mut paths: Vec<String> = Vec::new();
                                    for entry in entries.flatten() {
                                        let p = entry.path();
                                        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                                            if ext.is_empty() || name.ends_with(&ext) {
                                                if let Some(ps) = p.to_str() {
                                                    paths.push(ps.to_string());
                                                }
                                            }
                                        }
                                    }
                                    paths.sort(); // deterministic order
                                    for path_str in paths {
                                        files.push(string_to_chain(&path_str));
                                    }
                                }
                                let idx = array_heap.len();
                                array_heap.push(files);
                                let _ = stack.push(make_array_ref(idx));
                            }
                            #[cfg(not(feature = "std"))]
                            {
                                let _ = (dir, ext);
                                let idx = array_heap.len();
                                array_heap.push(Vec::new());
                                let _ = stack.push(make_array_ref(idx));
                            }
                        }
                        // ── Time builtins ────────────────────────────────
                        "__time" => {
                            // Push 0 — Runtime injects actual timestamp
                            let _ = stack.push(MolecularChain::from_number(0.0));
                        }
                        "__sleep" => {
                            // Pop duration_ms — no-op in VM (Runtime handles)
                            let _duration = vm_pop!(stack, events);
                        }

                        // ── Type system builtins ──────────────────────────────
                        "__struct_def" => {
                            // Stack: [fields_array, name_chain]
                            // Register struct type — store in scope as metadata
                            let name_chain = vm_pop!(stack, events);
                            let fields = vm_pop!(stack, events);
                            // Store struct definition: "__struct_Name" → fields array
                            let name = chain_to_string(&name_chain).unwrap_or_default();
                            let key = alloc::format!("__struct_{}", name);
                            let scope = scopes.last_mut().unwrap();
                            scope.push((key, fields));
                        }
                        "__struct_tag" => {
                            // Stack: [dict_chain, name_chain]
                            // Tag a dict with a struct type name
                            let name_chain = vm_pop!(stack, events);
                            let dict = vm_pop!(stack, events);
                            let type_key = string_to_chain("__type");
                            if let Some(idx) = as_dict_ref(&dict) {
                                // Heap-based dict: prepend __type entry
                                if idx < dict_heap.len() {
                                    dict_heap[idx].insert(0, (type_key, name_chain));
                                }
                                let _ = stack.push(dict); // return same ref
                            } else {
                                // Legacy flat chain: prepend __type
                                let sep = Molecule::raw(0, 0, 0, 0, 0);
                                let mut tagged = MolecularChain(Vec::new());
                                tagged.0.extend(type_key.0.iter().copied());
                                tagged.0.push(sep.bits);
                                tagged.0.extend(name_chain.0.iter().copied());
                                if !dict.is_empty() {
                                    tagged.0.push(sep.bits);
                                    tagged.0.extend(dict.0.iter().copied());
                                }
                                let _ = stack.push(tagged);
                            }
                        }
                        "__enum_def" => {
                            // Stack: [variants_array, name_chain]
                            let name_chain = vm_pop!(stack, events);
                            let variants = vm_pop!(stack, events);
                            let name = chain_to_string(&name_chain).unwrap_or_default();
                            let key = alloc::format!("__enum_{}", name);
                            let scope = scopes.last_mut().unwrap();
                            scope.push((key, variants));
                        }
                        "__enum_unit" => {
                            // Stack: [tag_chain]
                            // Unit variant — just push tag as-is
                            // tag is already on stack
                        }
                        "__enum_payload" => {
                            // Stack: [tag, arg0, arg1, ..., count]
                            // Build: tag | sep | arg0 | sep | arg1 ...
                            let count_chain = vm_pop!(stack, events);
                            let count = count_chain.to_number().unwrap_or(0.0) as usize;
                            let mut args = Vec::new();
                            for _ in 0..count {
                                args.push(vm_pop!(stack, events));
                            }
                            args.reverse();
                            let tag = vm_pop!(stack, events);
                            let sep = Molecule::raw(0, 0, 0, 0 , 0);
                            let mut result = MolecularChain(Vec::new());
                            result.0.extend(tag.0.iter().copied());
                            for arg in args {
                                result.0.push(sep.bits);
                                result.0.extend(arg.0.iter().copied());
                            }
                            let _ = stack.push(result);
                        }
                        // ── Phase 5 A12: Iterator protocol ──────────────────
                        "__iter_new" => {
                            // Stack: [array] → iterator
                            // Iterator encoding: tag "__ITER__" + iter_sep + source_array
                            // Transforms appended as: iter_sep + "F"/"M" + iter_sep + closure
                            let arr = vm_pop!(stack, events);
                            let tag = string_to_chain("__ITER__");
                            let isep = iter_sep();
                            let mut result = MolecularChain(Vec::new());
                            result.0.extend(tag.0.iter().copied());
                            result.0.push(isep.bits);
                            result.0.extend(arr.0.iter().copied());
                            let _ = stack.push(result);
                        }
                        "__iter_filter" | "__iter_map" => {
                            // Stack: [iterator, closure] → iterator with transform appended
                            let closure = vm_pop!(stack, events);
                            let iter_val = vm_pop!(stack, events);
                            let transform_tag = if name == "__iter_filter" { "F" } else { "M" };
                            let isep = iter_sep();
                            let mut result = MolecularChain(Vec::new());
                            result.0.extend(iter_val.0.iter().copied());
                            result.0.push(isep.bits);
                            result.0.extend(string_to_chain(transform_tag).0.iter().copied());
                            result.0.push(isep.bits);
                            result.0.extend(closure.0.iter().copied());
                            let _ = stack.push(result);
                        }
                        "__iter_collect" => {
                            // Stack: [iterator] → array (eagerly evaluate all transforms)
                            let iter_val = vm_pop!(stack, events);
                            let parts = split_iter_chain(&iter_val);
                            // parts[0] = "__ITER__" tag
                            // parts[1] = source array
                            // parts[2..] = transform pairs: [type, closure, type, closure, ...]
                            if parts.len() >= 2 {
                                let tag_str = chain_to_string(&parts[0]).unwrap_or_default();
                                if tag_str == "__ITER__" {
                                    let source = &parts[1];
                                    let mut elements = if let Some(arr_idx) = as_array_ref(source) {
                                        if arr_idx < array_heap.len() { array_heap[arr_idx].clone() } else { Vec::new() }
                                    } else { split_array_chain(source) };

                                    // Apply transforms in order
                                    let mut i = 2;
                                    while i + 1 < parts.len() {
                                        let xform = chain_to_string(&parts[i]).unwrap_or_default();
                                        let closure_marker = &parts[i + 1];
                                        i += 2;

                                        if let Some(body_pc) = closure_body_pc(closure_marker) {
                                            {
                                                match xform.as_str() {
                                                    "F" => {
                                                        // Filter: keep elements where closure returns non-empty
                                                        elements = elements.into_iter().filter(|elem| {
                                                            let r = call_closure_inline(
                                                                prog, body_pc, core::slice::from_ref(elem),
                                                                &scopes, &mut steps, self.max_steps,
                                                            );
                                                            !r.is_empty()
                                                        }).collect();
                                                    }
                                                    "M" => {
                                                        // Map: transform each element
                                                        elements = elements.into_iter().map(|elem| {
                                                            call_closure_inline(
                                                                prog, body_pc, core::slice::from_ref(&elem),
                                                                &scopes, &mut steps, self.max_steps,
                                                            )
                                                        }).collect();
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }

                                    // Build result array on heap
                                    let idx = array_heap.len();
                                    array_heap.push(elements);
                                    let _ = stack.push(make_array_ref(idx));
                                } else {
                                    let _ = stack.push(iter_val);
                                }
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__iter_next" => {
                            // Stack: [iterator] → [next_elem, updated_iterator]
                            // Simple: pop first element from source
                            let iter_val = vm_pop!(stack, events);
                            let parts = split_iter_chain(&iter_val);
                            if parts.len() >= 2 {
                                let source = &parts[1];
                                let elements = if let Some(arr_idx) = as_array_ref(source) {
                                    if arr_idx < array_heap.len() { array_heap[arr_idx].clone() } else { Vec::new() }
                                } else { split_array_chain(source) };
                                if elements.is_empty() {
                                    let _ = stack.push(string_to_chain("Option::None"));
                                } else {
                                    // Return Some(first_element)
                                    let first = elements[0].clone();
                                    let tag = string_to_chain("Option::Some");
                                    let sep = Molecule::raw(0, 0, 0, 0, 0);
                                    let mut result = MolecularChain(Vec::new());
                                    result.0.extend(tag.0.iter().copied());
                                    result.0.push(sep.bits);
                                    result.0.extend(first.0.iter().copied());
                                    let _ = stack.push(result);
                                }
                            } else {
                                let _ = stack.push(string_to_chain("Option::None"));
                            }
                        }
                        "__iter_take" => {
                            // Stack: [iterator, n] → iterator (limit to first n elements)
                            let n_chain = vm_pop!(stack, events);
                            let iter_val = vm_pop!(stack, events);
                            let n = n_chain.to_number().unwrap_or(0.0) as usize;
                            let parts = split_iter_chain(&iter_val);
                            if parts.len() >= 2 && chain_to_string(&parts[0]).unwrap_or_default() == "__ITER__" {
                                let source = &parts[1];
                                let elements = if let Some(arr_idx) = as_array_ref(source) {
                                    if arr_idx < array_heap.len() { array_heap[arr_idx].clone() } else { Vec::new() }
                                } else { split_array_chain(source) };
                                let taken: Vec<_> = elements.into_iter().take(n).collect();
                                // Store taken elements on heap and rebuild iterator
                                let new_arr_idx = array_heap.len();
                                array_heap.push(taken);
                                let new_source = make_array_ref(new_arr_idx);
                                let tag = string_to_chain("__ITER__");
                                let isep = iter_sep();
                                let mut result = MolecularChain(Vec::new());
                                result.0.extend(tag.0.iter().copied());
                                result.0.push(isep.bits);
                                result.0.extend(new_source.0.iter().copied());
                                // Keep transforms
                                for p in parts.iter().skip(2) {
                                    result.0.push(isep.bits);
                                    result.0.extend(p.0.iter().copied());
                                }
                                let _ = stack.push(result);
                            } else {
                                let _ = stack.push(iter_val);
                            }
                        }
                        "__iter_skip" => {
                            // Stack: [iterator, n] → iterator (skip first n elements)
                            let n_chain = vm_pop!(stack, events);
                            let iter_val = vm_pop!(stack, events);
                            let n = n_chain.to_number().unwrap_or(0.0) as usize;
                            let parts = split_iter_chain(&iter_val);
                            if parts.len() >= 2 && chain_to_string(&parts[0]).unwrap_or_default() == "__ITER__" {
                                let source = &parts[1];
                                let elements = if let Some(arr_idx) = as_array_ref(source) {
                                    if arr_idx < array_heap.len() { array_heap[arr_idx].clone() } else { Vec::new() }
                                } else { split_array_chain(source) };
                                let skipped: Vec<_> = elements.into_iter().skip(n).collect();
                                let new_arr_idx = array_heap.len();
                                array_heap.push(skipped);
                                let new_source = make_array_ref(new_arr_idx);
                                let tag = string_to_chain("__ITER__");
                                let isep = iter_sep();
                                let mut result = MolecularChain(Vec::new());
                                result.0.extend(tag.0.iter().copied());
                                result.0.push(isep.bits);
                                result.0.extend(new_source.0.iter().copied());
                                for p in parts.iter().skip(2) {
                                    result.0.push(isep.bits);
                                    result.0.extend(p.0.iter().copied());
                                }
                                let _ = stack.push(result);
                            } else {
                                let _ = stack.push(iter_val);
                            }
                        }
                        "__iter_sum" => {
                            // Stack: [iterator] → number (sum of all elements after transforms)
                            let collected = vm_pop!(stack, events);
                            let parts = split_iter_chain(&collected);
                            if parts.len() >= 2 && chain_to_string(&parts[0]).unwrap_or_default() == "__ITER__" {
                                let source = &parts[1];
                                let elements = if let Some(arr_idx) = as_array_ref(source) {
                                    if arr_idx < array_heap.len() { array_heap[arr_idx].clone() } else { Vec::new() }
                                } else { split_array_chain(source) };
                                let mut total = 0.0f64;
                                for elem in &elements {
                                    total += elem.to_number().unwrap_or(0.0);
                                }
                                let _ = stack.push(MolecularChain::from_number(total));
                            } else {
                                let _ = stack.push(MolecularChain::from_number(0.0));
                            }
                        }
                        "__iter_min" => {
                            let iter_val = vm_pop!(stack, events);
                            let parts = split_iter_chain(&iter_val);
                            if parts.len() >= 2 && chain_to_string(&parts[0]).unwrap_or_default() == "__ITER__" {
                                let elements = if let Some(arr_idx) = as_array_ref(&parts[1]) {
                                    if arr_idx < array_heap.len() { array_heap[arr_idx].clone() } else { Vec::new() }
                                } else { split_array_chain(&parts[1]) };
                                let min = elements.iter().filter_map(|e| e.to_number()).fold(f64::INFINITY, f64::min);
                                if min == f64::INFINITY {
                                    let _ = stack.push(string_to_chain("Option::None"));
                                } else {
                                    let _ = stack.push(MolecularChain::from_number(min));
                                }
                            } else {
                                let _ = stack.push(string_to_chain("Option::None"));
                            }
                        }
                        "__iter_max" => {
                            let iter_val = vm_pop!(stack, events);
                            let parts = split_iter_chain(&iter_val);
                            if parts.len() >= 2 && chain_to_string(&parts[0]).unwrap_or_default() == "__ITER__" {
                                let elements = if let Some(arr_idx) = as_array_ref(&parts[1]) {
                                    if arr_idx < array_heap.len() { array_heap[arr_idx].clone() } else { Vec::new() }
                                } else { split_array_chain(&parts[1]) };
                                let max = elements.iter().filter_map(|e| e.to_number()).fold(f64::NEG_INFINITY, f64::max);
                                if max == f64::NEG_INFINITY {
                                    let _ = stack.push(string_to_chain("Option::None"));
                                } else {
                                    let _ = stack.push(MolecularChain::from_number(max));
                                }
                            } else {
                                let _ = stack.push(string_to_chain("Option::None"));
                            }
                        }
                        "__iter_chain" => {
                            // Stack: [iter_a, iter_b] → combined iterator
                            let iter_b = vm_pop!(stack, events);
                            let iter_a = vm_pop!(stack, events);
                            let parts_a = split_iter_chain(&iter_a);
                            let parts_b = split_iter_chain(&iter_b);
                            if parts_a.len() >= 2 && parts_b.len() >= 2 {
                                let src_a = split_array_chain(&parts_a[1]);
                                let src_b = split_array_chain(&parts_b[1]);
                                let sep = Molecule::raw(0, 0, 0, 0, 0);
                                let mut combined = MolecularChain(Vec::new());
                                for (i, e) in src_a.iter().chain(src_b.iter()).enumerate() {
                                    if i > 0 { combined.0.push(sep.bits); }
                                    combined.0.extend(e.0.iter().copied());
                                }
                                let tag = string_to_chain("__ITER__");
                                let isep = iter_sep();
                                let mut result = MolecularChain(Vec::new());
                                result.0.extend(tag.0.iter().copied());
                                result.0.push(isep.bits);
                                result.0.extend(combined.0.iter().copied());
                                let _ = stack.push(result);
                            } else {
                                let _ = stack.push(iter_a);
                            }
                        }
                        // ── Phase 5 A12: Additional iterator methods ──────
                        "__iter_zip" => {
                            // Stack: [iter_a, iter_b] → array of [a_i, b_i] pairs
                            let iter_b = vm_pop!(stack, events);
                            let iter_a = vm_pop!(stack, events);
                            let elems_a = {
                                let parts = split_iter_chain(&iter_a);
                                if parts.len() >= 2 && chain_to_string(&parts[0]).unwrap_or_default() == "__ITER__" {
                                    split_array_chain(&parts[1])
                                } else { split_array_chain(&iter_a) }
                            };
                            let elems_b = {
                                let parts = split_iter_chain(&iter_b);
                                if parts.len() >= 2 && chain_to_string(&parts[0]).unwrap_or_default() == "__ITER__" {
                                    split_array_chain(&parts[1])
                                } else { split_array_chain(&iter_b) }
                            };
                            let sep = Molecule::raw(0, 0, 0, 0, 0);
                            let mut result = MolecularChain(Vec::new());
                            let len = elems_a.len().min(elems_b.len());
                            for i in 0..len {
                                if i > 0 { result.0.push(sep.bits); }
                                // Each pair is a 2-element sub-array: [a_i, b_i]
                                result.0.extend(elems_a[i].0.iter().copied());
                                result.0.push(sep.bits);
                                result.0.extend(elems_b[i].0.iter().copied());
                            }
                            let tag = string_to_chain("__ITER__");
                            let isep = iter_sep();
                            let mut iter_result = MolecularChain(Vec::new());
                            iter_result.0.extend(tag.0.iter().copied());
                            iter_result.0.push(isep.bits);
                            iter_result.0.extend(result.0.iter().copied());
                            let _ = stack.push(iter_result);
                        }
                        "__iter_flat_map" => {
                            // Stack: [iter_or_array, closure] → flatten mapped results
                            let closure_marker = vm_pop!(stack, events);
                            let arr = vm_pop!(stack, events);
                            let elements = {
                                let parts = split_iter_chain(&arr);
                                if parts.len() >= 2 && chain_to_string(&parts[0]).unwrap_or_default() == "__ITER__" {
                                    split_array_chain(&parts[1])
                                } else { split_array_chain(&arr) }
                            };
                            let sep = Molecule::raw(0, 0, 0, 0, 0);
                            let mut result = MolecularChain(Vec::new());
                            let mut count = 0usize;
                            if let Some(body_pc) = closure_body_pc(&closure_marker) {
                                {
                                    for elem in &elements {
                                        let mapped = call_closure_inline(
                                            prog, body_pc, core::slice::from_ref(elem),
                                            &scopes, &mut steps, self.max_steps,
                                        );
                                        // Flatten: split mapped result and add each sub-element
                                        let sub_elems = split_array_chain(&mapped);
                                        for sub in &sub_elems {
                                            if count > 0 { result.0.push(sep.bits); }
                                            result.0.extend(sub.0.iter().cloned());
                                            count += 1;
                                        }
                                    }
                                }
                            }
                            let _ = stack.push(result);
                        }
                        // ── Phase 5 B11: Set builtins ────────────────────────
                        "__set_new" => {
                            // Create empty set: tagged chain "__SET__" + separator + elements
                            let tag = string_to_chain("__SET__");
                            let _ = stack.push(tag);
                        }
                        "__set_insert" => {
                            // Stack: [set, value]
                            let value = vm_pop!(stack, events);
                            let set = vm_pop!(stack, events);
                            let isep = iter_sep();
                            let set_str = chain_to_string(&set).unwrap_or_default();
                            if set_str == "__SET__" || set_str.starts_with("__SET__") {
                                // Parse existing elements
                                let parts = split_iter_chain(&set);
                                let mut elements: Vec<MolecularChain> = Vec::new();
                                if parts.len() >= 2 {
                                    elements = split_array_chain(&parts[1]);
                                }
                                // Check uniqueness
                                let exists = elements.iter().any(|e| e.0 == value.0);
                                if !exists {
                                    elements.push(value);
                                }
                                // Rebuild set
                                let tag = string_to_chain("__SET__");
                                let sep = Molecule::raw(0, 0, 0, 0, 0);
                                let mut result = MolecularChain(Vec::new());
                                result.0.extend(tag.0.iter().copied());
                                result.0.push(isep.bits);
                                for (i, e) in elements.iter().enumerate() {
                                    if i > 0 { result.0.push(sep.bits); }
                                    result.0.extend(e.0.iter().copied());
                                }
                                let _ = stack.push(result);
                            } else {
                                let _ = stack.push(set);
                            }
                        }
                        "__set_contains" => {
                            // Stack: [set, value] → 1.0 or 0.0
                            let value = vm_pop!(stack, events);
                            let set = vm_pop!(stack, events);
                            let parts = split_iter_chain(&set);
                            let found = if parts.len() >= 2 {
                                let elements = split_array_chain(&parts[1]);
                                elements.iter().any(|e| e.0 == value.0)
                            } else { false };
                            let _ = stack.push(MolecularChain::from_number(if found { 1.0 } else { 0.0 }));
                        }
                        "__set_remove" => {
                            // Stack: [set, value] → new set without value
                            let value = vm_pop!(stack, events);
                            let set = vm_pop!(stack, events);
                            let parts = split_iter_chain(&set);
                            if parts.len() >= 2 {
                                let elements = split_array_chain(&parts[1]);
                                let filtered: Vec<_> = elements.into_iter().filter(|e| e.0 != value.0).collect();
                                let tag = string_to_chain("__SET__");
                                let isep = iter_sep();
                                let sep = Molecule::raw(0, 0, 0, 0, 0);
                                let mut result = MolecularChain(Vec::new());
                                result.0.extend(tag.0.iter().copied());
                                result.0.push(isep.bits);
                                for (i, e) in filtered.iter().enumerate() {
                                    if i > 0 { result.0.push(sep.bits); }
                                    result.0.extend(e.0.iter().copied());
                                }
                                let _ = stack.push(result);
                            } else {
                                let _ = stack.push(set);
                            }
                        }
                        "__set_len" => {
                            let set = vm_pop!(stack, events);
                            let parts = split_iter_chain(&set);
                            let len = if parts.len() >= 2 {
                                let elements = split_array_chain(&parts[1]);
                                elements.len()
                            } else { 0 };
                            let _ = stack.push(MolecularChain::from_number(len as f64));
                        }
                        "__set_union" => {
                            // Stack: [set_a, set_b] → union
                            let set_b = vm_pop!(stack, events);
                            let set_a = vm_pop!(stack, events);
                            let parts_a = split_iter_chain(&set_a);
                            let parts_b = split_iter_chain(&set_b);
                            let elems_a = if parts_a.len() >= 2 { split_array_chain(&parts_a[1]) } else { Vec::new() };
                            let elems_b = if parts_b.len() >= 2 { split_array_chain(&parts_b[1]) } else { Vec::new() };
                            let mut union_elems = elems_a;
                            for e in elems_b {
                                if !union_elems.iter().any(|x| x.0 == e.0) {
                                    union_elems.push(e);
                                }
                            }
                            let tag = string_to_chain("__SET__");
                            let isep = iter_sep();
                            let sep = Molecule::raw(0, 0, 0, 0, 0);
                            let mut result = MolecularChain(Vec::new());
                            result.0.extend(tag.0.iter().copied());
                            result.0.push(isep.bits);
                            for (i, e) in union_elems.iter().enumerate() {
                                if i > 0 { result.0.push(sep.bits); }
                                result.0.extend(e.0.iter().copied());
                            }
                            let _ = stack.push(result);
                        }
                        "__set_intersection" => {
                            let set_b = vm_pop!(stack, events);
                            let set_a = vm_pop!(stack, events);
                            let parts_a = split_iter_chain(&set_a);
                            let parts_b = split_iter_chain(&set_b);
                            let elems_a = if parts_a.len() >= 2 { split_array_chain(&parts_a[1]) } else { Vec::new() };
                            let elems_b = if parts_b.len() >= 2 { split_array_chain(&parts_b[1]) } else { Vec::new() };
                            let inter: Vec<_> = elems_a.into_iter().filter(|a| elems_b.iter().any(|b| b.0 == a.0)).collect();
                            let tag = string_to_chain("__SET__");
                            let isep = iter_sep();
                            let sep = Molecule::raw(0, 0, 0, 0, 0);
                            let mut result = MolecularChain(Vec::new());
                            result.0.extend(tag.0.iter().copied());
                            result.0.push(isep.bits);
                            for (i, e) in inter.iter().enumerate() {
                                if i > 0 { result.0.push(sep.bits); }
                                result.0.extend(e.0.iter().copied());
                            }
                            let _ = stack.push(result);
                        }
                        "__set_difference" => {
                            let set_b = vm_pop!(stack, events);
                            let set_a = vm_pop!(stack, events);
                            let parts_a = split_iter_chain(&set_a);
                            let parts_b = split_iter_chain(&set_b);
                            let elems_a = if parts_a.len() >= 2 { split_array_chain(&parts_a[1]) } else { Vec::new() };
                            let elems_b = if parts_b.len() >= 2 { split_array_chain(&parts_b[1]) } else { Vec::new() };
                            let diff: Vec<_> = elems_a.into_iter().filter(|a| !elems_b.iter().any(|b| b.0 == a.0)).collect();
                            let tag = string_to_chain("__SET__");
                            let isep = iter_sep();
                            let sep = Molecule::raw(0, 0, 0, 0, 0);
                            let mut result = MolecularChain(Vec::new());
                            result.0.extend(tag.0.iter().copied());
                            result.0.push(isep.bits);
                            for (i, e) in diff.iter().enumerate() {
                                if i > 0 { result.0.push(sep.bits); }
                                result.0.extend(e.0.iter().copied());
                            }
                            let _ = stack.push(result);
                        }
                        "__set_to_array" => {
                            let set = vm_pop!(stack, events);
                            let parts = split_iter_chain(&set);
                            if parts.len() >= 2 {
                                let _ = stack.push(parts[1].clone());
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        // ── Phase 5 B11: Deque builtins ──────────────────────
                        "__deque_new" => {
                            let tag = string_to_chain("__DEQUE__");
                            let _ = stack.push(tag);
                        }
                        "__deque_push_back" => {
                            // Stack: [deque, value]
                            let value = vm_pop!(stack, events);
                            let deque = vm_pop!(stack, events);
                            let isep = iter_sep();
                            let parts = split_iter_chain(&deque);
                            let mut elements: Vec<MolecularChain> = if parts.len() >= 2 {
                                split_array_chain(&parts[1])
                            } else { Vec::new() };
                            elements.push(value);
                            let tag = string_to_chain("__DEQUE__");
                            let sep = Molecule::raw(0, 0, 0, 0, 0);
                            let mut result = MolecularChain(Vec::new());
                            result.0.extend(tag.0.iter().copied());
                            result.0.push(isep.bits);
                            for (i, e) in elements.iter().enumerate() {
                                if i > 0 { result.0.push(sep.bits); }
                                result.0.extend(e.0.iter().copied());
                            }
                            let _ = stack.push(result);
                        }
                        "__deque_push_front" => {
                            let value = vm_pop!(stack, events);
                            let deque = vm_pop!(stack, events);
                            let isep = iter_sep();
                            let parts = split_iter_chain(&deque);
                            let mut elements: Vec<MolecularChain> = if parts.len() >= 2 {
                                split_array_chain(&parts[1])
                            } else { Vec::new() };
                            elements.insert(0, value);
                            let tag = string_to_chain("__DEQUE__");
                            let sep = Molecule::raw(0, 0, 0, 0, 0);
                            let mut result = MolecularChain(Vec::new());
                            result.0.extend(tag.0.iter().copied());
                            result.0.push(isep.bits);
                            for (i, e) in elements.iter().enumerate() {
                                if i > 0 { result.0.push(sep.bits); }
                                result.0.extend(e.0.iter().copied());
                            }
                            let _ = stack.push(result);
                        }
                        "__deque_pop_front" => {
                            let deque = vm_pop!(stack, events);
                            let parts = split_iter_chain(&deque);
                            if parts.len() >= 2 {
                                let mut elements = split_array_chain(&parts[1]);
                                if elements.is_empty() {
                                    let _ = stack.push(string_to_chain("Option::None"));
                                } else {
                                    let front = elements.remove(0);
                                    // Push updated deque back, then the popped value
                                    let tag = string_to_chain("__DEQUE__");
                                    let isep = iter_sep();
                                    let sep = Molecule::raw(0, 0, 0, 0, 0);
                                    let mut result = MolecularChain(Vec::new());
                                    result.0.extend(tag.0.iter().copied());
                                    result.0.push(isep.bits);
                                    for (i, e) in elements.iter().enumerate() {
                                        if i > 0 { result.0.push(sep.bits); }
                                        result.0.extend(e.0.iter().copied());
                                    }
                                    let _ = stack.push(result); // updated deque (caller must re-assign)
                                    let _ = stack.push(front);  // popped value on top
                                }
                            } else {
                                let _ = stack.push(string_to_chain("Option::None"));
                            }
                        }
                        "__deque_pop_back" => {
                            let deque = vm_pop!(stack, events);
                            let parts = split_iter_chain(&deque);
                            if parts.len() >= 2 {
                                let mut elements = split_array_chain(&parts[1]);
                                if elements.is_empty() {
                                    let _ = stack.push(string_to_chain("Option::None"));
                                } else {
                                    let back = elements.pop().unwrap();
                                    let tag = string_to_chain("__DEQUE__");
                                    let isep = iter_sep();
                                    let sep = Molecule::raw(0, 0, 0, 0, 0);
                                    let mut result = MolecularChain(Vec::new());
                                    result.0.extend(tag.0.iter().copied());
                                    result.0.push(isep.bits);
                                    for (i, e) in elements.iter().enumerate() {
                                        if i > 0 { result.0.push(sep.bits); }
                                        result.0.extend(e.0.iter().copied());
                                    }
                                    let _ = stack.push(result);
                                    let _ = stack.push(back);
                                }
                            } else {
                                let _ = stack.push(string_to_chain("Option::None"));
                            }
                        }
                        "__deque_len" => {
                            let deque = vm_pop!(stack, events);
                            let parts = split_iter_chain(&deque);
                            let len = if parts.len() >= 2 { split_array_chain(&parts[1]).len() } else { 0 };
                            let _ = stack.push(MolecularChain::from_number(len as f64));
                        }
                        "__deque_peek_front" => {
                            let deque = vm_pop!(stack, events);
                            let parts = split_iter_chain(&deque);
                            if parts.len() >= 2 {
                                let elements = split_array_chain(&parts[1]);
                                if let Some(front) = elements.first() {
                                    let _ = stack.push(front.clone());
                                } else {
                                    let _ = stack.push(string_to_chain("Option::None"));
                                }
                            } else {
                                let _ = stack.push(string_to_chain("Option::None"));
                            }
                        }
                        "__deque_peek_back" => {
                            let deque = vm_pop!(stack, events);
                            let parts = split_iter_chain(&deque);
                            if parts.len() >= 2 {
                                let elements = split_array_chain(&parts[1]);
                                if let Some(back) = elements.last() {
                                    let _ = stack.push(back.clone());
                                } else {
                                    let _ = stack.push(string_to_chain("Option::None"));
                                }
                            } else {
                                let _ = stack.push(string_to_chain("Option::None"));
                            }
                        }
                        // ── Phase 5 B12: String slice ────────────────────────
                        "__str_slice" => {
                            // Stack: [string_or_array, start, end]
                            // Works for both strings and arrays
                            let end_chain = vm_pop!(stack, events);
                            let start_chain = vm_pop!(stack, events);
                            let val = vm_pop!(stack, events);
                            let start = start_chain.to_number().unwrap_or(0.0) as usize;
                            let end = end_chain.to_number().unwrap_or(u32::MAX as f64) as usize;
                            // Try as string first
                            if let Some(s) = chain_to_string(&val) {
                                let chars: Vec<char> = s.chars().collect();
                                let actual_end = end.min(chars.len());
                                let actual_start = start.min(actual_end);
                                let sliced: String = chars[actual_start..actual_end].iter().collect();
                                let _ = stack.push(string_to_chain(&sliced));
                            } else {
                                // Array slice
                                let elements = split_array_chain(&val);
                                let actual_end = end.min(elements.len());
                                let actual_start = start.min(actual_end);
                                let sep = Molecule::raw(0, 0, 0, 0, 0);
                                let mut result = MolecularChain(Vec::new());
                                for (j, elem) in elements[actual_start..actual_end].iter().enumerate() {
                                    if j > 0 { result.0.push(sep.bits); }
                                    result.0.extend(elem.0.iter().cloned());
                                }
                                let _ = stack.push(result);
                            }
                        }
                        // ── Phase 5 A11: Option/Result .map() ────────────────
                        "__opt_map" => {
                            // Stack: [option_value, closure]
                            let closure_marker = vm_pop!(stack, events);
                            let val = vm_pop!(stack, events);
                            let parts = split_enum_parts(&val);
                            let tag = if !parts.is_empty() { chain_to_string(&parts[0]).unwrap_or_default() } else { chain_to_string(&val).unwrap_or_default() };
                            if tag.ends_with("::None") || tag == "None" || val.is_empty() {
                                let _ = stack.push(string_to_chain("Option::None"));
                            } else {
                                let payload = if parts.len() >= 2 { parts[1].clone() } else { val.clone() };
                                if let Some(body_pc) = closure_body_pc(&closure_marker) {
                                        let mapped = call_closure_inline(prog, body_pc, core::slice::from_ref(&payload), &scopes, &mut steps, self.max_steps);
                                        let some_tag = string_to_chain("Option::Some");
                                        let sep = Molecule::raw(0, 0, 0, 0, 0);
                                        let mut result = MolecularChain(Vec::new());
                                        result.0.extend(some_tag.0.iter().cloned());
                                        result.0.push(sep.bits);
                                        result.0.extend(mapped.0.iter().cloned());
                                        let _ = stack.push(result);
                                } else { let _ = stack.push(val); }
                            }
                        }
                        "__res_map" => {
                            // Stack: [result_value, closure]
                            let closure_marker = vm_pop!(stack, events);
                            let val = vm_pop!(stack, events);
                            let parts = split_enum_parts(&val);
                            let tag = if !parts.is_empty() { chain_to_string(&parts[0]).unwrap_or_default() } else { String::new() };
                            if tag.ends_with("::Ok") || tag == "Ok" {
                                let payload = if parts.len() >= 2 { parts[1].clone() } else { MolecularChain::empty() };
                                if let Some(body_pc) = closure_body_pc(&closure_marker) {
                                        let mapped = call_closure_inline(prog, body_pc, core::slice::from_ref(&payload), &scopes, &mut steps, self.max_steps);
                                        let ok_tag = string_to_chain("Result::Ok");
                                        let sep = Molecule::raw(0, 0, 0, 0, 0);
                                        let mut result = MolecularChain(Vec::new());
                                        result.0.extend(ok_tag.0.iter().cloned());
                                        result.0.push(sep.bits);
                                        result.0.extend(mapped.0.iter().cloned());
                                        let _ = stack.push(result);
                                } else { let _ = stack.push(val); }
                            } else {
                                let _ = stack.push(val); // Err passthrough
                            }
                        }
                        // ── Phase 5 A10: ? error propagation ──────────────
                        "__try_unwrap" => {
                            // Stack: [enum_value]
                            // If tag starts with "Result::Err" or "Option::None" → early return (push value, set Ret)
                            // If tag starts with "Result::Ok" or "Option::Some" → unwrap payload
                            // Otherwise → leave value as-is (non-enum passthrough)
                            let value = vm_pop!(stack, events);
                            let parts = split_enum_parts(&value);
                            let tag_str = if !parts.is_empty() {
                                chain_to_string(&parts[0]).unwrap_or_default()
                            } else {
                                chain_to_string(&value).unwrap_or_default()
                            };

                            if tag_str.ends_with("::Err") || tag_str.ends_with("::None")
                                || tag_str == "Err" || tag_str == "None"
                            {
                                // Early return: push the original Err/None value back
                                // and signal return via Ret event
                                let _ = stack.push(value);
                                // Set PC past end to force return
                                events.push(VmEvent::EarlyReturn);
                            } else if tag_str.ends_with("::Ok") || tag_str.ends_with("::Some")
                                || tag_str == "Ok" || tag_str == "Some"
                            {
                                // Unwrap payload: skip tag, return first payload element
                                if parts.len() >= 2 {
                                    let _ = stack.push(parts[1].clone());
                                } else {
                                    // Ok/Some with no payload → push empty
                                    let _ = stack.push(MolecularChain::empty());
                                }
                            } else if value.is_empty() {
                                // Empty chain treated as None → early return
                                let _ = stack.push(value);
                                events.push(VmEvent::EarlyReturn);
                            } else {
                                // Not an enum — passthrough (truthy value)
                                let _ = stack.push(value);
                            }
                        }
                        // ── Phase 5 A11: Option/Result constructors + methods ──
                        "__opt_some" => {
                            // Stack: [value] → Option::Some(value)
                            let val = vm_pop!(stack, events);
                            let tag = string_to_chain("Option::Some");
                            let sep = Molecule::raw(0, 0, 0, 0, 0);
                            let mut result = MolecularChain(Vec::new());
                            result.0.extend(tag.0.iter().copied());
                            result.0.push(sep.bits);
                            result.0.extend(val.0.iter().copied());
                            let _ = stack.push(result);
                        }
                        "__opt_none" => {
                            // Stack: [] → Option::None
                            let _ = stack.push(string_to_chain("Option::None"));
                        }
                        "__res_ok" => {
                            // Stack: [value] → Result::Ok(value)
                            let val = vm_pop!(stack, events);
                            let tag = string_to_chain("Result::Ok");
                            let sep = Molecule::raw(0, 0, 0, 0, 0);
                            let mut result = MolecularChain(Vec::new());
                            result.0.extend(tag.0.iter().copied());
                            result.0.push(sep.bits);
                            result.0.extend(val.0.iter().copied());
                            let _ = stack.push(result);
                        }
                        "__res_err" => {
                            // Stack: [value] → Result::Err(value)
                            let val = vm_pop!(stack, events);
                            let tag = string_to_chain("Result::Err");
                            let sep = Molecule::raw(0, 0, 0, 0, 0);
                            let mut result = MolecularChain(Vec::new());
                            result.0.extend(tag.0.iter().copied());
                            result.0.push(sep.bits);
                            result.0.extend(val.0.iter().copied());
                            let _ = stack.push(result);
                        }
                        "__opt_is_some" => {
                            let val = vm_pop!(stack, events);
                            let parts = split_enum_parts(&val);
                            let tag = if !parts.is_empty() { chain_to_string(&parts[0]).unwrap_or_default() } else { chain_to_string(&val).unwrap_or_default() };
                            let is = tag.ends_with("::Some") || tag == "Some";
                            let _ = stack.push(MolecularChain::from_number(if is { 1.0 } else { 0.0 }));
                        }
                        "__opt_is_none" => {
                            let val = vm_pop!(stack, events);
                            let parts = split_enum_parts(&val);
                            let tag = if !parts.is_empty() { chain_to_string(&parts[0]).unwrap_or_default() } else { chain_to_string(&val).unwrap_or_default() };
                            let is = tag.ends_with("::None") || tag == "None" || val.is_empty();
                            let _ = stack.push(MolecularChain::from_number(if is { 1.0 } else { 0.0 }));
                        }
                        "__res_is_ok" => {
                            let val = vm_pop!(stack, events);
                            let parts = split_enum_parts(&val);
                            let tag = if !parts.is_empty() { chain_to_string(&parts[0]).unwrap_or_default() } else { chain_to_string(&val).unwrap_or_default() };
                            let is = tag.ends_with("::Ok") || tag == "Ok";
                            let _ = stack.push(MolecularChain::from_number(if is { 1.0 } else { 0.0 }));
                        }
                        "__res_is_err" => {
                            let val = vm_pop!(stack, events);
                            let parts = split_enum_parts(&val);
                            let tag = if !parts.is_empty() { chain_to_string(&parts[0]).unwrap_or_default() } else { chain_to_string(&val).unwrap_or_default() };
                            let is = tag.ends_with("::Err") || tag == "Err";
                            let _ = stack.push(MolecularChain::from_number(if is { 1.0 } else { 0.0 }));
                        }
                        "__opt_unwrap" => {
                            // Unwrap Some payload, panic on None
                            let val = vm_pop!(stack, events);
                            let parts = split_enum_parts(&val);
                            let tag = if !parts.is_empty() { chain_to_string(&parts[0]).unwrap_or_default() } else { chain_to_string(&val).unwrap_or_default() };
                            if tag.ends_with("::None") || tag == "None" || val.is_empty() {
                                events.push(VmEvent::Error(VmError::StackUnderflow));
                            } else if parts.len() >= 2 {
                                let _ = stack.push(parts[1].clone());
                            } else {
                                let _ = stack.push(val);
                            }
                        }
                        "__opt_unwrap_or" => {
                            // Stack: [option_value, default_value]
                            let default = vm_pop!(stack, events);
                            let val = vm_pop!(stack, events);
                            let parts = split_enum_parts(&val);
                            let tag = if !parts.is_empty() { chain_to_string(&parts[0]).unwrap_or_default() } else { chain_to_string(&val).unwrap_or_default() };
                            if tag.ends_with("::None") || tag == "None" || val.is_empty()
                                || tag.ends_with("::Err") || tag == "Err"
                            {
                                let _ = stack.push(default);
                            } else if parts.len() >= 2 {
                                let _ = stack.push(parts[1].clone());
                            } else {
                                let _ = stack.push(val);
                            }
                        }
                        "__res_map_err" => {
                            // Stack: [result_value, closure]
                            // If Err → apply closure to error payload, re-wrap as Err; if Ok → passthrough
                            let closure_marker = vm_pop!(stack, events);
                            let val = vm_pop!(stack, events);
                            let parts = split_enum_parts(&val);
                            let tag = if !parts.is_empty() { chain_to_string(&parts[0]).unwrap_or_default() } else { String::new() };
                            if tag.ends_with("::Err") || tag == "Err" {
                                let payload = if parts.len() >= 2 { parts[1].clone() } else { MolecularChain::empty() };
                                if let Some(body_pc) = closure_body_pc(&closure_marker) {
                                        let mapped = call_closure_inline(prog, body_pc, core::slice::from_ref(&payload), &scopes, &mut steps, self.max_steps);
                                        let err_tag = string_to_chain("Result::Err");
                                        let sep = Molecule::raw(0, 0, 0, 0, 0);
                                        let mut result = MolecularChain(Vec::new());
                                        result.0.extend(err_tag.0.iter().cloned());
                                        result.0.push(sep.bits);
                                        result.0.extend(mapped.0.iter().cloned());
                                        let _ = stack.push(result);
                                } else { let _ = stack.push(val); }
                            } else {
                                let _ = stack.push(val); // Ok passthrough
                            }
                        }
                        "__method_call" => {
                            // Stack: [self, arg0, ..., argN, arg_count, method_name]
                            let method_name_chain = vm_pop!(stack, events);
                            let count_chain = vm_pop!(stack, events);
                            let count = count_chain.to_number().unwrap_or(1.0) as usize;
                            let method_name = chain_to_string(&method_name_chain).unwrap_or_default();
                            // Pop all args (including self)
                            let mut args = Vec::new();
                            for _ in 0..count {
                                args.push(vm_pop!(stack, events));
                            }
                            args.reverse();
                            // Get type tag from self (first arg) — look for __type key
                            let self_val = &args[0];
                            let elements = split_array_chain(self_val);
                            let mut type_name = String::new();
                            let type_key = string_to_chain("__type");
                            let mut i = 0;
                            while i + 1 < elements.len() {
                                if elements[i].0 == type_key.0 {
                                    type_name = chain_to_string(&elements[i + 1]).unwrap_or_default();
                                    break;
                                }
                                i += 2;
                            }
                            // Lookup mangled function name: __Type_method
                            let mangled = alloc::format!("__{type_name}_{method_name}");
                            // Push args back and call the mangled function
                            for arg in &args {
                                let _ = stack.push(arg.clone());
                            }
                            events.push(VmEvent::LookupAlias(mangled));
                        }
                        // ── Trait builtins ──────────────────────────────────
                        "__trait_def" => {
                            // Stack: [methods_array, name_chain]
                            // Register trait definition
                            let name_chain = vm_pop!(stack, events);
                            let methods = vm_pop!(stack, events);
                            let name = chain_to_string(&name_chain).unwrap_or_default();
                            let key = alloc::format!("__trait_{}", name);
                            let scope = scopes.last_mut().unwrap();
                            scope.push((key, methods));
                        }
                        "__trait_impl_register" => {
                            // Stack: [trait_name_chain, type_name_chain]
                            // Register that type implements trait
                            let type_chain = vm_pop!(stack, events);
                            let trait_chain = vm_pop!(stack, events);
                            let trait_name = chain_to_string(&trait_chain).unwrap_or_default();
                            let type_name = chain_to_string(&type_chain).unwrap_or_default();
                            // Store as "__impl_TraitName_TypeName" = 1
                            let key = alloc::format!("__impl_{}_{}", trait_name, type_name);
                            let scope = scopes.last_mut().unwrap();
                            scope.push((key, MolecularChain::from_number(1.0)));
                            // Also store in "__impls_TraitName" list
                            let list_key = alloc::format!("__impls_{}", trait_name);
                            let found = scopes.iter().flat_map(|s| s.iter())
                                .find(|(k, _)| k == &list_key)
                                .map(|(_, v)| v.clone());
                            let sep = Molecule::raw(0, 0, 0, 0 , 0);
                            let type_entry = string_to_chain(&type_name);
                            let mut list = found.unwrap_or_else(MolecularChain::empty);
                            if !list.is_empty() {
                                list.0.push(sep.bits);
                            }
                            list.0.extend(type_entry.0.iter().copied());
                            let scope = scopes.last_mut().unwrap();
                            scope.push((list_key, list));
                        }
                        "__trait_check" => {
                            // Stack: [value, trait_name_chain]
                            // Check if value's type implements the trait
                            let trait_chain = vm_pop!(stack, events);
                            let value = vm_pop!(stack, events);
                            let trait_name = chain_to_string(&trait_chain).unwrap_or_default();
                            // Get type from __type tag
                            let elements = split_array_chain(&value);
                            let type_key = string_to_chain("__type");
                            let mut type_name = String::new();
                            let mut i = 0;
                            while i + 1 < elements.len() {
                                if elements[i].0 == type_key.0 {
                                    type_name = chain_to_string(&elements[i + 1]).unwrap_or_default();
                                    break;
                                }
                                i += 2;
                            }
                            // Check impl registration
                            let impl_key = alloc::format!("__impl_{}_{}", trait_name, type_name);
                            let is_impl = scopes.iter().flat_map(|s| s.iter())
                                .any(|(k, _)| k == &impl_key);
                            let result = if is_impl { 1.0 } else { 0.0 };
                            let _ = stack.push(MolecularChain::from_number(result));
                        }
                        // ── Channel builtins ──────────────────────────────
                        "__channel_new" => {
                            // Create a new channel, push its ID as a number
                            let id = next_channel_id;
                            next_channel_id += 1;
                            channels.push(Vec::new()); // queue for this channel
                            let _ = stack.push(MolecularChain::from_number(id as f64));
                        }
                        "__channel_send" => {
                            // Stack: [channel_id, value]
                            let value = vm_pop!(stack, events);
                            let ch_chain = vm_pop!(stack, events);
                            let ch_id = ch_chain.to_number().unwrap_or(0.0) as usize;
                            if ch_id >= 1 && ch_id <= channels.len() {
                                channels[ch_id - 1].push(value);
                            }
                            let _ = stack.push(MolecularChain::from_number(1.0)); // success
                        }
                        "__channel_recv" => {
                            // Stack: [channel_id] → pops first message or empty
                            let ch_chain = vm_pop!(stack, events);
                            let ch_id = ch_chain.to_number().unwrap_or(0.0) as usize;
                            if ch_id >= 1 && ch_id <= channels.len() && !channels[ch_id - 1].is_empty() {
                                let msg = channels[ch_id - 1].remove(0);
                                let _ = stack.push(msg);
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }

                        "__use_module" => {
                            // Module loading — emit event for Runtime to handle
                            let module_chain = vm_pop!(stack, events);
                            let name = chain_to_string(&module_chain).unwrap_or_default();
                            events.push(VmEvent::UseModule { name });
                        }

                        "__use_module_selective" => {
                            // Selective module import: pop count, import names, module name
                            let count_chain = vm_pop!(stack, events);
                            let count = count_chain.to_number().unwrap_or(0.0) as usize;
                            let mut imports = Vec::new();
                            for _ in 0..count {
                                let imp = vm_pop!(stack, events);
                                imports.push(chain_to_string(&imp).unwrap_or_default());
                            }
                            imports.reverse(); // restore original order
                            let module_chain = vm_pop!(stack, events);
                            let name = chain_to_string(&module_chain).unwrap_or_default();
                            events.push(VmEvent::UseModuleSelective { name, imports });
                        }

                        "__mod_decl" => {
                            // Module declaration — register module path
                            let path_chain = vm_pop!(stack, events);
                            let path = chain_to_string(&path_chain).unwrap_or_default();
                            events.push(VmEvent::ModDecl { path });
                        }

                        // ── Phase 3 B5: String upgrades ────────────────────
                        "__str_matches" => {
                            // Stack: [string, pattern] → 1.0 if matches, empty if not
                            // Simple glob: * = any chars, ? = any single char
                            let pat = vm_pop!(stack, events);
                            let s = vm_pop!(stack, events);
                            let s_str = chain_to_string(&s).unwrap_or_default();
                            let pat_str = chain_to_string(&pat).unwrap_or_default();
                            let matched = glob_match(&s_str, &pat_str);
                            if matched {
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }
                        "__str_chars" => {
                            // Stack: [string] → array of single-char strings
                            let s = vm_pop!(stack, events);
                            let s_str = chain_to_string(&s).unwrap_or_default();
                            let sep = Molecule::raw(0, 0, 0, 0 , 0);
                            let mut mols: Vec<u16> = Vec::new();
                            for ch in s_str.chars() {
                                if !mols.is_empty() { mols.push(sep.bits); }
                                let mut buf = [0u8; 4];
                                let c_str = ch.encode_utf8(&mut buf);
                                for b in c_str.bytes() {
                                    mols.push(Molecule::raw(b, 1, 0x80, 0x80 , 3).bits);
                                }
                            }
                            let _ = stack.push(MolecularChain(mols));
                        }
                        "__str_repeat" => {
                            // Stack: [string, count] → repeated string
                            let count = vm_pop!(stack, events);
                            let s = vm_pop!(stack, events);
                            let s_str = chain_to_string(&s).unwrap_or_default();
                            let n = count.to_number().unwrap_or(1.0) as usize;
                            let repeated = s_str.repeat(n.min(10000));
                            let _ = stack.push(string_to_chain(&repeated));
                        }
                        "__str_pad_left" => {
                            // Stack: [string, width, fill_char] → padded string
                            let fill = vm_pop!(stack, events);
                            let width = vm_pop!(stack, events);
                            let s = vm_pop!(stack, events);
                            let s_str = chain_to_string(&s).unwrap_or_default();
                            let w = width.to_number().unwrap_or(0.0) as usize;
                            let fill_str = chain_to_string(&fill).unwrap_or_else(|| String::from(" "));
                            let fill_ch = fill_str.chars().next().unwrap_or(' ');
                            let pad_count = w.saturating_sub(s_str.len());
                            let mut result = String::new();
                            for _ in 0..pad_count { result.push(fill_ch); }
                            result.push_str(&s_str);
                            let _ = stack.push(string_to_chain(&result));
                        }
                        "__str_pad_right" => {
                            // Stack: [string, width, fill_char] → padded string
                            let fill = vm_pop!(stack, events);
                            let width = vm_pop!(stack, events);
                            let s = vm_pop!(stack, events);
                            let s_str = chain_to_string(&s).unwrap_or_default();
                            let w = width.to_number().unwrap_or(0.0) as usize;
                            let fill_str = chain_to_string(&fill).unwrap_or_else(|| String::from(" "));
                            let fill_ch = fill_str.chars().next().unwrap_or(' ');
                            let mut result = s_str.clone();
                            let pad_count = w.saturating_sub(s_str.len());
                            for _ in 0..pad_count { result.push(fill_ch); }
                            let _ = stack.push(string_to_chain(&result));
                        }
                        "__str_char_at" => {
                            // Stack: [string, index] → single char string or empty
                            // Zero-allocation: direct index into molecule array
                            let idx = vm_pop!(stack, events);
                            let s = vm_pop!(stack, events);
                            let i = idx.to_number().unwrap_or(0.0) as usize;
                            if i < s.0.len() && is_string_chain(&s) {
                                // Direct O(1) access — no String allocation
                                let _ = stack.push(MolecularChain(alloc::vec![s.0[i]]));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }

                        "__str_is_keyword" => {
                            // O(1) keyword check — replaces is_keyword() loop in lexer.ol
                            let s = vm_pop!(stack, events);
                            let bytes: Vec<u8> = s.0.iter().map(|&b| (b & 0xFF) as u8).collect();
                            let is_kw = matches!(&bytes[..],
                                b"let" | b"fn" | b"if" | b"else" | b"loop" | b"while" |
                                b"for" | b"in" | b"return" | b"break" | b"continue" |
                                b"emit" | b"type" | b"union" | b"impl" | b"trait" |
                                b"match" | b"try" | b"catch" | b"spawn" | b"select" |
                                b"timeout" | b"from" | b"use" | b"mod" | b"pub" |
                                b"true" | b"false"
                            );
                            if is_kw {
                                let _ = stack.push(MolecularChain::from_number(1.0));
                            } else {
                                let _ = stack.push(MolecularChain::empty());
                            }
                        }

                        // ── Bytecode encoding builtins (for codegen.ol) ──
                        "__f64_to_le_bytes" => {
                            // Stack: [number] → array of 8 bytes (LE)
                            let n = vm_pop!(stack, events);
                            let f = n.to_number().unwrap_or(0.0);
                            let bytes = f.to_le_bytes();
                            let idx = array_heap.len();
                            let elems: Vec<MolecularChain> = bytes.iter()
                                .map(|&b| MolecularChain::from_number(b as f64))
                                .collect();
                            array_heap.push(elems);
                            let _ = stack.push(make_array_ref(idx));
                        }
                        "__f64_from_le_bytes" => {
                            // Stack: [array_of_8_bytes] → number
                            let arr = vm_pop!(stack, events);
                            let mut bytes = [0u8; 8];
                            if let Some(arr_idx) = as_array_ref(&arr) {
                                if arr_idx < array_heap.len() && array_heap[arr_idx].len() >= 8 {
                                    for i in 0..8 {
                                        bytes[i] = array_heap[arr_idx][i].to_number().unwrap_or(0.0) as u8;
                                    }
                                }
                            }
                            let _ = stack.push(MolecularChain::from_number(f64::from_le_bytes(bytes)));
                        }
                        "__str_bytes" => {
                            // Stack: [string] → array of byte values (UTF-8)
                            let s = vm_pop!(stack, events);
                            let s_str = chain_to_string(&s).unwrap_or_default();
                            let idx = array_heap.len();
                            let elems: Vec<MolecularChain> = s_str.bytes()
                                .map(|b| MolecularChain::from_number(b as f64))
                                .collect();
                            array_heap.push(elems);
                            let _ = stack.push(make_array_ref(idx));
                        }
                        "__bytes_to_str" => {
                            // Stack: [array_of_bytes] → string
                            let arr = vm_pop!(stack, events);
                            let mut bytes = Vec::new();
                            if let Some(arr_idx) = as_array_ref(&arr) {
                                if arr_idx < array_heap.len() {
                                    for elem in &array_heap[arr_idx] {
                                        bytes.push(elem.to_number().unwrap_or(0.0) as u8);
                                    }
                                }
                            }
                            let s = alloc::string::String::from_utf8_lossy(&bytes);
                            let _ = stack.push(string_to_chain(&s));
                        }
                        "__array_concat" => {
                            // Stack: [arr1, arr2] → concatenated array
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let mut result = Vec::new();
                            if let Some(a_idx) = as_array_ref(&a) {
                                if a_idx < array_heap.len() {
                                    result.extend(array_heap[a_idx].iter().cloned());
                                }
                            }
                            if let Some(b_idx) = as_array_ref(&b) {
                                if b_idx < array_heap.len() {
                                    result.extend(array_heap[b_idx].iter().cloned());
                                }
                            }
                            let idx = array_heap.len();
                            array_heap.push(result);
                            let _ = stack.push(make_array_ref(idx));
                        }

                        // ── Phase 3 B6: Bitwise operations ─────────────────
                        "__bit_and" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0) as i64;
                            let nb = b.to_number().unwrap_or(0.0) as i64;
                            let _ = stack.push(MolecularChain::from_number((na & nb) as f64));
                        }
                        "__bit_or" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0) as i64;
                            let nb = b.to_number().unwrap_or(0.0) as i64;
                            let _ = stack.push(MolecularChain::from_number((na | nb) as f64));
                        }
                        "__bit_xor" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0) as i64;
                            let nb = b.to_number().unwrap_or(0.0) as i64;
                            let _ = stack.push(MolecularChain::from_number((na ^ nb) as f64));
                        }
                        "__bit_not" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0) as i64;
                            let _ = stack.push(MolecularChain::from_number((!na) as f64));
                        }
                        "__bit_shl" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0) as i64;
                            let nb = b.to_number().unwrap_or(0.0) as u32;
                            let _ = stack.push(MolecularChain::from_number(na.wrapping_shl(nb.min(63)) as f64));
                        }
                        "__bit_shr" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0) as i64;
                            let nb = b.to_number().unwrap_or(0.0) as u32;
                            let _ = stack.push(MolecularChain::from_number(na.wrapping_shr(nb.min(63)) as f64));
                        }

                        // ── Phase 3 B6: Bytes operations ───────────────────
                        "__bytes_new" => {
                            // Stack: [size] → bytes chain (all zeros)
                            let size = vm_pop!(stack, events);
                            let n = size.to_number().unwrap_or(0.0) as usize;
                            let n = n.min(65536); // safety limit
                            let mut mols: Vec<u16> = Vec::with_capacity(n);
                            for _ in 0..n {
                                mols.push(Molecule::raw(0, 1, 0x80, 0x80 , 3).bits);
                            }
                            let _ = stack.push(MolecularChain(mols));
                        }
                        "__byte_len" => {
                            let a = vm_pop!(stack, events);
                            let _ = stack.push(MolecularChain::from_number(a.0.len() as f64));
                        }
                        "__bytes_get_u8" => {
                            // Stack: [bytes, index] → value
                            let idx = vm_pop!(stack, events);
                            let buf = vm_pop!(stack, events);
                            let i = idx.to_number().unwrap_or(0.0) as usize;
                            if i < buf.0.len() {
                                // Raw byte stored in low 8 bits of u16
                                let _ = stack.push(MolecularChain::from_number((buf.0[i] & 0xFF) as f64));
                            } else {
                                let _ = stack.push(MolecularChain::from_number(0.0));
                            }
                        }
                        "__bytes_set_u8" => {
                            // Stack: [bytes, index, value] → updated bytes
                            let val = vm_pop!(stack, events);
                            let idx = vm_pop!(stack, events);
                            let mut buf = vm_pop!(stack, events);
                            let i = idx.to_number().unwrap_or(0.0) as usize;
                            let v = val.to_number().unwrap_or(0.0) as u8;
                            if i < buf.0.len() {
                                // Store raw byte in low 8 bits, mark high byte to avoid confusion with string/number
                                buf.0[i] = 0x0100 | (v as u16);
                            }
                            let _ = stack.push(buf);
                        }
                        "__bytes_get_u16_be" => {
                            // Stack: [bytes, index] → u16 value (big-endian)
                            let idx = vm_pop!(stack, events);
                            let buf = vm_pop!(stack, events);
                            let i = idx.to_number().unwrap_or(0.0) as usize;
                            if i + 1 < buf.0.len() {
                                let hi = (buf.0[i] & 0xFF) as u16;
                                let lo = (buf.0[i + 1] & 0xFF) as u16;
                                let _ = stack.push(MolecularChain::from_number(((hi << 8) | lo) as f64));
                            } else {
                                let _ = stack.push(MolecularChain::from_number(0.0));
                            }
                        }
                        "__bytes_set_u16_be" => {
                            // Stack: [bytes, index, value] → updated bytes
                            let val = vm_pop!(stack, events);
                            let idx = vm_pop!(stack, events);
                            let mut buf = vm_pop!(stack, events);
                            let i = idx.to_number().unwrap_or(0.0) as usize;
                            let v = val.to_number().unwrap_or(0.0) as u16;
                            if i + 1 < buf.0.len() {
                                buf.0[i] = 0x0100 | ((v >> 8) as u16);
                                buf.0[i + 1] = 0x0100 | ((v & 0xFF) as u16);
                            }
                            let _ = stack.push(buf);
                        }
                        "__bytes_get_u32_be" => {
                            // Stack: [bytes, index] → u32 value (big-endian)
                            let idx = vm_pop!(stack, events);
                            let buf = vm_pop!(stack, events);
                            let i = idx.to_number().unwrap_or(0.0) as usize;
                            if i + 3 < buf.0.len() {
                                let v = ((buf.0[i] & 0xFF) as u32) << 24
                                    | ((buf.0[i + 1] & 0xFF) as u32) << 16
                                    | ((buf.0[i + 2] & 0xFF) as u32) << 8
                                    | ((buf.0[i + 3] & 0xFF) as u32);
                                let _ = stack.push(MolecularChain::from_number(v as f64));
                            } else {
                                let _ = stack.push(MolecularChain::from_number(0.0));
                            }
                        }
                        "__bytes_set_u32_be" => {
                            // Stack: [bytes, index, value] → updated bytes
                            let val = vm_pop!(stack, events);
                            let idx = vm_pop!(stack, events);
                            let mut buf = vm_pop!(stack, events);
                            let i = idx.to_number().unwrap_or(0.0) as usize;
                            let v = val.to_number().unwrap_or(0.0) as u32;
                            if i + 3 < buf.0.len() {
                                buf.0[i] = 0x0100 | ((v >> 24) & 0xFF) as u16;
                                buf.0[i + 1] = 0x0100 | ((v >> 16) & 0xFF) as u16;
                                buf.0[i + 2] = 0x0100 | ((v >> 8) & 0xFF) as u16;
                                buf.0[i + 3] = 0x0100 | (v & 0xFF) as u16;
                            }
                            let _ = stack.push(buf);
                        }
                        "__pack" => {
                            // Simple pack: Stack: [format_str, value1, value2, ...] → bytes chain
                            // Format: "NB" where N = count of bytes per field
                            // e.g., "4B 4B 1B 3B" → 12 bytes
                            let fmt = vm_pop!(stack, events);
                            let fmt_str = chain_to_string(&fmt).unwrap_or_default();
                            let fields: Vec<&str> = fmt_str.split_whitespace().collect();
                            let mut values = Vec::new();
                            for _ in 0..fields.len() {
                                values.push(vm_pop!(stack, events));
                            }
                            values.reverse();
                            let mut result_mols: Vec<u16> = Vec::new();
                            for (field, val) in fields.iter().zip(values.iter()) {
                                let size: usize = field.trim_end_matches('B').parse().unwrap_or(1);
                                let n = val.to_number().unwrap_or(0.0) as u64;
                                for bi in (0..size).rev() {
                                    result_mols.push(Molecule::raw(((n >> (bi * 8)) & 0xFF) as u8, 1, 0x80, 0x80 , 3).bits);
                                }
                            }
                            let _ = stack.push(MolecularChain(result_mols));
                        }
                        "__unpack" => {
                            // Stack: [format_str, bytes_chain] → array of values
                            let bytes = vm_pop!(stack, events);
                            let fmt = vm_pop!(stack, events);
                            let fmt_str = chain_to_string(&fmt).unwrap_or_default();
                            let fields: Vec<&str> = fmt_str.split_whitespace().collect();
                            let sep = Molecule::raw(0, 0, 0, 0 , 0);
                            let mut result_mols: Vec<u16> = Vec::new();
                            let mut offset = 0usize;
                            for field in &fields {
                                let size: usize = field.trim_end_matches('B').parse().unwrap_or(1);
                                let mut val: u64 = 0;
                                for bi in 0..size {
                                    if offset + bi < bytes.0.len() {
                                        val = (val << 8) | Molecule::from_u16(bytes.0[offset + bi]).shape() as u64;
                                    }
                                }
                                offset += size;
                                if !result_mols.is_empty() { result_mols.push(sep.bits); }
                                let num_chain = MolecularChain::from_number(val as f64);
                                result_mols.extend(num_chain.0);
                            }
                            let _ = stack.push(MolecularChain(result_mols));
                        }

                        // ── Phase 3 B7: Math stdlib ────────────────────────
                        "__hyp_tan" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::tan(na)));
                        }
                        "__hyp_atan" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::atan(na)));
                        }
                        "__hyp_atan2" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let nb = b.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::atan2(na, nb)));
                        }
                        "__hyp_exp" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::exp(na)));
                        }
                        "__hyp_ln" => {
                            let a = vm_pop!(stack, events);
                            let na = a.to_number().unwrap_or(0.0);
                            let _ = stack.push(MolecularChain::from_number(homemath::log(na)));
                        }
                        "__hyp_clamp" => {
                            let hi = vm_pop!(stack, events);
                            let lo = vm_pop!(stack, events);
                            let x = vm_pop!(stack, events);
                            let nx = x.to_number().unwrap_or(0.0);
                            let nlo = lo.to_number().unwrap_or(0.0);
                            let nhi = hi.to_number().unwrap_or(1.0);
                            let clamped = if nx < nlo { nlo } else if nx > nhi { nhi } else { nx };
                            let _ = stack.push(MolecularChain::from_number(clamped));
                        }
                        "__math_fib" => {
                            // Fibonacci: fib(n) → F(n)
                            let a = vm_pop!(stack, events);
                            let n = a.to_number().unwrap_or(0.0) as u64;
                            let result = if n <= 1 {
                                n
                            } else {
                                let (mut a, mut b) = (0u64, 1u64);
                                for _ in 0..n - 1 {
                                    let tmp = b;
                                    b = a.saturating_add(b);
                                    a = tmp;
                                }
                                b
                            };
                            let _ = stack.push(MolecularChain::from_number(result as f64));
                        }
                        "__math_pi" => {
                            let _ = stack.push(MolecularChain::from_number(core::f64::consts::PI));
                        }
                        "__math_phi" => {
                            // Golden ratio φ = (1 + √5) / 2
                            let _ = stack.push(MolecularChain::from_number(1.618_033_988_749_895));
                        }

                        // ── Phase 4 B8: Platform detection ──────────────────
                        "__platform_arch" => {
                            let arch_str = if cfg!(target_arch = "x86_64") { "x86_64" }
                                else if cfg!(target_arch = "x86") { "x86" }
                                else if cfg!(target_arch = "aarch64") { "aarch64" }
                                else if cfg!(target_arch = "arm") { "arm" }
                                else if cfg!(target_arch = "riscv64") { "riscv64" }
                                else if cfg!(target_arch = "riscv32") { "riscv32" }
                                else if cfg!(target_arch = "mips") { "mips" }
                                else if cfg!(target_arch = "wasm32") { "wasm32" }
                                else { "unknown" };
                            let _ = stack.push(string_to_chain(arch_str));
                        }
                        "__platform_os" => {
                            let os_str = if cfg!(target_os = "linux") { "linux" }
                                else if cfg!(target_os = "macos") { "macos" }
                                else if cfg!(target_os = "windows") { "windows" }
                                else if cfg!(target_os = "none") { "bare" }
                                else { "unknown" };
                            let _ = stack.push(string_to_chain(os_str));
                        }
                        "__platform_memory" => {
                            // Returns 0 — Runtime can inject actual value
                            let _ = stack.push(MolecularChain::from_number(0.0));
                        }

                        // ── Phase 4 B10: Test framework builtins ────────────
                        "__panic" => {
                            let msg_chain = vm_pop!(stack, events);
                            let msg = chain_to_string(&msg_chain)
                                .unwrap_or_else(|| format_chain_display(&msg_chain));
                            events.push(VmEvent::Error(VmError::RuntimeError(
                                alloc::format!("panic: {}", msg)
                            )));
                            break;
                        }
                        "__assert_eq" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            if a.chain_hash() != b.chain_hash() {
                                let a_str = format_chain_display(&a);
                                let b_str = format_chain_display(&b);
                                events.push(VmEvent::Error(VmError::RuntimeError(
                                    alloc::format!("assert_eq failed: {} != {}", a_str, b_str)
                                )));
                                break;
                            }
                            let _ = stack.push(MolecularChain::from_number(1.0));
                        }
                        "__assert_ne" => {
                            let b = vm_pop!(stack, events);
                            let a = vm_pop!(stack, events);
                            if a.chain_hash() == b.chain_hash() {
                                let a_str = format_chain_display(&a);
                                events.push(VmEvent::Error(VmError::RuntimeError(
                                    alloc::format!("assert_ne failed: both are {}", a_str)
                                )));
                                break;
                            }
                            let _ = stack.push(MolecularChain::from_number(1.0));
                        }
                        "__assert_true" => {
                            let val = vm_pop!(stack, events);
                            if val.is_empty() {
                                events.push(VmEvent::Error(VmError::RuntimeError(
                                    "assert_true failed: value is empty/falsy".into()
                                )));
                                break;
                            }
                            let _ = stack.push(MolecularChain::from_number(1.0));
                        }

                        _ => {
                            // Unknown function → emit lookup event
                            events.push(VmEvent::LookupAlias(name.clone()));
                        }
                    }
                }

                Op::Ret => {
                    if let Some((saved_pc, saved_scope_depth, saved_stack_depth, param_count)) = closure_call_stack.pop() {
                        let ret_val = stack.pop().unwrap_or_else(|_| MolecularChain::empty());

                        // Pop body scopes back to CallClosure's scope level
                        while scopes.len() > saved_scope_depth {
                            scopes.pop();
                            call_depth = call_depth.saturating_sub(1);
                        }

                        // Write-back: copy parameter values that are heap refs (dicts/arrays)
                        // back to the IMMEDIATE caller scope only.
                        // This propagates struct mutations (e.g., p.pos = p.pos + 1).
                        //
                        // IMPORTANT: Only write back heap refs (dict/array), not primitives.
                        // Only write to the immediate caller scope to avoid corrupting
                        // variables in outer scopes that happen to share the same name.
                        // Heap-based dict/array mutations already propagate automatically
                        // through shared references, so this write-back handles the case
                        // where the parameter variable itself was reassigned.
                        if saved_scope_depth >= 2 && param_count > 0 {
                            let fn_scope_idx = saved_scope_depth - 1;
                            let caller_scope_idx = fn_scope_idx.saturating_sub(1);
                            if fn_scope_idx < scopes.len() {
                                let write_count = param_count.min(scopes[fn_scope_idx].len());
                                let params_to_write: Vec<(String, MolecularChain)> =
                                    scopes[fn_scope_idx][..write_count].to_vec();
                                for (pname, val) in &params_to_write {
                                    // Only write back heap refs (dict/array) to avoid
                                    // corrupting caller variables that share names with
                                    // function locals (e.g., match bindings named 'name').
                                    if as_dict_ref(val).is_none() && as_array_ref(val).is_none() {
                                        continue;
                                    }
                                    if let Some(entry) = scopes[caller_scope_idx].iter_mut().rev()
                                        .find(|(n, _)| n == pname) {
                                        entry.1 = val.clone();
                                    }
                                }
                            }
                        }

                        // Pop the function's param scope
                        if scopes.len() == saved_scope_depth {
                            scopes.pop();
                        }

                        // Restore stack depth
                        while stack.data.len() > saved_stack_depth {
                            let _ = stack.pop();
                        }
                        let _ = stack.push(ret_val);
                        pc = saved_pc;
                    } else {
                        break;
                    }
                }

                Op::Dream => {
                    events.push(VmEvent::TriggerDream);
                }

                Op::Stats => {
                    events.push(VmEvent::RequestStats);
                }

                Op::Store(name) => {
                    let val = vm_pop!(stack, events);
                    // Store in current (innermost) scope.
                    // Update existing in current scope, else insert new.
                    // SAFETY: scopes always has root scope (initialized above)
                    let Some(scope) = scopes.last_mut() else { break };
                    if let Some(entry) = scope.iter_mut().find(|(n, _)| n == name) {
                        entry.1 = val;
                    } else {
                        scope.push((name.clone(), val));
                    }
                }

                Op::StoreUpdate(name) => {
                    let val = vm_pop!(stack, events);
                    // Search ALL scopes from innermost outward; update first match.
                    // If not found anywhere, store in current scope (fallback).
                    let mut found = false;
                    for scope in scopes.iter_mut().rev() {
                        if let Some(entry) = scope.iter_mut().find(|(n, _)| n == name) {
                            entry.1 = val.clone();
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        if let Some(scope) = scopes.last_mut() {
                            scope.push((name.clone(), val));
                        }
                    }
                }

                Op::LoadLocal(name) => {
                    // Search from innermost scope outward (lexical scoping)
                    let val = scopes
                        .iter()
                        .rev()
                        .find_map(|scope| {
                            scope.iter().rev().find(|(n, _)| n == name).map(|(_, c)| c.clone())
                        })
                        .unwrap_or_else(MolecularChain::empty);
                    if let Err(e) = stack.push(val) {
                        events.push(VmEvent::Error(e));
                        break;
                    }
                }

                Op::ScopeBegin => {
                    call_depth += 1;
                    if call_depth > self.max_call_depth {
                        events.push(VmEvent::Error(VmError::MaxCallDepthExceeded));
                        break;
                    }
                    scopes.push(Vec::new());
                }

                Op::ScopeEnd => {
                    // Pop innermost scope (discard locals)
                    // Never pop root scope
                    if scopes.len() > 1 {
                        scopes.pop();
                        call_depth = call_depth.saturating_sub(1);
                    }
                    // Check loop stack: if we're at end of a loop body, jump back
                    if let Some(entry) = loop_stack.last_mut() {
                        if entry.1 > 0 {
                            entry.1 -= 1;
                            pc = entry.0; // jump back to loop body start
                        } else {
                            loop_stack.pop();
                        }
                    }
                }

                Op::Fuse => {
                    // QT2: ∞ là sai — ∞-1 mới đúng
                    // Pop chain, check nó hữu hạn (không có self-reference loop).
                    // Nếu chain hữu hạn → push lại (∞-1 = đúng).
                    // Nếu chain rỗng hoặc bất thường → push empty (∞ = sai).
                    let chain = vm_pop!(stack, events);
                    // Finite check: chain must have bounded length
                    // (MolecularChain is always finite by construction,
                    //  but FUSE ensures no runtime-generated infinite loops)
                    if chain.is_empty() {
                        // ∞ = sai → empty
                        let _ = stack.push(MolecularChain::empty());
                    } else {
                        // ∞-1 = đúng → push back
                        let _ = stack.push(chain);
                    }
                }

                Op::Trace => {
                    // Toggle tracing: emit a trace event for current state
                    events.push(VmEvent::TraceStep {
                        op_name: "TRACE",
                        stack_depth: stack.data.len(),
                        pc: pc - 1,
                    });
                }

                Op::Inspect => {
                    let chain = vm_pop!(stack, events);
                    let bytes = chain.to_bytes();
                    events.push(VmEvent::InspectChain {
                        hash: chain.chain_hash(),
                        molecule_count: chain.len(),
                        byte_size: bytes.len(),
                        is_empty: chain.is_empty(),
                    });
                    // Push back so chain is still available
                    let _ = stack.push(chain);
                }

                Op::Assert => {
                    let chain = vm_pop!(stack, events);
                    if chain.is_empty() {
                        events.push(VmEvent::AssertFailed);
                    }
                    // Push back regardless
                    let _ = stack.push(chain);
                }

                Op::TypeOf => {
                    let chain = vm_pop!(stack, events);
                    // Classify based on molecule bytes
                    let classification = classify_chain(&chain);
                    events.push(VmEvent::TypeInfo {
                        hash: chain.chain_hash(),
                        classification,
                    });
                    let _ = stack.push(chain);
                }

                Op::Why => {
                    let b = vm_pop!(stack, events);
                    let a = vm_pop!(stack, events);
                    events.push(VmEvent::WhyConnection {
                        from: a.chain_hash(),
                        to: b.chain_hash(),
                    });
                    // Push LCA as result (their common ancestor = their connection)
                    let result = if a.is_empty() || b.is_empty() {
                        if !a.is_empty() {
                            a
                        } else {
                            b
                        }
                    } else {
                        lca(&a, &b)
                    };
                    let _ = stack.push(result);
                }

                Op::Explain => {
                    let chain = vm_pop!(stack, events);
                    events.push(VmEvent::ExplainOrigin {
                        hash: chain.chain_hash(),
                    });
                    let _ = stack.push(chain);
                }

                // ── Device I/O opcodes ─────────────────────────────────────────
                // VM = side-effect free. Emit events → Runtime xử lý → HAL.
                // Đây là bridge: Olang → VmEvent → Runtime → HAL → phần cứng.

                Op::DeviceWrite(device_id) => {
                    let val_chain = vm_pop!(stack, events);
                    // Extract value: nếu là number chain → u8, nếu là mol → valence
                    let value = if let Some(n) = val_chain.to_number() {
                        n as u8
                    } else if let Some(mol) = val_chain.first() {
                        mol.valence_u8()
                    } else {
                        0
                    };
                    events.push(VmEvent::DeviceWrite {
                        device_id: device_id.clone(),
                        value,
                    });
                }

                Op::DeviceRead(device_id) => {
                    // Emit event — Runtime sẽ gọi HAL.device_read() và inject kết quả.
                    // Tạm push empty chain → Runtime sẽ replace.
                    events.push(VmEvent::DeviceRead {
                        device_id: device_id.clone(),
                    });
                    // Push placeholder — caller (Runtime) có thể inject actual value
                    let _ = stack.push(MolecularChain::empty());
                }

                Op::DeviceList => {
                    events.push(VmEvent::DeviceListRequest);
                }

                // ── FFI & System I/O ──────────────────────────────────────────

                Op::Ffi(name, arity) => {
                    let mut args = Vec::new();
                    for _ in 0..*arity {
                        args.push(vm_pop!(stack, events));
                    }
                    args.reverse(); // stack order → call order
                    events.push(VmEvent::FfiCall {
                        name: name.clone(),
                        args,
                    });
                    // Push placeholder — Runtime injects actual result
                    let _ = stack.push(MolecularChain::empty());
                }

                Op::FileRead => {
                    let path_chain = vm_pop!(stack, events);
                    let path = chain_to_string(&path_chain)
                        .unwrap_or_default();
                    events.push(VmEvent::FileReadRequest { path });
                    // Push placeholder — Runtime injects file contents
                    let _ = stack.push(MolecularChain::empty());
                }

                Op::FileWrite => {
                    let data_chain = vm_pop!(stack, events);
                    let path_chain = vm_pop!(stack, events);
                    let path = chain_to_string(&path_chain)
                        .unwrap_or_default();
                    let data = if let Some(s) = chain_to_string(&data_chain) {
                        s.into_bytes()
                    } else {
                        data_chain.to_tagged_bytes()
                    };
                    events.push(VmEvent::FileWriteRequest { path, data });
                }

                Op::FileAppend => {
                    let data_chain = vm_pop!(stack, events);
                    let path_chain = vm_pop!(stack, events);
                    let path = chain_to_string(&path_chain)
                        .unwrap_or_default();
                    let data = if let Some(s) = chain_to_string(&data_chain) {
                        s.into_bytes()
                    } else {
                        data_chain.to_tagged_bytes()
                    };
                    events.push(VmEvent::FileAppendRequest { path, data });
                }

                Op::SpawnBegin => {
                    // Count opcodes until SpawnEnd
                    let mut count = 0usize;
                    let mut search_pc = pc;
                    while search_pc < prog.ops.len() {
                        if matches!(prog.ops[search_pc], Op::SpawnEnd) {
                            break;
                        }
                        count += 1;
                        search_pc += 1;
                    }
                    events.push(VmEvent::SpawnRequest {
                        body_ops_count: count,
                    });
                    // Skip past SpawnEnd — the body will be executed by Runtime as async
                    pc = search_pc + 1;
                    continue;
                }

                Op::SpawnEnd => {
                    // Should not be reached — SpawnBegin skips past SpawnEnd
                }

                Op::TryBegin(catch_target) => {
                    // Push catch handler PC onto try stack
                    try_stack.push(*catch_target);
                }

                Op::CatchEnd => {
                    // End of catch block — pop try entry if still present
                    // (already popped if error occurred and caught)
                }

                Op::Closure(_param_count, body_len) => {
                    // Create closure: jump over body, push closure marker.
                    // Closure marker uses make_closure_marker() which stores body_pc
                    // losslessly in raw u16 slots (not quantized molecule fields).
                    let body_pc = pc;
                    let marker = make_closure_marker(*_param_count, body_pc);
                    let _ = stack.push(marker);
                    // Jump past the body
                    pc += body_len;
                }

                Op::CallClosure(arity) => {
                    // Call a closure: stack has [closure, arg1, arg2, ...]
                    let arity_val = *arity as usize;
                    // Pop args first (they're on top)
                    let mut closure_args = Vec::new();
                    for _ in 0..arity_val {
                        closure_args.push(vm_pop!(stack, events));
                    }
                    closure_args.reverse();
                    // Pop closure marker
                    let closure = vm_pop!(stack, events);
                    // Extract body_pc from closure marker (lossless encoding)
                    if let Some(body_pc) = closure_body_pc(&closure) {
                            // Save stack depth BEFORE pushing args back — this is the caller's
                            // clean stack depth. Ret will restore to this depth.
                            let caller_stack_depth = stack.data.len();
                            // Push args back so body can Store them.
                            // Args are in order [arg0, arg1, ...]. Body does Store in
                            // params.iter().rev() order, so we need arg_last on top.
                            // Push in forward order: arg0 first, arg_last last (on top).
                            for arg in closure_args.into_iter() {
                                let _ = stack.push(arg);
                            }
                            // Push new scope for function params and body.
                            scopes.push(Vec::new());
                            // Save current PC, scope depth, and caller stack depth on call stack,
                            // then jump to body.
                            closure_call_stack.push((pc, scopes.len(), caller_stack_depth, arity_val));
                            pc = body_pc;
                    } else {
                        let _ = stack.push(MolecularChain::empty());
                    }
                }

// ── First-class channel opcodes ──────────────────────
                Op::ChanNew => {
                    let id = next_channel_id;
                    next_channel_id += 1;
                    channels.push(Vec::new());
                    let _ = stack.push(MolecularChain::from_number(id as f64));
                }

                Op::ChanSend => {
                    // Stack: [channel_id, value] → send value into channel
                    let value = vm_pop!(stack, events);
                    let ch_chain = vm_pop!(stack, events);
                    let ch_id = ch_chain.to_number().unwrap_or(0.0) as usize;
                    if ch_id >= 1 && ch_id <= channels.len() {
                        channels[ch_id - 1].push(value);
                    }
                    let _ = stack.push(MolecularChain::from_number(1.0));
                }

                Op::ChanRecv => {
                    // Stack: [channel_id] → pop first message or empty
                    let ch_chain = vm_pop!(stack, events);
                    let ch_id = ch_chain.to_number().unwrap_or(0.0) as usize;
                    if ch_id >= 1 && ch_id <= channels.len() && !channels[ch_id - 1].is_empty() {
                        let msg = channels[ch_id - 1].remove(0);
                        let _ = stack.push(msg);
                    } else {
                        let _ = stack.push(MolecularChain::empty());
                    }
                }

                Op::Select(_arm_count) => {
                    // Select: cooperative channel multiplexing.
                    // The Select opcode itself is a marker — the lowered code
                    // after it contains ChanRecv + body for each arm sequentially.
                    // In cooperative (non-preemptive) mode, we just let the
                    // sequential arms execute. The VM records the arm_count as
                    // metadata for future preemptive scheduling.
                }

                Op::Halt => {
                    break;
                }
            }

            // Phase 5 A10: Handle EarlyReturn from ? operator
            // ? on Err/None → early return (same as Ret: break current execution)
            if events.iter().any(|e| matches!(e, VmEvent::EarlyReturn)) {
                events.retain(|e| !matches!(e, VmEvent::EarlyReturn));
                break;
            }
        }

        // Check: if we broke due to error and have a try handler, resume there
        if !try_stack.is_empty() {
            // Check if last event was an error
            let has_error = events.iter().rev().any(|e| matches!(e, VmEvent::Error(_)));
            if has_error {
                if let Some(catch_pc) = try_stack.pop() {
                    // Remove the error event (caught)
                    if let Some(pos) = events.iter().rposition(|e| matches!(e, VmEvent::Error(_))) {
                        events.remove(pos);
                    }
                    // Resume at catch handler — re-enter execution loop
                    pc = catch_pc;
                    let mut remaining_steps = self.max_steps.saturating_sub(steps);
                    while pc < prog.ops.len() && remaining_steps > 0 {
                        remaining_steps -= 1;
                        steps += 1;
                        let op = &prog.ops[pc];
                        pc += 1;
                        match op {
                            Op::Halt | Op::CatchEnd => break,
                            Op::Emit => {
                                if let Ok(c) = stack.pop() {
                                    events.push(VmEvent::Output(c));
                                }
                            }
                            Op::Dream => events.push(VmEvent::TriggerDream),
                            Op::Stats => events.push(VmEvent::RequestStats),
                            Op::Load(name) => events.push(VmEvent::LookupAlias(name.clone())),
                            Op::PushNum(n) => {
                                let _ = stack.push(MolecularChain::from_number(*n));
                            }
                            Op::Push(chain) => {
                                let _ = stack.push(chain.clone());
                            }
                            Op::PushMol(bits) => {
                                let chain = MolecularChain(alloc::vec![*bits]);
                                let _ = stack.push(chain);
                            }
                            Op::Dup => {
                                if let Some(top) = stack.data.last() {
                                    let c = top.clone();
                                    let _ = stack.push(c);
                                }
                            }
                            Op::Store(name) => {
                                if let Ok(c) = stack.pop() {
                                    if let Some(scope) = scopes.last_mut() {
                                        if let Some(entry) = scope.iter_mut().find(|(n, _)| n == name) {
                                            entry.1 = c;
                                        } else {
                                            scope.push((name.clone(), c));
                                        }
                                    }
                                }
                            }
                            Op::LoadLocal(name) => {
                                for scope in scopes.iter().rev() {
                                    if let Some((_, c)) = scope.iter().find(|(n, _)| n == name) {
                                        let _ = stack.push(c.clone());
                                        break;
                                    }
                                }
                            }
                            Op::StoreUpdate(name) => {
                                if let Ok(c) = stack.pop() {
                                    let mut found = false;
                                    for scope in scopes.iter_mut().rev() {
                                        if let Some(entry) = scope.iter_mut().find(|(n, _)| n == name) {
                                            entry.1 = c.clone();
                                            found = true;
                                            break;
                                        }
                                    }
                                    if !found {
                                        if let Some(scope) = scopes.last_mut() {
                                            scope.push((name.clone(), c));
                                        }
                                    }
                                }
                            }
                            Op::Nop | Op::ScopeBegin | Op::ScopeEnd => {}
                            Op::Pop => { let _ = stack.pop(); }
                            _ => {} // other ops skipped in catch recovery
                        }
                    }
                }
            }
        }

        VmResult {
            events,
            steps,
            stack_depth: stack.data.len(),
        }
    }
}

impl Default for OlangVM {
    fn default() -> Self {
        Self::new()
    }
}

/// Kết quả execute.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct VmResult {
    pub events: Vec<VmEvent>,
    pub steps: u32,
    pub stack_depth: usize,
}

#[allow(missing_docs)]
impl VmResult {
    pub fn outputs(&self) -> Vec<&MolecularChain> {
        self.events
            .iter()
            .filter_map(|e| {
                if let VmEvent::Output(c) = e {
                    Some(c)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn has_error(&self) -> bool {
        self.events.iter().any(|e| matches!(e, VmEvent::Error(_)))
    }

    pub fn errors(&self) -> Vec<&VmError> {
        self.events
            .iter()
            .filter_map(|e| {
                if let VmEvent::Error(err) = e {
                    Some(err)
                } else {
                    None
                }
            })
            .collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(missing_docs)]
pub mod tests {
    use super::*;
    use crate::encoder::encode_codepoint;
    use crate::ir::{compile_expr, OlangIrExpr};

    /// Test helper: split array chain for assertions in other test modules.
    pub fn split_test_array(chain: &MolecularChain) -> Vec<MolecularChain> {
        split_array_chain(chain)
    }

    fn vm() -> OlangVM {
        OlangVM::new()
    }

    // ── Basic execution ──────────────────────────────────────────────────────

    #[test]
    fn execute_halt_immediately() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert_eq!(result.steps, 1);
        assert!(!result.has_error());
    }

    #[test]
    fn execute_nop() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Nop).push_op(Op::Nop).push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert_eq!(result.steps, 3);
    }

    #[test]
    fn execute_push_emit() {
        let chain = encode_codepoint(0x1F525); // 🔥
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(chain.clone()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1);
        assert_eq!(*outputs[0], chain);
    }

    #[test]
    fn execute_lca() {
        let fire = encode_codepoint(0x1F525);
        let water = encode_codepoint(0x1F4A7);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(fire))
            .push_op(Op::Push(water))
            .push_op(Op::Lca)
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1);
        assert!(!outputs[0].is_empty(), "LCA output không rỗng");
    }

    #[test]
    fn execute_dup() {
        let chain = encode_codepoint(0x1F525);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(chain))
            .push_op(Op::Dup)
            .push_op(Op::Emit) // emit copy
            .push_op(Op::Emit) // emit original
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert_eq!(result.outputs().len(), 2, "DUP → 2 outputs");
    }

    #[test]
    fn execute_swap() {
        let fire = encode_codepoint(0x1F525);
        let water = encode_codepoint(0x1F4A7);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(fire.clone()))
            .push_op(Op::Push(water.clone()))
            .push_op(Op::Swap)
            .push_op(Op::Emit) // emit fire (sau swap)
            .push_op(Op::Emit) // emit water
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert_eq!(result.outputs().len(), 2);
        // Sau swap: stack = [fire, water] → emit water trước, fire sau?
        // Không — swap đổi top 2: push fire, push water → swap → [water, fire]
        // Emit → fire (top), emit → water
        assert_eq!(*result.outputs()[0], fire);
    }

    #[test]
    fn execute_load_emits_event() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Load("fire".into())).push_op(Op::Halt);
        let result = vm().execute(&prog);
        let has_lookup = result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::LookupAlias(s) if s == "fire"));
        assert!(has_lookup, "LOAD phải emit LookupAlias event");
    }

    #[test]
    fn execute_edge_emits_event() {
        let fire = encode_codepoint(0x1F525);
        let water = encode_codepoint(0x1F4A7);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(fire))
            .push_op(Op::Push(water))
            .push_op(Op::Edge(0x06)) // Causes relation
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let has_edge = result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::CreateEdge { rel: 0x06, .. }));
        assert!(has_edge, "EDGE phải emit CreateEdge event");
    }

    #[test]
    fn execute_dream_event() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Dream).push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::TriggerDream)));
    }

    #[test]
    fn execute_stats_event() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Stats).push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::RequestStats)));
    }

    // ── Control flow ─────────────────────────────────────────────────────────

    #[test]
    fn execute_jmp() {
        let chain = encode_codepoint(0x1F525);
        // JMP 2 → skip PUSH → chỉ HALT
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Jmp(2)) // 0: jump to 2
            .push_op(Op::Push(chain.clone())) // 1: SKIP
            .push_op(Op::Halt); // 2: halt
        let result = vm().execute(&prog);
        assert_eq!(result.outputs().len(), 0, "JMP phải skip PUSH");
    }

    #[test]
    fn execute_max_steps_guard() {
        // Vòng lặp vô hạn → bị chặn bởi max_steps
        let vm = OlangVM::with_max_steps(10);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Jmp(0)); // infinite loop
        let result = vm.execute(&prog);
        assert!(result.has_error(), "Infinite loop phải bị chặn");
        assert!(result.errors().contains(&&VmError::MaxStepsExceeded));
    }

    #[test]
    fn execute_stack_underflow() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Pop) // pop khi stack rỗng
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result.has_error(), "Pop khi rỗng → error");
    }

    #[test]
    fn execute_invalid_jump() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Jmp(9999)) // target không tồn tại
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result.has_error());
    }

    // ── Compile + Execute ────────────────────────────────────────────────────

    #[test]
    fn compile_and_execute_query() {
        let expr = OlangIrExpr::Query("fire".into());
        let prog = compile_expr(&expr);
        let result = vm().execute(&prog);
        // LOAD → LookupAlias event
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::LookupAlias(s) if s == "fire")));
    }

    #[test]
    fn compile_and_execute_compose() {
        let expr = OlangIrExpr::Compose("fire".into(), "water".into());
        let prog = compile_expr(&expr);
        // Phải có: LOAD fire, LOAD water, LCA, EMIT, HALT
        assert_eq!(prog.ops.len(), 5);
        let result = vm().execute(&prog);
        // 2 LookupAlias events (fire, water)
        let lookups: Vec<_> = result
            .events
            .iter()
            .filter_map(|e| {
                if let VmEvent::LookupAlias(s) = e {
                    Some(s.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert!(lookups.contains(&"fire"));
        assert!(lookups.contains(&"water"));
    }

    #[test]
    fn compile_and_execute_dream() {
        let expr = OlangIrExpr::Command("dream".into());
        let prog = compile_expr(&expr);
        let result = vm().execute(&prog);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::TriggerDream)));
    }

    // ── Reasoning & Debug primitives ────────────────────────────────────────

    #[test]
    fn execute_trace() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Trace).push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::TraceStep { .. })));
    }

    #[test]
    fn execute_inspect() {
        let chain = encode_codepoint(0x1F525); // 🔥
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(chain))
            .push_op(Op::Inspect)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let has_inspect = result.events.iter().any(|e| {
            matches!(
                e,
                VmEvent::InspectChain {
                    is_empty: false,
                    ..
                }
            )
        });
        assert!(has_inspect, "INSPECT phải emit InspectChain event");
        // Chain should still be on stack (inspect doesn't consume)
        assert_eq!(result.stack_depth, 1);
    }

    #[test]
    fn execute_assert_pass() {
        let chain = encode_codepoint(0x1F525);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(chain))
            .push_op(Op::Assert)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        // No AssertFailed since chain is non-empty
        assert!(!result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::AssertFailed)));
    }

    #[test]
    fn execute_assert_fail() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(MolecularChain::empty()))
            .push_op(Op::Assert)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::AssertFailed)));
    }

    #[test]
    fn execute_typeof() {
        let chain = encode_codepoint(0x1F525);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(chain))
            .push_op(Op::TypeOf)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::TypeInfo { .. })));
    }

    #[test]
    fn execute_explain() {
        let chain = encode_codepoint(0x1F525);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(chain))
            .push_op(Op::Explain)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::ExplainOrigin { .. })));
        // Chain still on stack
        assert_eq!(result.stack_depth, 1);
    }

    #[test]
    fn execute_why() {
        let fire = encode_codepoint(0x1F525);
        let water = encode_codepoint(0x1F4A7);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(fire))
            .push_op(Op::Push(water))
            .push_op(Op::Why)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, VmEvent::WhyConnection { .. })));
        // LCA result on stack
        assert_eq!(result.stack_depth, 1);
    }

    // ── Phase 1: Math Runtime — arithmetic execution ────────────────────────

    #[test]
    fn math_addition() {
        // 1 + 2 = 3
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(1.0))
            .push_op(Op::PushNum(2.0))
            .push_op(Op::Call("__hyp_add".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1);
        let n = outputs[0].to_number().unwrap();
        assert!((n - 3.0).abs() < f64::EPSILON, "1 + 2 should = 3, got {}", n);
    }

    #[test]
    fn math_subtraction() {
        // 10 - 3 = 7
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(10.0))
            .push_op(Op::PushNum(3.0))
            .push_op(Op::Call("__hyp_sub".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn math_multiplication() {
        // 6 × 7 = 42
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(6.0))
            .push_op(Op::PushNum(7.0))
            .push_op(Op::Call("__hyp_mul".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn math_division() {
        // 10 ÷ 4 = 2.5
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(10.0))
            .push_op(Op::PushNum(4.0))
            .push_op(Op::Call("__hyp_div".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn math_division_by_zero() {
        // 5 ÷ 0 → error
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(5.0))
            .push_op(Op::PushNum(0.0))
            .push_op(Op::Call("__hyp_div".into()))
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result.has_error(), "Division by zero should error");
        assert!(
            result.errors().contains(&&VmError::DivisionByZero),
            "Should be DivisionByZero, not StackUnderflow"
        );
    }

    #[test]
    fn math_physical_add() {
        // Physical add: same as hyp_add but for proven values
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(100.0))
            .push_op(Op::PushNum(50.0))
            .push_op(Op::Call("__phys_add".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn math_truth_equal() {
        // fire == fire → truthy (push back)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(42.0))
            .push_op(Op::PushNum(42.0))
            .push_op(Op::Call("__assert_truth".into()))
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        // Stack should have non-empty chain (truthy)
        assert_eq!(result.stack_depth, 1);
    }

    #[test]
    fn math_truth_not_equal() {
        // 1 == 2 → falsy (empty chain)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(1.0))
            .push_op(Op::PushNum(2.0))
            .push_op(Op::Call("__assert_truth".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1);
        assert!(outputs[0].is_empty(), "1 == 2 should be falsy (empty chain)");
    }

    #[test]
    fn math_chained_operations() {
        // (2 + 3) * 4 = 20
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(2.0))
            .push_op(Op::PushNum(3.0))
            .push_op(Op::Call("__hyp_add".into()))
            .push_op(Op::PushNum(4.0))
            .push_op(Op::Call("__hyp_mul".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 20.0).abs() < f64::EPSILON, "(2+3)*4 = {}", n);
    }

    #[test]
    fn math_negative_result() {
        // 3 - 7 = -4
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(3.0))
            .push_op(Op::PushNum(7.0))
            .push_op(Op::Call("__hyp_sub".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - (-4.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn math_pushnum_roundtrip() {
        // PushNum → Emit → to_number roundtrip
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(3.14159))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 3.14159).abs() < 1e-10);
    }

    // ── Scope tests ─────────────────────────────────────────────────────────

    #[test]
    fn scope_basic_store_load() {
        // Store x=5 in root scope, load x → emit
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(5.0))
            .push_op(Op::Store("x".into()))
            .push_op(Op::LoadLocal("x".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn scope_nested_shadows_outer() {
        // Root: x = 10
        // Inner scope: x = 20, emit x (should be 20)
        // After scope end: emit x (should be 10 again)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(10.0))
            .push_op(Op::Store("x".into()))
            .push_op(Op::ScopeBegin)
            .push_op(Op::PushNum(20.0))
            .push_op(Op::Store("x".into()))
            .push_op(Op::LoadLocal("x".into()))
            .push_op(Op::Emit) // should output 20
            .push_op(Op::ScopeEnd)
            .push_op(Op::LoadLocal("x".into()))
            .push_op(Op::Emit) // should output 10 (inner x discarded)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 2);
        let inner = outputs[0].to_number().unwrap();
        let outer = outputs[1].to_number().unwrap();
        assert!((inner - 20.0).abs() < f64::EPSILON, "Inner scope x=20: {}", inner);
        assert!((outer - 10.0).abs() < f64::EPSILON, "Outer scope x=10: {}", outer);
    }

    #[test]
    fn scope_inner_reads_outer() {
        // Root: y = 42
        // Inner scope: load y (should find in outer scope)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(42.0))
            .push_op(Op::Store("y".into()))
            .push_op(Op::ScopeBegin)
            .push_op(Op::LoadLocal("y".into()))
            .push_op(Op::Emit)
            .push_op(Op::ScopeEnd)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 42.0).abs() < f64::EPSILON, "Inner reads outer y=42");
    }

    #[test]
    fn scope_double_nesting() {
        // Root: a = 1
        // Scope 1: b = 2
        // Scope 2: c = 3, emit a+b+c
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(1.0))
            .push_op(Op::Store("a".into()))
            .push_op(Op::ScopeBegin) // scope 1
            .push_op(Op::PushNum(2.0))
            .push_op(Op::Store("b".into()))
            .push_op(Op::ScopeBegin) // scope 2
            .push_op(Op::PushNum(3.0))
            .push_op(Op::Store("c".into()))
            // a + b
            .push_op(Op::LoadLocal("a".into()))
            .push_op(Op::LoadLocal("b".into()))
            .push_op(Op::Call("__hyp_add".into()))
            // + c
            .push_op(Op::LoadLocal("c".into()))
            .push_op(Op::Call("__hyp_add".into()))
            .push_op(Op::Emit) // 1+2+3 = 6
            .push_op(Op::ScopeEnd) // pop scope 2
            .push_op(Op::ScopeEnd) // pop scope 1
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let n = result.outputs()[0].to_number().unwrap();
        assert!((n - 6.0).abs() < f64::EPSILON, "a+b+c = 6: {}", n);
    }

    #[test]
    fn scope_end_without_begin_safe() {
        // ScopeEnd without Begin should not crash (root scope protected)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::ScopeEnd)
            .push_op(Op::ScopeEnd)
            .push_op(Op::PushNum(1.0))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert_eq!(result.outputs().len(), 1);
    }

    #[test]
    fn scope_undefined_var_returns_empty() {
        // Loading undefined variable → empty chain
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::LoadLocal("nonexistent".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result.outputs()[0].is_empty(), "Undefined var → empty");
    }

    // ── Recursion depth limit ───────────────────────────────────────────────

    #[test]
    fn call_depth_exceeded_triggers_error() {
        // Build program with deeply nested ScopeBegin without ScopeEnd
        let mut prog = OlangProgram::new("test");
        for _ in 0..520 {
            prog.push_op(Op::ScopeBegin);
        }
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result.has_error(), "Should error on depth > 512");
        let has_depth_err = result.events.iter().any(|e| {
            matches!(e, VmEvent::Error(VmError::MaxCallDepthExceeded))
        });
        assert!(has_depth_err, "Should have MaxCallDepthExceeded error");
    }

    #[test]
    fn call_depth_within_limit_ok() {
        // 10 nested scopes should be fine
        let mut prog = OlangProgram::new("test");
        for _ in 0..10 {
            prog.push_op(Op::ScopeBegin);
        }
        prog.push_op(Op::PushNum(42.0));
        prog.push_op(Op::Emit);
        for _ in 0..10 {
            prog.push_op(Op::ScopeEnd);
        }
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "10 nested scopes should work");
        assert_eq!(result.outputs().len(), 1);
    }

    #[test]
    fn call_depth_decrements_on_scope_end() {
        // Open 5 scopes, close 5, open 5 more → should be within limit
        let mut prog = OlangProgram::new("test");
        for _ in 0..5 {
            prog.push_op(Op::ScopeBegin);
        }
        for _ in 0..5 {
            prog.push_op(Op::ScopeEnd);
        }
        for _ in 0..5 {
            prog.push_op(Op::ScopeBegin);
        }
        prog.push_op(Op::PushNum(99.0));
        prog.push_op(Op::Emit);
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "Depth should reset after ScopeEnd");
    }

    // ── Loop ──────────────────────────────────────────────────────────────

    #[test]
    fn loop_basic_3_times() {
        // Loop 3 times: emit 1.0 each iteration → 3 outputs
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Loop(3));
        prog.push_op(Op::ScopeBegin);
        prog.push_op(Op::PushNum(1.0));
        prog.push_op(Op::Emit);
        prog.push_op(Op::ScopeEnd);
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "Loop 3 should not error");
        assert_eq!(result.outputs().len(), 3, "3 iterations → 3 outputs");
    }

    #[test]
    fn loop_once_same_as_no_loop() {
        // Loop(1) = execute body once (no repeat)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Loop(1));
        prog.push_op(Op::ScopeBegin);
        prog.push_op(Op::PushNum(42.0));
        prog.push_op(Op::Emit);
        prog.push_op(Op::ScopeEnd);
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert_eq!(result.outputs().len(), 1);
    }

    #[test]
    fn loop_zero_no_body() {
        // Loop(0) = skip body entirely? No — falls through once, no repeat.
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Loop(0));
        prog.push_op(Op::ScopeBegin);
        prog.push_op(Op::PushNum(1.0));
        prog.push_op(Op::Emit);
        prog.push_op(Op::ScopeEnd);
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        // Loop(0).min(1024) = 0, so no push to loop_stack, body runs once (fall-through)
        assert_eq!(result.outputs().len(), 1, "Loop(0) falls through once");
    }

    #[test]
    fn loop_capped_at_1024() {
        // Loop(u32::MAX) should be capped to 1024
        let vm = OlangVM::with_max_steps(65_536);
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Loop(u32::MAX));
        prog.push_op(Op::ScopeBegin);
        prog.push_op(Op::Nop); // lightweight body
        prog.push_op(Op::ScopeEnd);
        prog.push_op(Op::Halt);
        let result = vm.execute(&prog);
        // Should complete without MaxStepsExceeded (1024 * 3 ops = 3072 < 65536)
        assert!(!result.has_error(), "Capped loop should complete");
    }

    // ── PushMol — molecular literal execution ──────────────────────────────

    /// Helper: pack 5 dimensions into u16 for PushMol in tests.
    fn pm(s: u8, r: u8, v: u8, a: u8, t: u8) -> u16 {
        Molecule::pack(s, r, v, a, t).bits
    }

    #[test]
    fn push_mol_creates_chain() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushMol(pm(1, 6, 200, 180, 4)));
        prog.push_op(Op::Emit);
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "PushMol should not error");
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1, "One chain emitted");
        let chain = outputs[0];
        assert_eq!(chain.len(), 1, "Chain has exactly 1 molecule");
        let mol = chain.first().unwrap();
        assert_eq!(mol.shape(), 1 >> 4);
        assert_eq!(mol.relation(), 6 >> 4);
        assert_eq!(mol.valence_u8(), (200 >> 5) << 5);
        assert_eq!(mol.arousal_u8(), (180 >> 5) << 5);
        assert_eq!(mol.time(), 4 >> 6);
    }

    #[test]
    fn push_mol_default_values() {
        // Defaults from semantic: S=1, R=1, V=128, A=128, T=3
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushMol(pm(1, 1, 128, 128, 3)));
        prog.push_op(Op::Emit);
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let mol = result.outputs()[0].first().unwrap();
        assert_eq!(mol.shape(), 1 >> 4);
        assert_eq!(mol.relation(), 1 >> 4);
        assert_eq!(mol.valence_u8(), (128 >> 5) << 5);
        assert_eq!(mol.arousal_u8(), (128 >> 5) << 5);
        assert_eq!(mol.time(), 3 >> 6);
    }

    #[test]
    fn push_mol_then_lca() {
        // Two molecular literals → LCA → single output
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushMol(pm(1, 6, 200, 180, 4)));
        prog.push_op(Op::PushMol(pm(2, 3, 100, 90, 2)));
        prog.push_op(Op::Lca);
        prog.push_op(Op::Emit);
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "LCA of two mol literals should work");
        assert_eq!(result.outputs().len(), 1);
        assert!(!result.outputs()[0].is_empty());
    }

    #[test]
    fn push_mol_dup_and_truth() {
        // PushMol → Dup → Truth (==) → should produce 1 output
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushMol(pm(1, 6, 200, 180, 4)));
        prog.push_op(Op::Dup);
        prog.push_op(Op::Call("__assert_truth".into()));
        prog.push_op(Op::Emit);
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        // Truth of identical chains → non-empty output
        assert_eq!(result.outputs().len(), 1);
        assert!(!result.outputs()[0].is_empty(), "Same chain == same chain should be truthy");
    }

    #[test]
    fn match_mol_same() {
        // __match_mol: compare two identical PushMol chains → truthy
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushMol(pm(1, 6, 200, 180, 4)))
            .push_op(Op::PushMol(pm(1, 6, 200, 180, 4)))
            .push_op(Op::Call("__match_mol".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(!result.outputs()[0].is_empty(), "Same mol should match");
    }

    #[test]
    fn match_mol_different() {
        // __match_mol: different mols → falsy (empty)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushMol(pm(1, 6, 200, 180, 4)))
            .push_op(Op::PushMol(pm(2, 3, 100, 50, 1)))
            .push_op(Op::Call("__match_mol".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(result.outputs()[0].is_empty(), "Different mol should not match");
    }

    #[test]
    fn try_catch_no_error() {
        // try { emit 42 } catch { emit 99 }
        // No error → should output 42, not 99
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::TryBegin(5))  // catch at position 5
            .push_op(Op::PushNum(42.0))
            .push_op(Op::Emit)
            .push_op(Op::Jmp(7))        // skip catch
            .push_op(Op::CatchEnd)       // 4 (should not execute)
            .push_op(Op::PushNum(99.0))  // 5: catch body
            .push_op(Op::Emit)           // 6
            .push_op(Op::Halt);          // 7
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1);
        let n = outputs[0].to_number().unwrap();
        assert!((n - 42.0).abs() < f64::EPSILON, "Should output 42, got {}", n);
    }

    #[test]
    fn try_catch_with_error_recovery() {
        // try { pop (underflow!) } catch { emit 99 }
        // Error in try → should recover and emit from catch
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::TryBegin(4))  // catch at position 4
            .push_op(Op::Pop)           // underflow! → error
            .push_op(Op::Jmp(7))        // skip catch
            .push_op(Op::Halt)          // 3 (fallthrough guard)
            .push_op(Op::PushNum(99.0)) // 4: catch body
            .push_op(Op::Emit)          // 5
            .push_op(Op::CatchEnd)      // 6
            .push_op(Op::Halt);         // 7
        let result = vm().execute(&prog);
        // Error should be caught — no errors in result
        assert!(!result.has_error(), "Error should be caught: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1, "Should have catch output");
        let n = outputs[0].to_number().unwrap();
        assert!((n - 99.0).abs() < f64::EPSILON, "Catch should output 99, got {}", n);
    }

    #[test]
    fn for_in_emits_counter_values() {
        // for i in 0..3 { emit i }
        // Counter on stack, DUP into scoped var each iteration.
        // Should output: 0.0, 1.0, 2.0
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(0.0));          // counter = 0 on stack
        prog.push_op(Op::Loop(3));               // 3 iterations
        prog.push_op(Op::ScopeBegin);
        prog.push_op(Op::Dup);                   // dup counter for body
        prog.push_op(Op::Store("i".into()));     // body can use i
        prog.push_op(Op::LoadLocal("i".into())); // load counter var
        prog.push_op(Op::Emit);                  // emit it
        // Increment counter on stack
        prog.push_op(Op::PushNum(1.0));
        prog.push_op(Op::Call("__hyp_add".into()));
        prog.push_op(Op::ScopeEnd);              // destroys scope, triggers loop
        prog.push_op(Op::Pop);                   // discard counter after loop
        prog.push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "for-in error: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 3, "Should emit 3 values, got {}", outputs.len());
        let v0 = outputs[0].to_number().unwrap();
        let v1 = outputs[1].to_number().unwrap();
        let v2 = outputs[2].to_number().unwrap();
        assert!((v0 - 0.0).abs() < f64::EPSILON, "First should be 0, got {}", v0);
        assert!((v1 - 1.0).abs() < f64::EPSILON, "Second should be 1, got {}", v1);
        assert!((v2 - 2.0).abs() < f64::EPSILON, "Third should be 2, got {}", v2);
    }

    #[test]
    fn cmp_lt_true() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(3.0))
            .push_op(Op::PushNum(5.0))
            .push_op(Op::Call("__cmp_lt".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 1.0).abs() < f64::EPSILON, "3 < 5 should be true (1.0)");
    }

    #[test]
    fn cmp_lt_false() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(7.0))
            .push_op(Op::PushNum(5.0))
            .push_op(Op::Call("__cmp_lt".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        // False comparisons return empty chain (falsy for Jz)
        assert!(result.outputs()[0].is_empty(), "7 < 5 should be false (empty)");
    }

    #[test]
    fn cmp_ge_true() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(5.0))
            .push_op(Op::PushNum(5.0))
            .push_op(Op::Call("__cmp_ge".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 1.0).abs() < f64::EPSILON, "5 >= 5 should be true");
    }

    #[test]
    fn while_loop_counts_to_three() {
        // while i < 3 { emit i; i = i + 1 }
        // Simulate: counter on stack, Loop(1024), cond check via __cmp_lt + Jz
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(0.0));          // counter on stack
        prog.push_op(Op::Loop(1024));
        prog.push_op(Op::ScopeBegin);
        // Check: counter < 3?
        prog.push_op(Op::Dup);                    // dup counter for cmp
        prog.push_op(Op::PushNum(3.0));
        prog.push_op(Op::Call("__cmp_lt".into()));
        let jz_idx = prog.ops.len();
        prog.push_op(Op::Jz(0));                  // placeholder → jumps to end
        prog.push_op(Op::Pop);                    // pop cmp result (true path)
        // Body: emit counter
        prog.push_op(Op::Dup);
        prog.push_op(Op::Emit);
        // Increment
        prog.push_op(Op::PushNum(1.0));
        prog.push_op(Op::Call("__hyp_add".into()));
        prog.push_op(Op::ScopeEnd);               // loop back
        let end = prog.ops.len();
        prog.ops[jz_idx] = Op::Jz(end);
        prog.push_op(Op::Pop);                    // pop cmp result (false path, jumped here)
        prog.push_op(Op::Pop);                    // discard counter
        prog.push_op(Op::Halt);

        let result = vm().execute(&prog);
        assert!(!result.has_error(), "while errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 3, "Should emit 3 values");
        let v0 = outputs[0].to_number().unwrap();
        let v1 = outputs[1].to_number().unwrap();
        let v2 = outputs[2].to_number().unwrap();
        assert!((v0 - 0.0).abs() < f64::EPSILON);
        assert!((v1 - 1.0).abs() < f64::EPSILON);
        assert!((v2 - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cmp_ne_true() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(3.0))
            .push_op(Op::PushNum(5.0))
            .push_op(Op::Call("__cmp_ne".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 1.0).abs() < f64::EPSILON, "3 != 5 should be true");
    }

    #[test]
    fn cmp_ne_false() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(5.0))
            .push_op(Op::PushNum(5.0))
            .push_op(Op::Call("__cmp_ne".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(result.outputs()[0].is_empty(), "5 != 5 should be false");
    }

    #[test]
    fn logic_not_empty_becomes_truthy() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(MolecularChain::empty()))
            .push_op(Op::Call("__logic_not".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 1.0).abs() < f64::EPSILON, "!empty should be truthy (1.0)");
    }

    #[test]
    fn logic_not_truthy_becomes_empty() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(42.0))
            .push_op(Op::Call("__logic_not".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(result.outputs()[0].is_empty(), "!42 should be empty (falsy)");
    }

    // ── StoreUpdate ──────────────────────────────────────────────────────────

    #[test]
    fn store_update_modifies_outer_scope() {
        // Root: x = 10
        // Inner scope: StoreUpdate x = 20, emit x (should be 20)
        // After scope end: emit x (should be 20 — updated in outer scope)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(10.0))
            .push_op(Op::Store("x".into()))
            .push_op(Op::ScopeBegin)
            .push_op(Op::PushNum(20.0))
            .push_op(Op::StoreUpdate("x".into()))
            .push_op(Op::LoadLocal("x".into()))
            .push_op(Op::Emit) // should output 20
            .push_op(Op::ScopeEnd)
            .push_op(Op::LoadLocal("x".into()))
            .push_op(Op::Emit) // should output 20 (outer scope was updated)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "StoreUpdate errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 2);
        let inner = outputs[0].to_number().unwrap();
        let outer = outputs[1].to_number().unwrap();
        assert!((inner - 20.0).abs() < f64::EPSILON, "Inner scope sees 20: {}", inner);
        assert!((outer - 20.0).abs() < f64::EPSILON, "Outer scope updated to 20: {}", outer);
    }

    #[test]
    fn store_update_vs_store_shadowing() {
        // Store shadows (creates new in inner scope), StoreUpdate modifies outer.
        // Root: x = 10
        // Inner scope: Store(x)=99 → shadows outer
        // After scope end: emit x (should be 10, shadow discarded)
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(10.0))
            .push_op(Op::Store("x".into()))
            .push_op(Op::ScopeBegin)
            .push_op(Op::PushNum(99.0))
            .push_op(Op::Store("x".into())) // shadows, not updates outer
            .push_op(Op::ScopeEnd)
            .push_op(Op::LoadLocal("x".into()))
            .push_op(Op::Emit) // should output 10
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let outer = result.outputs()[0].to_number().unwrap();
        assert!((outer - 10.0).abs() < f64::EPSILON, "Store shadows, outer still 10: {}", outer);
    }

    // ── Array builtins ───────────────────────────────────────────────────────

    #[test]
    fn array_new_and_get() {
        // [10, 20, 30] then get index 1
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(10.0))
            .push_op(Op::PushNum(20.0))
            .push_op(Op::PushNum(30.0))
            .push_op(Op::PushNum(3.0))
            .push_op(Op::Call("__array_new".into()))
            .push_op(Op::Store("arr".into()))
            .push_op(Op::LoadLocal("arr".into()))
            .push_op(Op::PushNum(1.0))
            .push_op(Op::Call("__array_get".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "array errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert_eq!(outputs.len(), 1);
        let v = outputs[0].to_number();
        assert!(v.is_some(), "arr[1] should be number, mol_count={}", outputs[0].len());
        assert!((v.unwrap() - 20.0).abs() < f64::EPSILON, "arr[1]=20, got {}", v.unwrap());
    }

    #[test]
    fn array_len() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(10.0))
            .push_op(Op::PushNum(20.0))
            .push_op(Op::PushNum(30.0))
            .push_op(Op::PushNum(3.0))
            .push_op(Op::Call("__array_new".into()))
            .push_op(Op::Call("__array_len".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "errors: {:?}", result.errors());
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 3.0).abs() < f64::EPSILON, "len=3, got {}", v);
    }

    // ── Dict builtins ────────────────────────────────────────────────────────

    fn key_chain(s: &str) -> MolecularChain {
        string_to_chain(s)
    }

    #[test]
    fn dict_new_and_get() {
        // Create dict {key1: 10, key2: 20}, get key2 → 20
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(key_chain("key1")))
            .push_op(Op::PushNum(10.0))
            .push_op(Op::Push(key_chain("key2")))
            .push_op(Op::PushNum(20.0))
            .push_op(Op::PushNum(2.0))
            .push_op(Op::Call("__dict_new".into()))
            .push_op(Op::Store("d".into()))
            // Get key2
            .push_op(Op::LoadLocal("d".into()))
            .push_op(Op::Push(key_chain("key2")))
            .push_op(Op::Call("__dict_get".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "errors: {:?}", result.errors());
        let outputs = result.outputs();
        assert!(!outputs.is_empty(), "Should have output");
        let v = outputs[0].to_number().unwrap();
        assert!((v - 20.0).abs() < f64::EPSILON, "dict.key2 should be 20, got {}", v);
    }

    #[test]
    fn dict_set_updates_value() {
        // Create dict {key1: 10}, set key1 = 99, get key1 → 99
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(key_chain("key1")))
            .push_op(Op::PushNum(10.0))
            .push_op(Op::PushNum(1.0))
            .push_op(Op::Call("__dict_new".into()))
            // Set key1 = 99
            .push_op(Op::Push(key_chain("key1")))
            .push_op(Op::PushNum(99.0))
            .push_op(Op::Call("__dict_set".into()))
            .push_op(Op::Store("d".into()))
            // Get key1
            .push_op(Op::LoadLocal("d".into()))
            .push_op(Op::Push(key_chain("key1")))
            .push_op(Op::Call("__dict_get".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error(), "errors: {:?}", result.errors());
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 99.0).abs() < f64::EPSILON, "dict.key1 should be 99, got {}", v);
    }

    // ── Phase 5: String Builtins ────────────────────────────────────────────

    fn str_chain(s: &str) -> MolecularChain {
        string_to_chain(s)
    }

    #[test]
    fn str_contains_found() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(str_chain("hello world")))
            .push_op(Op::Push(str_chain("world")))
            .push_op(Op::Call("__str_contains".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(!result.outputs()[0].is_empty(), "Should find 'world'");
    }

    #[test]
    fn str_contains_not_found() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(str_chain("hello")))
            .push_op(Op::Push(str_chain("xyz")))
            .push_op(Op::Call("__str_contains".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(result.outputs()[0].is_empty(), "Should not find 'xyz'");
    }

    #[test]
    fn str_starts_with_true() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(str_chain("hello world")))
            .push_op(Op::Push(str_chain("hello")))
            .push_op(Op::Call("__str_starts_with".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.outputs()[0].is_empty());
    }

    #[test]
    fn str_index_of_found() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(str_chain("abcdef")))
            .push_op(Op::Push(str_chain("cd")))
            .push_op(Op::Call("__str_index_of".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 2.0).abs() < f64::EPSILON, "index of 'cd' = 2, got {}", v);
    }

    #[test]
    fn str_replace_basic() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(str_chain("hello world")))
            .push_op(Op::Push(str_chain("world")))
            .push_op(Op::Push(str_chain("olang")))
            .push_op(Op::Call("__str_replace".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        let out = &result.outputs()[0];
        // Result should be "hello olang" encoded as molecules
        let decoded: Vec<u8> = out.0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
        assert_eq!(&decoded, b"hello olang");
    }

    #[test]
    fn str_split_basic() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(str_chain("a,b,c")))
            .push_op(Op::Push(str_chain(",")))
            .push_op(Op::Call("__str_split".into()))
            .push_op(Op::Call("__array_len".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 3.0).abs() < f64::EPSILON, "split 'a,b,c' by ',' = 3 parts, got {}", v);
    }

    #[test]
    fn str_trim_whitespace() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(str_chain("  hello  ")))
            .push_op(Op::Call("__str_trim".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let decoded: Vec<u8> = result.outputs()[0].0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
        assert_eq!(&decoded, b"hello");
    }

    #[test]
    fn str_upper_lower() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(str_chain("Hello")))
            .push_op(Op::Call("__str_upper".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let decoded: Vec<u8> = result.outputs()[0].0.iter().map(|&bits| (bits & 0xFF) as u8).collect();
        assert_eq!(&decoded, b"HELLO");
    }

    // ── Phase 5: Math Builtins ──────────────────────────────────────────────

    #[test]
    fn math_floor() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(3.7))
            .push_op(Op::Call("__hyp_floor".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 3.0).abs() < f64::EPSILON, "floor(3.7)=3, got {}", v);
    }

    #[test]
    fn math_ceil() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(3.2))
            .push_op(Op::Call("__hyp_ceil".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 4.0).abs() < f64::EPSILON, "ceil(3.2)=4, got {}", v);
    }

    #[test]
    fn math_sqrt() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(16.0))
            .push_op(Op::Call("__hyp_sqrt".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 4.0).abs() < 0.01, "sqrt(16)=4, got {}", v);
    }

    #[test]
    fn math_pow() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(2.0))
            .push_op(Op::PushNum(8.0))
            .push_op(Op::Call("__hyp_pow".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 256.0).abs() < f64::EPSILON, "pow(2,8)=256, got {}", v);
    }

    // ── Phase 5: Dict Builtins ──────────────────────────────────────────────

    #[test]
    fn dict_has_key_exists() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(key_chain("x")))
            .push_op(Op::PushNum(10.0))
            .push_op(Op::PushNum(1.0))
            .push_op(Op::Call("__dict_new".into()))
            .push_op(Op::Push(key_chain("x")))
            .push_op(Op::Call("__dict_has_key".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.outputs()[0].is_empty(), "has_key should be truthy");
    }

    #[test]
    fn dict_has_key_missing() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(key_chain("x")))
            .push_op(Op::PushNum(10.0))
            .push_op(Op::PushNum(1.0))
            .push_op(Op::Call("__dict_new".into()))
            .push_op(Op::Push(key_chain("z")))
            .push_op(Op::Call("__dict_has_key".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result.outputs()[0].is_empty(), "has_key for missing key should be falsy");
    }

    #[test]
    fn array_reverse() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(1.0))
            .push_op(Op::PushNum(2.0))
            .push_op(Op::PushNum(3.0))
            .push_op(Op::PushNum(3.0))
            .push_op(Op::Call("__array_new".into()))
            .push_op(Op::Call("__array_reverse".into()))
            // Get first element (should be 3 after reverse)
            .push_op(Op::PushNum(0.0))
            .push_op(Op::Call("__array_get".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        assert!((v - 3.0).abs() < f64::EPSILON, "first after reverse should be 3, got {}", v);
    }

    #[test]
    fn array_contains_found() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(10.0))
            .push_op(Op::PushNum(20.0))
            .push_op(Op::PushNum(30.0))
            .push_op(Op::PushNum(3.0))
            .push_op(Op::Call("__array_new".into()))
            .push_op(Op::PushNum(20.0))
            .push_op(Op::Call("__array_contains".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.outputs()[0].is_empty(), "should contain 20");
    }

    #[test]
    fn type_of_builtin() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(42.0))
            .push_op(Op::Call("__type_of".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(!result.has_error());
        assert!(!result.outputs()[0].is_empty());
    }

    #[test]
    fn chain_len_builtin() {
        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::PushNum(42.0))
            .push_op(Op::Call("__chain_len".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        let v = result.outputs()[0].to_number().unwrap();
        // Number chain has 4 molecules
        assert!(v > 0.0, "chain_len should be > 0, got {}", v);
    }

    // ── Task 3.4: Self-compile test ─────────────────────────────────────

    #[test]
    fn self_compile_parse_lower_encode() {
        // Test __parse → __lower → __encode_bytecode pipeline
        let mut prog = OlangProgram::new("test");
        // Push source: "let x = 42;"
        prog.push_op(Op::Push(key_chain("let x = 42;")))
            .push_op(Op::Call("__parse".into()))
            // Stack: [stmt_count] — should be > 0
            .push_op(Op::Emit)
            .push_op(Op::Call("__lower".into()))
            // Stack: [op_count]
            .push_op(Op::Emit)
            .push_op(Op::Call("__encode_bytecode".into()))
            // Stack: [array_ref of bytecode bytes]
            // Get length via __array_len builtin
            .push_op(Op::Call("__array_len".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);
        let result = vm().execute(&prog);
        assert!(result.outputs().len() >= 3, "should have 3 outputs");

        let parse_count = result.outputs()[0].to_number().unwrap_or(0.0);
        assert!(parse_count > 0.0, "parse should return > 0 stmts, got {}", parse_count);

        let op_count = result.outputs()[1].to_number().unwrap_or(0.0);
        assert!(op_count > 0.0, "lower should return > 0 ops, got {}", op_count);

        let bc_len = result.outputs()[2].to_number().unwrap_or(0.0);
        assert!(bc_len > 0.0, "encode should return > 0 bytes, got {}", bc_len);
    }

    // ── Phase 3 file I/O integration tests (std feature) ─────────────────

    #[cfg(feature = "std")]
    #[test]
    fn file_read_write_roundtrip() {
        // Test __file_write → __file_read roundtrip
        let dir = std::env::temp_dir().join("homeos_test_rw");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test_rw.txt");
        let path_str = path.to_str().unwrap();

        let mut prog = OlangProgram::new("test");
        // Write "hello" to file
        prog.push_op(Op::Push(key_chain(path_str)))
            .push_op(Op::Push(key_chain("hello")))
            .push_op(Op::Call("__file_write".into()))
            .push_op(Op::Emit) // should be 1.0 (success)
            // Read it back
            .push_op(Op::Push(key_chain(path_str)))
            .push_op(Op::Call("__file_read".into()))
            .push_op(Op::Call("__array_len".into()))
            .push_op(Op::Emit) // should be 5 (length of "hello")
            .push_op(Op::Halt);

        let result = vm().execute(&prog);
        let _ = std::fs::remove_dir_all(&dir);

        assert!(result.outputs().len() >= 2, "need 2 outputs");
        let write_ok = result.outputs()[0].to_number().unwrap_or(0.0);
        assert!((write_ok - 1.0).abs() < f64::EPSILON, "write should succeed");
        let read_len = result.outputs()[1].to_number().unwrap_or(0.0);
        assert!((read_len - 5.0).abs() < f64::EPSILON, "read 5 bytes, got {}", read_len);
    }

    #[cfg(feature = "std")]
    #[test]
    fn list_files_in_directory() {
        // Test __list_files returns sorted file list
        let dir = std::env::temp_dir().join("homeos_test_ls");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("alpha.ol"), "// a").unwrap();
        std::fs::write(dir.join("beta.ol"), "// b").unwrap();
        std::fs::write(dir.join("gamma.txt"), "// c").unwrap(); // should be filtered out
        let dir_str = dir.to_str().unwrap();

        let mut prog = OlangProgram::new("test");
        prog.push_op(Op::Push(key_chain(dir_str)))
            .push_op(Op::Push(key_chain(".ol")))
            .push_op(Op::Call("__list_files".into()))
            .push_op(Op::Call("__array_len".into()))
            .push_op(Op::Emit)
            .push_op(Op::Halt);

        let result = vm().execute(&prog);
        let _ = std::fs::remove_dir_all(&dir);

        let count = result.outputs()[0].to_number().unwrap_or(0.0);
        assert!((count - 2.0).abs() < f64::EPSILON, "should find 2 .ol files, got {}", count);
    }

    #[cfg(feature = "std")]
    #[test]
    fn builder_compile_write_roundtrip() {
        // End-to-end: __parse → __lower → __encode_bytecode → __file_write → __file_read → verify
        let dir = std::env::temp_dir().join("homeos_test_builder");
        let _ = std::fs::create_dir_all(&dir);
        let bc_path = dir.join("test.bc");
        let bc_path_str = bc_path.to_str().unwrap();

        let mut prog = OlangProgram::new("test");
        // 1. Compile "let x = 42;"
        prog.push_op(Op::Push(key_chain("let x = 42;")))
            .push_op(Op::Call("__parse".into()))
            .push_op(Op::Emit) // parse count
            .push_op(Op::Call("__lower".into()))
            .push_op(Op::Emit) // op count
            .push_op(Op::Call("__encode_bytecode".into()))
            // Stack: [bytecode array ref]
            // 2. Write bytecode to file
            .push_op(Op::Push(key_chain(bc_path_str)))
            .push_op(Op::Swap) // [path, bytecode]
            .push_op(Op::Call("__file_write".into()))
            .push_op(Op::Emit) // write success
            // 3. Read bytecode back
            .push_op(Op::Push(key_chain(bc_path_str)))
            .push_op(Op::Call("__file_read".into()))
            .push_op(Op::Call("__array_len".into()))
            .push_op(Op::Emit) // read length
            .push_op(Op::Halt);

        let result = vm().execute(&prog);
        let _ = std::fs::remove_dir_all(&dir);

        assert!(result.outputs().len() >= 4, "need 4 outputs, got {}", result.outputs().len());
        let parse_count = result.outputs()[0].to_number().unwrap_or(0.0);
        assert!(parse_count > 0.0, "parse should succeed");
        let op_count = result.outputs()[1].to_number().unwrap_or(0.0);
        assert!(op_count > 0.0, "lower should succeed");
        let write_ok = result.outputs()[2].to_number().unwrap_or(0.0);
        assert!((write_ok - 1.0).abs() < f64::EPSILON, "write should succeed");
        let read_len = result.outputs()[3].to_number().unwrap_or(0.0);
        assert!(read_len > 0.0, "bytecode file should have content, got {}", read_len);
    }
}
