//! # address — ISLAddress
//!
//! 4 bytes: [layer][group][subgroup][index]
//!
//! Allocation không collision:
//!   layer    = depth trong KnowledgeTree (0=L0, 1=L1, ...)
//!   group    = ShapeBase của molecule đầu tiên
//!   subgroup = RelationBase của molecule đầu tiên
//!   index    = counter trong (layer, group, subgroup) — tự tăng
//!
//! Bug cũ: 34 nodes trùng key → fix bằng cách enforce counter tự tăng

extern crate alloc;

// ─────────────────────────────────────────────────────────────────────────────
// ISLAddress — 4 bytes
// ─────────────────────────────────────────────────────────────────────────────

/// Địa chỉ không gian 4 bytes.
///
/// Serialize: [layer, group, subgroup, index] = 4 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ISLAddress {
    pub layer: u8,
    pub group: u8,
    pub subgroup: u8,
    pub index: u8,
}

impl ISLAddress {
    /// Tạo address từ 4 components.
    pub const fn new(layer: u8, group: u8, subgroup: u8, index: u8) -> Self {
        Self {
            layer,
            group,
            subgroup,
            index,
        }
    }

    /// L0 root address.
    pub const ROOT: Self = Self::new(0x00, 0x00, 0x00, 0x00);

    /// Broadcast address (tất cả 0xFF).
    pub const BROADCAST: Self = Self::new(0xFF, 0xFF, 0xFF, 0xFF);

    /// Serialize → 4 bytes [layer, group, subgroup, index].
    pub fn to_bytes(self) -> [u8; 4] {
        [self.layer, self.group, self.subgroup, self.index]
    }

    /// Deserialize từ 4 bytes.
    pub fn from_bytes(b: [u8; 4]) -> Self {
        Self::new(b[0], b[1], b[2], b[3])
    }

    /// Deserialize từ u32 (big-endian).
    pub fn from_u32(v: u32) -> Self {
        let b = v.to_be_bytes();
        Self::from_bytes(b)
    }

    /// Serialize → u32 (big-endian).
    pub fn to_u32(self) -> u32 {
        u32::from_be_bytes(self.to_bytes())
    }

    /// Kiểm tra có phải broadcast không.
    pub fn is_broadcast(self) -> bool {
        self == Self::BROADCAST
    }

    /// Layer number.
    pub fn layer(self) -> u8 {
        self.layer
    }

    /// Địa chỉ layer tiếp theo (sub-address).
    pub fn child(self, group: u8, subgroup: u8, index: u8) -> Self {
        Self::new(self.layer.saturating_add(1), group, subgroup, index)
    }
}

impl core::fmt::Display for ISLAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ISL[{:02X}:{:02X}:{:02X}:{:02X}]",
            self.layer, self.group, self.subgroup, self.index
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ISLAllocator — đảm bảo unique address
// ─────────────────────────────────────────────────────────────────────────────

/// Allocator đảm bảo không collision.
///
/// Mỗi (layer, group, subgroup) có counter riêng.
/// Counter tự tăng → không bao giờ trùng trong cùng namespace.
#[derive(Debug, Default)]
pub struct ISLAllocator {
    /// Counter per namespace: key = (layer, group, subgroup) → next index (u16, full=256)
    counters: alloc::collections::BTreeMap<(u8, u8, u8), u16>,
}

