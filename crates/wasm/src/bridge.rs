//! # bridge — WebSocket ↔ ISL Bridge
//!
//! Browser ↔ HomeOS qua WebSocket → ISL binary frames.
//!
//! Pipeline:
//!   Browser JS → WebSocket → binary ISL frame → HomeOS process
//!   HomeOS response → ISL frame → WebSocket → Browser JS
//!
//! Frame format (WebSocket binary):
//!   [0x4F 0x53] — magic "OS"
//!   [type:1B]   — BridgeMsg type
//!   [len:2B]    — payload length
//!   [payload]   — ISL frame or text
//!
//! Event streaming: HomeOS → Browser (push events):
//!   Emotion update, Dream results, Silk changes, STM observations

extern crate alloc;
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// Bridge message types
// ─────────────────────────────────────────────────────────────────────────────

/// WebSocket bridge message type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BridgeMsg {
    /// Text input from browser
    TextInput = 0x01,
    /// Olang expression from browser
    OlangInput = 0x02,
    /// Audio features from browser (Web Audio API)
    AudioInput = 0x03,
    /// Response from HomeOS → browser
    Response = 0x10,
    /// Emotion update event (push)
    EmotionUpdate = 0x11,
    /// Dream result event (push)
    DreamResult = 0x12,
    /// Silk change event (push)
    SilkUpdate = 0x13,
    /// Scene update (3D world changed)
    SceneUpdate = 0x14,
    /// System stats
    Stats = 0x20,
    /// Health check
    Health = 0x21,
    /// Ping/Pong
    Ping = 0xFE,
    /// Pong
    Pong = 0xFF,
}

impl BridgeMsg {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::TextInput),
            0x02 => Some(Self::OlangInput),
            0x03 => Some(Self::AudioInput),
            0x10 => Some(Self::Response),
            0x11 => Some(Self::EmotionUpdate),
            0x12 => Some(Self::DreamResult),
            0x13 => Some(Self::SilkUpdate),
            0x14 => Some(Self::SceneUpdate),
            0x20 => Some(Self::Stats),
            0x21 => Some(Self::Health),
            0xFE => Some(Self::Ping),
            0xFF => Some(Self::Pong),
            _ => None,
        }
    }

    pub fn as_byte(self) -> u8 {
        self as u8
    }
}

/// Magic bytes for bridge frames.
pub const BRIDGE_MAGIC: [u8; 2] = [0x4F, 0x53]; // "OS"

// ─────────────────────────────────────────────────────────────────────────────
// BridgeFrame — encode/decode
// ─────────────────────────────────────────────────────────────────────────────

/// WebSocket bridge frame.
#[derive(Debug, Clone)]
pub struct BridgeFrame {
    pub msg_type: BridgeMsg,
    pub payload: Vec<u8>,
}

impl BridgeFrame {
    /// Create frame.
    pub fn new(msg_type: BridgeMsg, payload: Vec<u8>) -> Self {
        Self { msg_type, payload }
    }

    /// Create text input frame.
    pub fn text_input(text: &str) -> Self {
        Self::new(BridgeMsg::TextInput, text.as_bytes().to_vec())
    }

    /// Create response frame (JSON text).
    pub fn response(json: &str) -> Self {
        Self::new(BridgeMsg::Response, json.as_bytes().to_vec())
    }

    /// Create emotion update frame.
    pub fn emotion_update(valence: f32, arousal: f32, fx: f32) -> Self {
        let mut payload = Vec::with_capacity(12);
        payload.extend_from_slice(&valence.to_be_bytes());
        payload.extend_from_slice(&arousal.to_be_bytes());
        payload.extend_from_slice(&fx.to_be_bytes());
        Self::new(BridgeMsg::EmotionUpdate, payload)
    }

    /// Create scene update frame (JSON).
    pub fn scene_update(scene_json: &str) -> Self {
        Self::new(BridgeMsg::SceneUpdate, scene_json.as_bytes().to_vec())
    }

