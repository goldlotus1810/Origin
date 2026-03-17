//! # driver — Device driver abstractions
//!
//! Driver traits cho các thiết bị ngoại vi cơ bản:
//!   Input:  Keyboard, Mouse/Touchpad, Touchscreen
//!   Output: Display, Speaker/Audio
//!   Sensor: Accelerometer, Gyroscope, GPS, Light, Proximity
//!   Storage: FileSystem abstraction
//!
//! Mỗi driver = trait. Platform impl cung cấp concrete implementation.
//! Worker gọi trait → không biết hardware cụ thể.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// Input Devices
// ─────────────────────────────────────────────────────────────────────────────

/// Sự kiện input chung.
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Phím bấm (keycode, pressed)
    Key { code: u16, pressed: bool },
    /// Chuột/touchpad di chuyển (dx, dy)
    MouseMove { dx: i16, dy: i16 },
    /// Chuột click (button, pressed)
    MouseClick { button: u8, pressed: bool },
    /// Cuộn chuột
    Scroll { delta: i16 },
    /// Touch screen (x, y, pressure, finger_id)
    Touch {
        x: u16,
        y: u16,
        pressure: u8,
        finger: u8,
    },
    /// Touch kết thúc
    TouchEnd { finger: u8 },
    /// Gesture (pinch, swipe, rotate)
    Gesture(GestureEvent),
}

/// Gesture events (mobile).
#[derive(Debug, Clone, Copy)]
pub enum GestureEvent {
    /// Pinch zoom (scale factor, 1.0 = no change)
    Pinch { scale: f32 },
    /// Swipe (direction)
    Swipe { direction: SwipeDir },
    /// Long press tại (x, y)
    LongPress { x: u16, y: u16 },
}

/// Hướng swipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeDir {
    /// Lên
    Up,
    /// Xuống
    Down,
    /// Trái
    Left,
    /// Phải
    Right,
}

/// Keyboard driver trait.
pub trait KeyboardDriver {
    /// Đọc sự kiện phím (non-blocking). None = không có event.
    fn poll_key(&self) -> Option<InputEvent>;
    /// Layout hiện tại (VD: "us", "vn_telex")
    fn layout(&self) -> &str;
}

/// Mouse/Touchpad driver trait.
pub trait PointerDriver {
    /// Đọc sự kiện chuột/touchpad (non-blocking).
    fn poll_pointer(&self) -> Option<InputEvent>;
    /// Sensitivity (0.0 = chậm, 1.0 = nhanh)
    fn sensitivity(&self) -> f32;
}

/// Touchscreen driver trait.
pub trait TouchDriver {
    /// Đọc sự kiện touch (non-blocking).
    fn poll_touch(&self) -> Option<InputEvent>;
    /// Hỗ trợ multi-touch? (max fingers)
    fn max_fingers(&self) -> u8;
    /// Screen resolution
    fn resolution(&self) -> (u16, u16);
}

// ─────────────────────────────────────────────────────────────────────────────
// Output Devices
// ─────────────────────────────────────────────────────────────────────────────

/// Display info.
#[derive(Debug, Clone)]
pub struct DisplayInfo {
    /// Width (pixels)
    pub width: u16,
    /// Height (pixels)
    pub height: u16,
    /// DPI
    pub dpi: u16,
    /// Refresh rate (Hz)
    pub refresh_hz: u8,
    /// Display type
    pub kind: DisplayKind,
}

/// Loại display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayKind {
    /// LCD (PC monitor, laptop)
    Lcd,
    /// OLED (smartphone, tablet)
    Oled,
    /// E-ink (e-reader)
    Eink,
    /// LED matrix (embedded)
    LedMatrix,
    /// Terminal (text-only)
    Terminal,
    /// None (headless)
    None,
}

/// Display driver trait.
pub trait DisplayDriver {
    /// Thông tin display.
    fn info(&self) -> DisplayInfo;
    /// Ghi text tại vị trí (row, col).
    fn write_text(&self, row: u16, col: u16, text: &str);
    /// Clear screen.
    fn clear(&self);
    /// Có hỗ trợ đồ họa không.
    fn supports_graphics(&self) -> bool;
}

/// Audio output info.
#[derive(Debug, Clone)]
pub struct AudioInfo {
    /// Sample rate (Hz)
    pub sample_rate: u32,
    /// Channels (1=mono, 2=stereo)
    pub channels: u8,
    /// Volume (0..100)
    pub volume: u8,
    /// Muted?
    pub muted: bool,
}

/// Audio driver trait.
pub trait AudioDriver {
    /// Thông tin audio.
    fn info(&self) -> AudioInfo;
    /// Phát âm thanh (PCM samples).
    fn play(&self, samples: &[i16]) -> bool;
    /// Dừng phát.
    fn stop(&self);
    /// Set volume (0..100).
    fn set_volume(&self, vol: u8);
}

// ─────────────────────────────────────────────────────────────────────────────
// Mobile Sensors
// ─────────────────────────────────────────────────────────────────────────────