impl ISLAllocator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Cấp phát address mới trong (layer, group, subgroup).
    ///
    /// Trả None nếu namespace đã đầy (256 addresses).
    pub fn alloc(&mut self, layer: u8, group: u8, subgroup: u8) -> Option<ISLAddress> {
        let key = (layer, group, subgroup);
        let counter = self.counters.entry(key).or_insert(0u16);
        if *counter >= 256 {
            return None;
        } // đầy (0..=255 dùng hết)
        let addr = ISLAddress::new(layer, group, subgroup, *counter as u8);
        *counter += 1;
        Some(addr)
    }

    /// Số addresses đã cấp phát trong namespace.
    /// Số addresses đã cấp phát trong namespace.
    pub fn count(&self, layer: u8, group: u8, subgroup: u8) -> u16 {
        self.counters
            .get(&(layer, group, subgroup))
            .copied()
            .unwrap_or(0)
    }

    /// Derive address từ molecular chain hash (thuật toán từ spec).
    ///
    /// layer    = depth
    /// group    = hash[0] % 64 (ShapeBase)
    /// subgroup = hash[1] % 64 (RelationBase)
    /// index    = counter tự tăng
    pub fn alloc_from_hash(&mut self, depth: u8, hash: u64) -> Option<ISLAddress> {
        let b = hash.to_be_bytes();
        let group = b[0] % 64;
        let sub = b[1] % 64;
        self.alloc(depth, group, sub)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_round_trip_bytes() {
        let a = ISLAddress::new(0x02, 0x1A, 0x3F, 0x07);
        assert_eq!(ISLAddress::from_bytes(a.to_bytes()), a);
    }

    #[test]
    fn address_round_trip_u32() {
        let a = ISLAddress::new(0x01, 0x02, 0x03, 0x04);
        assert_eq!(ISLAddress::from_u32(a.to_u32()), a);
    }

    #[test]
    fn address_broadcast() {
        assert!(ISLAddress::BROADCAST.is_broadcast());
        assert!(!ISLAddress::ROOT.is_broadcast());
    }

    #[test]
    fn address_ordering() {
        // Layer cao hơn → address "lớn hơn"
        let a = ISLAddress::new(0x00, 0x00, 0x00, 0x00);
        let b = ISLAddress::new(0x01, 0x00, 0x00, 0x00);
        assert!(b > a);
    }

    #[test]
    fn allocator_unique() {
        let mut alloc = ISLAllocator::new();
        let a1 = alloc.alloc(1, 0, 0).unwrap();
        let a2 = alloc.alloc(1, 0, 0).unwrap();
        assert_ne!(a1, a2, "Hai lần alloc phải khác nhau");
        assert_eq!(a1.index, 0);
        assert_eq!(a2.index, 1);
    }

    #[test]
    fn allocator_different_namespaces() {
        let mut alloc = ISLAllocator::new();
        let a = alloc.alloc(1, 0, 0).unwrap();
        let b = alloc.alloc(1, 0, 1).unwrap(); // subgroup khác
                                               // Cùng index 0 nhưng subgroup khác → không collision
        assert_eq!(a.index, 0);
        assert_eq!(b.index, 0);
        assert_ne!(a, b);
    }

    #[test]
    fn allocator_full_namespace() {
        let mut alloc = ISLAllocator::new();
        // Điền đầy 256 addresses
        for i in 0..=254u8 {
            let a = alloc.alloc(0, 0, 0);
            assert!(a.is_some(), "Lần {} phải có địa chỉ", i);
        }
        // Lần thứ 256 (0xFF đã chiếm) → None
        // Thực ra 0..=254 = 255 lần, lần 256 là index 255 vẫn được
        let a255 = alloc.alloc(0, 0, 0);
        assert!(a255.is_some()); // index 255 vẫn OK
        let overflow = alloc.alloc(0, 0, 0);
        assert!(overflow.is_none(), "Đầy → None");
    }

    #[test]
    fn allocator_from_hash() {
        let mut alloc = ISLAllocator::new();
        let hash = 0xABCD1234_EF567890_u64;
        let a = alloc.alloc_from_hash(2, hash).unwrap();
        assert_eq!(a.layer, 2);
        // group + subgroup từ hash bytes
        assert!(a.group < 64);
        assert!(a.subgroup < 64);
    }

    #[test]
    fn address_display() {
        let a = ISLAddress::new(0x01, 0x02, 0x03, 0x04);
        let s = alloc::format!("{}", a);
        assert!(s.contains("ISL[01:02:03:04]"), "Display: {}", s);
    }
}