    /// Create ping frame.
    pub fn ping() -> Self {
        Self::new(BridgeMsg::Ping, Vec::new())
    }

    /// Create pong frame.
    pub fn pong() -> Self {
        Self::new(BridgeMsg::Pong, Vec::new())
    }

    /// Encode frame → bytes (for WebSocket binary message).
    ///
    /// Format: [magic:2][type:1][len:2][payload:N]
    pub fn to_bytes(&self) -> Vec<u8> {
        let len = self.payload.len() as u16;
        let mut buf = Vec::with_capacity(5 + self.payload.len());
        buf.extend_from_slice(&BRIDGE_MAGIC);
        buf.push(self.msg_type.as_byte());
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }

    /// Decode bytes → frame.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 5 {
            return None;
        }
        if bytes[0] != BRIDGE_MAGIC[0] || bytes[1] != BRIDGE_MAGIC[1] {
            return None;
        }
        let msg_type = BridgeMsg::from_byte(bytes[2])?;
        let len = u16::from_be_bytes([bytes[3], bytes[4]]) as usize;
        if bytes.len() < 5 + len {
            return None;
        }
        let payload = bytes[5..5 + len].to_vec();
        Some(Self { msg_type, payload })
    }

    /// Payload as UTF-8 string (if applicable).
    pub fn payload_str(&self) -> Option<&str> {
        core::str::from_utf8(&self.payload).ok()
    }

    /// Frame size in bytes.
    pub fn size(&self) -> usize {
        5 + self.payload.len()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EventStream — push events from HomeOS → browser
// ─────────────────────────────────────────────────────────────────────────────

/// Event stream — collect events to push to browser.
///
/// HomeOS core → push events → EventStream → serialize → WebSocket → browser
pub struct EventStream {
    /// Pending events
    events: Vec<BridgeFrame>,
    /// Max events before flush
    max_pending: usize,
}

impl EventStream {
    pub fn new(max_pending: usize) -> Self {
        Self {
            events: Vec::new(),
            max_pending,
        }
    }

    /// Push emotion update.
    pub fn push_emotion(&mut self, valence: f32, arousal: f32, fx: f32) {
        self.events
            .push(BridgeFrame::emotion_update(valence, arousal, fx));
        self.auto_flush();
    }

    /// Push scene update.
    pub fn push_scene(&mut self, scene_json: &str) {
        self.events.push(BridgeFrame::scene_update(scene_json));
        self.auto_flush();
    }

    /// Push dream result.
    pub fn push_dream(&mut self, summary: &str) {
        self.events.push(BridgeFrame::new(
            BridgeMsg::DreamResult,
            summary.as_bytes().to_vec(),
        ));
    }

    /// Push silk change notification.
    pub fn push_silk_update(&mut self, edge_count: u32) {
        self.events.push(BridgeFrame::new(
            BridgeMsg::SilkUpdate,
            edge_count.to_be_bytes().to_vec(),
        ));
    }

    /// Drain all pending events as frames.
    pub fn drain(&mut self) -> Vec<BridgeFrame> {
        core::mem::take(&mut self.events)
    }

    /// Drain all pending events as bytes (for WebSocket).
    pub fn drain_bytes(&mut self) -> Vec<Vec<u8>> {
        self.drain().into_iter().map(|f| f.to_bytes()).collect()
    }

    /// Pending count.
    pub fn pending(&self) -> usize {
        self.events.len()
    }

    /// Auto-flush: drop oldest if too many pending.
    fn auto_flush(&mut self) {
        if self.events.len() > self.max_pending {
            // Keep only the latest half
            let keep = self.max_pending / 2;
            let start = self.events.len() - keep;
            self.events = self.events[start..].to_vec();
        }
    }
}

impl Default for EventStream {
    fn default() -> Self {
        Self::new(64)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_frame_roundtrip() {
        let frame = BridgeFrame::text_input("xin chào");
        let bytes = frame.to_bytes();
        let decoded = BridgeFrame::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.msg_type, BridgeMsg::TextInput);
        assert_eq!(decoded.payload_str().unwrap(), "xin chào");
    }

    #[test]
    fn bridge_frame_magic() {
        let frame = BridgeFrame::ping();
        let bytes = frame.to_bytes();
        assert_eq!(&bytes[0..2], &BRIDGE_MAGIC);
        assert_eq!(bytes[2], BridgeMsg::Ping.as_byte());
    }

    #[test]
    fn bridge_frame_emotion() {
        let frame = BridgeFrame::emotion_update(-0.5, 0.7, -0.15);
        let bytes = frame.to_bytes();
        let decoded = BridgeFrame::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.msg_type, BridgeMsg::EmotionUpdate);
        assert_eq!(decoded.payload.len(), 12); // 3 × f32
    }

    #[test]
    fn bridge_frame_response() {
        let json = r#"{"text":"hello","tone":"engaged"}"#;
        let frame = BridgeFrame::response(json);
        let bytes = frame.to_bytes();
        let decoded = BridgeFrame::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.payload_str().unwrap(), json);
    }

    #[test]
    fn bridge_frame_invalid() {
        // Too short
        assert!(BridgeFrame::from_bytes(&[0x4F]).is_none());
        // Wrong magic
        assert!(BridgeFrame::from_bytes(&[0x00, 0x00, 0x01, 0x00, 0x00]).is_none());
        // Invalid msg type
        assert!(BridgeFrame::from_bytes(&[0x4F, 0x53, 0x99, 0x00, 0x00]).is_none());
    }

    #[test]
    fn bridge_msg_roundtrip() {
        for b in [0x01, 0x02, 0x10, 0x11, 0x14, 0x20, 0xFE, 0xFF] {
            let msg = BridgeMsg::from_byte(b).unwrap();
            assert_eq!(msg.as_byte(), b);
        }
    }

    #[test]
    fn event_stream_push_drain() {
        let mut stream = EventStream::new(64);
        stream.push_emotion(-0.3, 0.5, -0.1);
        stream.push_emotion(-0.4, 0.6, -0.2);
        assert_eq!(stream.pending(), 2);

        let events = stream.drain();
        assert_eq!(events.len(), 2);
        assert_eq!(stream.pending(), 0);
    }

    #[test]
    fn event_stream_auto_flush() {
        let mut stream = EventStream::new(4);
        for i in 0..10 {
            stream.push_emotion(i as f32 * 0.1, 0.5, 0.0);
        }
        // Should have auto-flushed to keep max_pending/2 = 2
        assert!(stream.pending() <= 4);
    }

    #[test]
    fn event_stream_drain_bytes() {
        let mut stream = EventStream::new(64);
        stream.push_emotion(-0.5, 0.7, -0.15);
        let bytes_list = stream.drain_bytes();
        assert_eq!(bytes_list.len(), 1);
        // Each frame starts with magic
        assert_eq!(&bytes_list[0][0..2], &BRIDGE_MAGIC);
    }

    #[test]
    fn event_stream_scene_update() {
        let mut stream = EventStream::new(64);
        let json = r#"{"nodes":[]}"#;
        stream.push_scene(json);
        let events = stream.drain();
        assert_eq!(events[0].msg_type, BridgeMsg::SceneUpdate);
        assert_eq!(events[0].payload_str().unwrap(), json);
    }

    #[test]
    fn event_stream_dream_silk() {
        let mut stream = EventStream::new(64);
        stream.push_dream("3 clusters, 2 proposals");
        stream.push_silk_update(150);
        assert_eq!(stream.pending(), 2);
        let events = stream.drain();
        assert_eq!(events[0].msg_type, BridgeMsg::DreamResult);
        assert_eq!(events[1].msg_type, BridgeMsg::SilkUpdate);
    }
}