/// Accelerometer reading (m/s²).
#[derive(Debug, Clone, Copy)]
pub struct AccelReading {
    /// X axis
    pub x: f32,
    /// Y axis
    pub y: f32,
    /// Z axis
    pub z: f32,
    /// Timestamp (ms)
    pub ts: i64,
}

impl AccelReading {
    /// Magnitude = √(x² + y² + z²)
    pub fn magnitude(&self) -> f32 {
        homemath::sqrtf(self.x * self.x + self.y * self.y + self.z * self.z)
    }
}

/// Gyroscope reading (rad/s).
#[derive(Debug, Clone, Copy)]
pub struct GyroReading {
    /// Roll
    pub roll: f32,
    /// Pitch
    pub pitch: f32,
    /// Yaw
    pub yaw: f32,
    /// Timestamp (ms)
    pub ts: i64,
}

/// GPS location.
#[derive(Debug, Clone, Copy)]
pub struct GpsLocation {
    /// Latitude (degrees)
    pub lat: f64,
    /// Longitude (degrees)
    pub lon: f64,
    /// Altitude (meters, NaN if unknown)
    pub alt: f64,
    /// Accuracy (meters)
    pub accuracy: f32,
    /// Timestamp (ms)
    pub ts: i64,
}

/// Light sensor reading.
#[derive(Debug, Clone, Copy)]
pub struct LightReading {
    /// Luminosity (lux)
    pub lux: f32,
    /// Timestamp (ms)
    pub ts: i64,
}

/// Proximity sensor reading.
#[derive(Debug, Clone, Copy)]
pub struct ProximityReading {
    /// Khoảng cách (cm). 0 = rất gần (trong túi/áp mặt).
    pub distance_cm: f32,
    /// Near flag (binary: gần/xa)
    pub is_near: bool,
    /// Timestamp (ms)
    pub ts: i64,
}

/// Accelerometer driver trait.
pub trait AccelDriver {
    /// Đọc gia tốc hiện tại.
    fn read(&self) -> Option<AccelReading>;
    /// Sample rate (Hz).
    fn sample_rate(&self) -> u16;
}

/// Gyroscope driver trait.
pub trait GyroDriver {
    /// Đọc góc quay hiện tại.
    fn read(&self) -> Option<GyroReading>;
}

/// GPS driver trait.
pub trait GpsDriver {
    /// Đọc vị trí hiện tại.
    fn read(&self) -> Option<GpsLocation>;
    /// Có fix GPS không.
    fn has_fix(&self) -> bool;
}

/// Light sensor driver trait.
pub trait LightDriver {
    /// Đọc ánh sáng hiện tại.
    fn read(&self) -> Option<LightReading>;
}

/// Proximity sensor driver trait.
pub trait ProximityDriver {
    /// Đọc proximity.
    fn read(&self) -> Option<ProximityReading>;
}

// ─────────────────────────────────────────────────────────────────────────────
// Mock implementations for testing
// ─────────────────────────────────────────────────────────────────────────────

/// Mock keyboard cho testing.
pub struct MockKeyboard {
    /// Pre-loaded events
    pub events: Vec<InputEvent>,
    /// Layout
    pub layout_name: String,
}

impl MockKeyboard {
    /// PC keyboard.
    pub fn pc() -> Self {
        Self {
            events: Vec::new(),
            layout_name: String::from("us"),
        }
    }
    /// Mobile soft keyboard.
    pub fn mobile() -> Self {
        Self {
            events: Vec::new(),
            layout_name: String::from("vn_telex"),
        }
    }
}

impl KeyboardDriver for MockKeyboard {
    fn poll_key(&self) -> Option<InputEvent> {
        None // Mock: no events queued
    }
    fn layout(&self) -> &str {
        &self.layout_name
    }
}

/// Mock display cho testing.
pub struct MockDisplay {
    /// Display info
    pub display: DisplayInfo,
}

impl MockDisplay {
    /// PC 1080p monitor.
    pub fn pc_1080p() -> Self {
        Self {
            display: DisplayInfo {
                width: 1920,
                height: 1080,
                dpi: 96,
                refresh_hz: 60,
                kind: DisplayKind::Lcd,
            },
        }
    }
    /// Smartphone OLED.
    pub fn smartphone_oled() -> Self {
        Self {
            display: DisplayInfo {
                width: 1080,
                height: 2400,
                dpi: 420,
                refresh_hz: 120,
                kind: DisplayKind::Oled,
            },
        }
    }
    /// Embedded LED matrix.
    pub fn led_matrix() -> Self {
        Self {
            display: DisplayInfo {
                width: 16,
                height: 8,
                dpi: 1,
                refresh_hz: 30,
                kind: DisplayKind::LedMatrix,
            },
        }
    }
    /// Terminal (text).
    pub fn terminal() -> Self {
        Self {
            display: DisplayInfo {
                width: 80,
                height: 24,
                dpi: 0,
                refresh_hz: 0,
                kind: DisplayKind::Terminal,
            },
        }
    }
}

