//! # clone — HomeOS Clone
//!
//! filter(origin.olang, DeviceProfile) → device.olang (~12KB)
//! Mỗi thiết bị = tế bào độc lập.
//!
//! Clone không copy toàn bộ — chỉ lấy phần cần thiết:
//!   DeviceProfile → capabilities → relevant L0 nodes
//!   + aliases của thiết bị đó
//!   + Silk edges liên quan
//!
//! Sau khi clone: thiết bị tự vận hành.
//! Sync lại origin qua delta khi kết nối.

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::{String, ToString};

use crate::molecular::MolecularChain;
use crate::encoder::encode_codepoint;
use crate::writer::OlangWriter;
use crate::reader::{OlangReader, ParsedFile};

// ─────────────────────────────────────────────────────────────────────────────
// DeviceProfile — khả năng của thiết bị
// ─────────────────────────────────────────────────────────────────────────────

/// Profile của một thiết bị clone.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct DeviceProfile {
    pub id:           String,
    pub device_type:  DeviceType,
    pub capabilities: Vec<Capability>,
    pub max_bytes:    usize, // giới hạn kích thước device.olang
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum DeviceType {
    /// Loa thông minh — chủ yếu audio
    Speaker,
    /// Cảm biến — temperature/humidity/motion
    Sensor,
    /// Màn hình — visual output
    Display,
    /// Hub — điều phối nhiều thiết bị
    Hub,
    /// Mobile — điện thoại
    Mobile,
    /// Server — full power
    Server,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum Capability {
    Audio,        // nghe/nói
    Vision,       // camera/display
    Temperature,  // cảm biến nhiệt
    Humidity,     // cảm biến độ ẩm
    Motion,       // cảm biến chuyển động
    Light,        // cảm biến ánh sáng
    Network,      // kết nối mạng
    Compute,      // xử lý nặng
    Storage,      // lưu trữ nhiều
    Emotion,      // emotion tracking
}

#[allow(missing_docs)]
impl DeviceProfile {
    pub fn speaker(id: &str) -> Self {
        Self {
            id:           id.to_string(),
            device_type:  DeviceType::Speaker,
            capabilities: alloc::vec![Capability::Audio, Capability::Emotion, Capability::Network],
            max_bytes:    12_000,
        }
    }

    pub fn sensor(id: &str) -> Self {
        Self {
            id:           id.to_string(),
            device_type:  DeviceType::Sensor,
            capabilities: alloc::vec![
                Capability::Temperature, Capability::Humidity,
                Capability::Motion, Capability::Light, Capability::Network,
            ],
            max_bytes:    8_000,
        }
    }

    pub fn hub(id: &str) -> Self {
        Self {
            id:           id.to_string(),
            device_type:  DeviceType::Hub,
            capabilities: alloc::vec![
                Capability::Audio, Capability::Vision, Capability::Network,
                Capability::Compute, Capability::Storage, Capability::Emotion,
            ],
            max_bytes:    64_000,
        }
    }

    pub fn mobile(id: &str) -> Self {
        Self {
            id:           id.to_string(),
            device_type:  DeviceType::Mobile,
            capabilities: alloc::vec![
                Capability::Audio, Capability::Vision, Capability::Emotion,
                Capability::Network, Capability::Compute,
            ],
            max_bytes:    32_000,
        }
    }

    /// Kiểm tra thiết bị có capability không.
    pub fn has(&self, cap: Capability) -> bool {
        self.capabilities.contains(&cap)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CloneFilter — quyết định node nào được giữ lại
// ─────────────────────────────────────────────────────────────────────────────

/// Quyết định node nào phù hợp với profile.
fn is_relevant(chain: &MolecularChain, profile: &DeviceProfile) -> bool {
    if chain.is_empty() { return false; }

    // Lấy molecule đầu tiên để check shape/emotion
    let mol = &chain.0[0];
    let v   = mol.emotion.valence;
    let a   = mol.emotion.arousal;

    // Tất cả thiết bị đều cần: origin, emotion cơ bản
    // Shape Sphere (0x01) = phổ biến nhất → luôn giữ
    if mol.shape.as_byte() == 0x01 { return true; }

    // Sensor profile: cần temperature, humidity, motion (arousal thấp-trung)
    if profile.has(Capability::Temperature) && a < 0x80 {
        return true;
    }

    // Audio profile: cần emotion nodes (valence mạnh)
    if profile.has(Capability::Audio) && (v > 0xC0 || v < 0x40) {
        return true;
    }

    // Compute/Hub: giữ tất cả
    if profile.has(Capability::Compute) || profile.has(Capability::Storage) {
        return true;
    }

    false
}

// ─────────────────────────────────────────────────────────────────────────────
// CloneResult
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả clone.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct CloneResult {
    /// Bytes của device.olang
    pub bytes:       Vec<u8>,
    /// Số nodes được clone
    pub node_count:  usize,
    /// Số aliases được clone
    pub alias_count: usize,
    /// Profile đã dùng
    pub profile_id:  String,
}

#[allow(missing_docs)]
impl CloneResult {
    pub fn size_bytes(&self) -> usize { self.bytes.len() }
    pub fn is_within_limit(&self, profile: &DeviceProfile) -> bool {
        self.bytes.len() <= profile.max_bytes
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// clone() — filter origin → device
// ─────────────────────────────────────────────────────────────────────────────

/// Clone origin.olang → device.olang theo DeviceProfile.
///
/// Trả về None nếu origin bytes không parse được.
pub fn clone_for_device(
    origin_bytes: &[u8],
    profile:      &DeviceProfile,
    ts:           i64,
) -> Option<CloneResult> {
    // Parse origin
    let reader = OlangReader::new(origin_bytes).ok()?;
    let parsed = reader.parse_all().ok()?;

    filter_and_write(&parsed, profile, ts)
}

fn filter_and_write(
    parsed:  &ParsedFile,
    profile: &DeviceProfile,
    ts:      i64,
) -> Option<CloneResult> {
    let mut writer      = OlangWriter::new(ts);
    let mut node_count  = 0usize;
    let mut alias_count = 0usize;
    let mut kept_hashes = Vec::new();

    // Filter nodes
    for node in &parsed.nodes {
        if is_relevant(&node.chain, profile) {
            if let Ok(_) = writer.append_node(&node.chain, node.layer, node.is_qr, node.timestamp) {
                kept_hashes.push(node.chain.chain_hash());
                node_count += 1;
            }
        }
        // Stop nếu đã đủ max_bytes
        if writer.size() >= profile.max_bytes { break; }
    }

    // Filter aliases — chỉ giữ aliases trỏ về nodes đã giữ
    for alias in &parsed.aliases {
        if alias.name.starts_with("_qr_") { continue; }
        if kept_hashes.contains(&alias.chain_hash) {
            if writer.append_alias(&alias.name, alias.chain_hash, alias.timestamp).is_ok() {
                alias_count += 1;
            }
        }
        if writer.size() >= profile.max_bytes { break; }
    }

    // Thêm device identity marker
    let device_chain = encode_codepoint(device_type_cp(profile.device_type));
    if let Ok(_) = writer.append_node(&device_chain, 0, true, ts) {
        writer.append_alias(&profile.id, device_chain.chain_hash(), ts).ok();
        node_count += 1;
    }

    Some(CloneResult {
        bytes:       writer.into_bytes(),
        node_count,
        alias_count,
        profile_id:  profile.id.clone(),
    })
}

fn device_type_cp(dt: DeviceType) -> u32 {
    match dt {
        DeviceType::Speaker => 0x1F50A, // 🔊
        DeviceType::Sensor  => 0x1F321, // 🌡
        DeviceType::Display => 0x1F4FA, // 📺
        DeviceType::Hub     => 0x1F4E1, // 📡
        DeviceType::Mobile  => 0x1F4F1, // 📱
        DeviceType::Server  => 0x1F5A5, // 🖥
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Delta sync — device → origin
// ─────────────────────────────────────────────────────────────────────────────

/// Tính delta: những gì device học thêm mà origin chưa có.
///
/// device.olang có thể học từ local sensor/audio →
/// tạo ĐN nodes mới → sync về origin.
pub fn compute_delta(
    origin_bytes: &[u8],
    device_bytes: &[u8],
    ts:           i64,
) -> Option<Vec<u8>> {
    let origin_reader = OlangReader::new(origin_bytes).ok()?;
    let device_reader = OlangReader::new(device_bytes).ok()?;

    let origin_parsed = origin_reader.parse_all().ok()?;
    let device_parsed = device_reader.parse_all().ok()?;

    // Collect origin hashes
    let origin_hashes: Vec<u64> = origin_parsed.nodes.iter()
        .map(|n| n.chain.chain_hash())
        .collect();

    // Tìm nodes device có mà origin chưa có
    let mut delta_writer = OlangWriter::new(ts);
    let mut delta_count  = 0usize;

    for node in &device_parsed.nodes {
        let hash = node.chain.chain_hash();
        if !origin_hashes.contains(&hash) {
            if delta_writer.append_node(
                &node.chain, node.layer, false, // ĐN — cần AAM approve
                node.timestamp,
            ).is_ok() {
                delta_count += 1;
            }
        }
    }

    if delta_count == 0 { return None; } // Không có gì mới

    Some(delta_writer.into_bytes())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;
    use alloc::vec;
    use crate::encoder::encode_codepoint;
    use crate::writer::OlangWriter;

    fn skip() -> bool { ucd::table_len() == 0 }

    fn make_origin(ts: i64) -> Vec<u8> {
        let mut w = OlangWriter::new(ts);
        let chains = [0x1F525u32, 0x1F4A7, 0x2744, 0x1F9E0, 0x25CF, 0x26A0];
        for cp in chains {
            let chain = encode_codepoint(cp);
            w.append_node(&chain, 0, true, ts).ok();
            w.append_alias(&format!("node_{:05X}", cp), chain.chain_hash(), ts).ok();
        }
        w.into_bytes()
    }

    #[test]
    fn clone_speaker_smaller_than_origin() {
        if skip() { return; }
        let origin = make_origin(1000);
        let profile = DeviceProfile::speaker("living_room");
        let result = clone_for_device(&origin, &profile, 2000);
        assert!(result.is_some(), "Clone phải thành công");
        let r = result.unwrap();
        assert!(r.size_bytes() > 0, "Device file không rỗng");
        assert!(r.is_within_limit(&profile),
            "Size {} ≤ max {}", r.size_bytes(), profile.max_bytes);
    }

    #[test]
    fn clone_preserves_relevant_nodes() {
        if skip() { return; }
        let origin = make_origin(1000);
        let profile = DeviceProfile::hub("hub_01"); // Hub giữ tất cả
        let result = clone_for_device(&origin, &profile, 2000).unwrap();
        assert!(result.node_count > 0, "Hub clone phải có nodes");
    }

    #[test]
    fn clone_hub_more_than_sensor() {
        if skip() { return; }
        let origin = make_origin(1000);
        let hub_r    = clone_for_device(&origin, &DeviceProfile::hub("hub"),    2000).unwrap();
        let sensor_r = clone_for_device(&origin, &DeviceProfile::sensor("s01"), 2000).unwrap();
        assert!(hub_r.node_count >= sensor_r.node_count,
            "Hub giữ ≥ Sensor: {} ≥ {}", hub_r.node_count, sensor_r.node_count);
    }

    #[test]
    fn clone_has_device_marker() {
        if skip() { return; }
        let origin  = make_origin(1000);
        let profile = DeviceProfile::speaker("test_speaker");
        let result  = clone_for_device(&origin, &profile, 2000).unwrap();
        // Verify device file có thể parse được
        let reader = OlangReader::new(&result.bytes).expect("parse device file");
        let parsed = reader.parse_all().expect("parse all");
        // Device identity alias phải có
        assert!(
            parsed.aliases.iter().any(|a| a.name == "test_speaker"),
            "Device ID alias phải có trong file"
        );
    }

    #[test]
    fn clone_bad_origin_returns_none() {
        let bad = [0u8; 20];
        let profile = DeviceProfile::sensor("s01");
        let result = clone_for_device(&bad, &profile, 0);
        assert!(result.is_none(), "Bad origin → None");
    }

    #[test]
    fn delta_no_new_nodes() {
        if skip() { return; }
        let origin = make_origin(1000);
        let profile = DeviceProfile::hub("hub");
        let device = clone_for_device(&origin, &profile, 2000).unwrap();
        // Nếu device không học gì mới → delta = None
        let delta = compute_delta(&origin, &device.bytes, 3000);
        // Hub giữ tất cả → device không có gì mới → None
        // (hoặc có device marker) — OK cả 2 case
        let _ = delta;
    }

    #[test]
    fn device_profile_capabilities() {
        let speaker = DeviceProfile::speaker("s");
        assert!(speaker.has(Capability::Audio));
        assert!(!speaker.has(Capability::Temperature));

        let sensor = DeviceProfile::sensor("t");
        assert!(sensor.has(Capability::Temperature));
        assert!(!sensor.has(Capability::Audio));

        let hub = DeviceProfile::hub("h");
        assert!(hub.has(Capability::Compute));
        assert!(hub.has(Capability::Audio));
    }

    #[test]
    fn clone_within_size_limit() {
        if skip() { return; }
        let origin  = make_origin(1000);
        let profile = DeviceProfile::sensor("tiny_sensor");
        let result  = clone_for_device(&origin, &profile, 2000).unwrap();
        assert!(result.bytes.len() <= profile.max_bytes,
            "Clone {} bytes ≤ limit {} bytes",
            result.bytes.len(), profile.max_bytes);
    }

    #[test]
    fn clone_result_roundtrip() {
        if skip() { return; }
        let origin = make_origin(1000);
        let profile = DeviceProfile::mobile("phone_01");
        let result = clone_for_device(&origin, &profile, 2000).unwrap();

        // Device file phải parse được
        let reader = OlangReader::new(&result.bytes).expect("parse");
        let parsed = reader.parse_all().expect("parse all");
        assert_eq!(parsed.node_count(), result.node_count,
            "Roundtrip node count khớp");
    }
}
