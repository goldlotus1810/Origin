//! # occlusion — SDF Occlusion Buffer
//!
//! Ring buffer giữ kết quả SDF gần nhất (N frames).
//! Dùng để:
//!   - Temporal coherence: frame gần nhau → SDF gần nhau
//!   - Occlusion: nếu SDF đã âm liên tục → skip ray march
//!   - Persistence: giữ 5 frames cuối → detect movement

/// Số frames mặc định trong occlusion buffer.
pub const DEFAULT_BUFFER_SIZE: usize = 5;

/// Một frame trong occlusion buffer.
#[derive(Debug, Clone, Copy)]
pub struct OcclusionFrame {
    /// SDF distance tại điểm sample.
    pub distance: f32,
    /// Shape index (0..17) của primitive gần nhất.
    pub shape_idx: u8,
    /// Timestamp (ns).
    pub timestamp: i64,
}

/// Ring buffer cho SDF occlusion — giữ N frames cuối.
///
/// Persistence: 5 frames cho phép detect movement mà không ray march lại.
pub struct OcclusionBuffer {
    frames: [Option<OcclusionFrame>; DEFAULT_BUFFER_SIZE],
    head: usize,
    count: usize,
}

impl OcclusionBuffer {
    /// Tạo buffer rỗng.
    pub const fn new() -> Self {
        Self {
            frames: [None; DEFAULT_BUFFER_SIZE],
            head: 0,
            count: 0,
        }
    }

    /// Push frame mới vào buffer (FIFO ring).
    pub fn push(&mut self, frame: OcclusionFrame) {
        self.frames[self.head] = Some(frame);
        self.head = (self.head + 1) % DEFAULT_BUFFER_SIZE;
        if self.count < DEFAULT_BUFFER_SIZE {
            self.count += 1;
        }
    }

    /// Số frames đã có.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Buffer rỗng?
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Frame gần nhất.
    pub fn latest(&self) -> Option<&OcclusionFrame> {
        if self.count == 0 {
            return None;
        }
        let idx = if self.head == 0 {
            DEFAULT_BUFFER_SIZE - 1
        } else {
            self.head - 1
        };
        self.frames[idx].as_ref()
    }

    /// Tất cả frames (cũ → mới).
    pub fn all(&self) -> alloc::vec::Vec<&OcclusionFrame> {
        let mut result = alloc::vec::Vec::new();
        if self.count == 0 {
            return result;
        }
        let start = if self.count < DEFAULT_BUFFER_SIZE {
            0
        } else {
            self.head
        };
        for i in 0..self.count {
            let idx = (start + i) % DEFAULT_BUFFER_SIZE;
            if let Some(ref f) = self.frames[idx] {
                result.push(f);
            }
        }
        result
    }

    /// Kiểm tra occlusion: distance < 0 liên tục qua tất cả frames?
    ///
    /// Nếu true → vật thể đã bị che hoàn toàn → skip ray march.
    pub fn is_occluded(&self) -> bool {
        self.count == DEFAULT_BUFFER_SIZE
            && self
                .frames
                .iter()
                .all(|f| f.as_ref().is_some_and(|fr| fr.distance < 0.0))
    }

    /// Movement detection: variance cao giữa các frames → đang di chuyển.
    pub fn movement_variance(&self) -> f32 {
        if self.count < 2 {
            return 0.0;
        }
        let frames = self.all();
        let mean = frames.iter().map(|f| f.distance).sum::<f32>() / frames.len() as f32;
        let var = frames
            .iter()
            .map(|f| (f.distance - mean) * (f.distance - mean))
            .sum::<f32>()
            / frames.len() as f32;
        var
    }
}

impl Default for OcclusionBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_buffer() {
        let buf = OcclusionBuffer::new();
        assert!(buf.is_empty());
        assert!(!buf.is_occluded());
        assert_eq!(buf.movement_variance(), 0.0);
    }

    #[test]
    fn push_and_latest() {
        let mut buf = OcclusionBuffer::new();
        buf.push(OcclusionFrame {
            distance: 1.0,
            shape_idx: 0,
            timestamp: 100,
        });
        assert_eq!(buf.len(), 1);
        assert!((buf.latest().unwrap().distance - 1.0).abs() < 0.001);
    }

    #[test]
    fn ring_buffer_wraps() {
        let mut buf = OcclusionBuffer::new();
        for i in 0..8 {
            buf.push(OcclusionFrame {
                distance: i as f32,
                shape_idx: 0,
                timestamp: i * 100,
            });
        }
        assert_eq!(buf.len(), DEFAULT_BUFFER_SIZE);
        assert!((buf.latest().unwrap().distance - 7.0).abs() < 0.001);
    }

    #[test]
    fn occluded_when_all_negative() {
        let mut buf = OcclusionBuffer::new();
        for i in 0..DEFAULT_BUFFER_SIZE {
            buf.push(OcclusionFrame {
                distance: -0.5,
                shape_idx: 0,
                timestamp: i as i64,
            });
        }
        assert!(buf.is_occluded());
    }

    #[test]
    fn not_occluded_when_mixed() {
        let mut buf = OcclusionBuffer::new();
        for i in 0..DEFAULT_BUFFER_SIZE {
            let d = if i % 2 == 0 { -0.5 } else { 0.5 };
            buf.push(OcclusionFrame {
                distance: d,
                shape_idx: 0,
                timestamp: i as i64,
            });
        }
        assert!(!buf.is_occluded());
    }

    #[test]
    fn movement_variance_detects_motion() {
        let mut buf = OcclusionBuffer::new();
        // Constant distance → low variance
        for i in 0..5 {
            buf.push(OcclusionFrame {
                distance: 1.0,
                shape_idx: 0,
                timestamp: i,
            });
        }
        assert!(buf.movement_variance() < 0.01);

        // Varying distance → high variance
        let mut buf2 = OcclusionBuffer::new();
        for i in 0..5 {
            buf2.push(OcclusionFrame {
                distance: i as f32 * 2.0,
                shape_idx: 0,
                timestamp: i,
            });
        }
        assert!(buf2.movement_variance() > 1.0);
    }
}