impl DisplayDriver for MockDisplay {
    fn info(&self) -> DisplayInfo {
        self.display.clone()
    }
    fn write_text(&self, _row: u16, _col: u16, _text: &str) {}
    fn clear(&self) {}
    fn supports_graphics(&self) -> bool {
        !matches!(
            self.display.kind,
            DisplayKind::Terminal | DisplayKind::LedMatrix | DisplayKind::None
        )
    }
}

/// Mock accelerometer.
pub struct MockAccel {
    /// Static reading (giả lập)
    pub reading: AccelReading,
}

impl MockAccel {
    /// Stationary (gravity only: z = 9.8)
    pub fn stationary() -> Self {
        Self {
            reading: AccelReading {
                x: 0.0,
                y: 0.0,
                z: 9.8,
                ts: 0,
            },
        }
    }
    /// Shaking
    pub fn shaking() -> Self {
        Self {
            reading: AccelReading {
                x: 5.0,
                y: -3.0,
                z: 12.0,
                ts: 0,
            },
        }
    }
}

impl AccelDriver for MockAccel {
    fn read(&self) -> Option<AccelReading> {
        Some(self.reading)
    }
    fn sample_rate(&self) -> u16 {
        100
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_mock_pc() {
        let kb = MockKeyboard::pc();
        assert_eq!(kb.layout(), "us");
        assert!(kb.poll_key().is_none(), "Mock has no queued events");
    }

    #[test]
    fn keyboard_mock_mobile() {
        let kb = MockKeyboard::mobile();
        assert_eq!(kb.layout(), "vn_telex");
    }

    #[test]
    fn display_pc_1080p() {
        let disp = MockDisplay::pc_1080p();
        let info = disp.info();
        assert_eq!(info.width, 1920);
        assert_eq!(info.height, 1080);
        assert!(disp.supports_graphics());
    }

    #[test]
    fn display_smartphone_oled() {
        let disp = MockDisplay::smartphone_oled();
        let info = disp.info();
        assert_eq!(info.kind, DisplayKind::Oled);
        assert_eq!(info.refresh_hz, 120);
        assert!(disp.supports_graphics());
    }

    #[test]
    fn display_embedded_led() {
        let disp = MockDisplay::led_matrix();
        assert!(!disp.supports_graphics(), "LED matrix = no graphics");
    }

    #[test]
    fn display_terminal() {
        let disp = MockDisplay::terminal();
        assert!(!disp.supports_graphics(), "Terminal = text only");
    }

    #[test]
    fn accel_stationary_magnitude() {
        let accel = MockAccel::stationary();
        let reading = accel.read().unwrap();
        let mag = reading.magnitude();
        assert!((mag - 9.8).abs() < 0.1, "Gravity ≈ 9.8: {}", mag);
    }

    #[test]
    fn accel_shaking_magnitude() {
        let accel = MockAccel::shaking();
        let reading = accel.read().unwrap();
        let mag = reading.magnitude();
        assert!(mag > 10.0, "Shaking > gravity: {}", mag);
    }

    #[test]
    fn accel_sample_rate() {
        let accel = MockAccel::stationary();
        assert_eq!(accel.sample_rate(), 100);
    }

    #[test]
    fn input_event_key() {
        let ev = InputEvent::Key {
            code: 0x1B,
            pressed: true,
        }; // ESC
        match ev {
            InputEvent::Key { code, pressed } => {
                assert_eq!(code, 0x1B);
                assert!(pressed);
            }
            _ => panic!("Expected Key event"),
        }
    }

    #[test]
    fn input_event_touch() {
        let ev = InputEvent::Touch {
            x: 540,
            y: 1200,
            pressure: 128,
            finger: 0,
        };
        match ev {
            InputEvent::Touch {
                x,
                y,
                pressure,
                finger,
            } => {
                assert_eq!(x, 540);
                assert_eq!(y, 1200);
                assert_eq!(pressure, 128);
                assert_eq!(finger, 0);
            }
            _ => panic!("Expected Touch event"),
        }
    }

    #[test]
    fn gesture_pinch() {
        let g = GestureEvent::Pinch { scale: 2.0 };
        match g {
            GestureEvent::Pinch { scale } => assert_eq!(scale, 2.0),
            _ => panic!("Expected Pinch"),
        }
    }

    #[test]
    fn gps_location_struct() {
        let loc = GpsLocation {
            lat: 10.762622,
            lon: 106.660172,
            alt: 10.0,
            accuracy: 5.0,
            ts: 1000,
        };
        assert!((loc.lat - 10.762622).abs() < 0.0001);
        assert!((loc.lon - 106.660172).abs() < 0.0001);
    }

    #[test]
    fn proximity_near() {
        let prox = ProximityReading {
            distance_cm: 0.5,
            is_near: true,
            ts: 1000,
        };
        assert!(prox.is_near);
    }

    #[test]
    fn light_reading() {
        let light = LightReading {
            lux: 500.0,
            ts: 1000,
        };
        assert_eq!(light.lux, 500.0);
    }
}
