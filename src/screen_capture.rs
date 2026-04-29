// ═══════════════════════════════════════════════════════════════════════════════
// screen_capture.rs — High-speed screen and window capture using xcap and image
// ═══════════════════════════════════════════════════════════════════════════════

use image::RgbaImage;

/// Info about an available display or window
#[derive(Debug, Clone)]
pub struct TargetInfo {
    pub id: usize,
    pub label: String,
    pub is_window: bool,
    #[allow(dead_code)]
    pub width: usize,
    #[allow(dead_code)]
    pub height: usize,
}

/// Result of a pixel comparison
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub avg_diff: f64,
    #[allow(dead_code)]
    pub max_diff: u32,
    pub triggered: bool,
}

/// RGBA color
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl PixelColor {
    pub fn distance(&self, other: &PixelColor) -> u32 {
        let dr = (self.r as i32 - other.r as i32).unsigned_abs();
        let dg = (self.g as i32 - other.g as i32).unsigned_abs();
        let db = (self.b as i32 - other.b as i32).unsigned_abs();
        dr + dg + db
    }
}

// ─── Backend trait ────────────────────────────────────────────────────────────

pub trait ScreenCaptureBackend: Send {
    fn list_displays(&self) -> Vec<TargetInfo>;
    fn list_windows(&self) -> Vec<TargetInfo>;
    fn capture_display(&self, id: usize) -> Option<RgbaImage>;
    fn capture_window(&self, id: usize) -> Option<RgbaImage>;
}

// ─── XcapBackend ─────────────────────────────────────────────────────────────

pub struct XcapBackend;

impl ScreenCaptureBackend for XcapBackend {
    fn list_displays(&self) -> Vec<TargetInfo> {
        use xcap::Monitor;
        let mut list = Vec::new();
        if let Ok(monitors) = Monitor::all() {
            for m in monitors {
                list.push(TargetInfo {
                    id: m.id().unwrap_or(0) as usize,
                    label: format!(
                        "🖥 Pantalla {} ({}x{})",
                        m.id().unwrap_or(0),
                        m.width().unwrap_or(0),
                        m.height().unwrap_or(0)
                    ),
                    is_window: false,
                    width: m.width().unwrap_or(0) as usize,
                    height: m.height().unwrap_or(0) as usize,
                });
            }
        }
        list
    }

    fn list_windows(&self) -> Vec<TargetInfo> {
        use xcap::Window;
        let mut list = Vec::new();
        if let Ok(windows) = Window::all() {
            for w in windows {
                let title = w.title().unwrap_or_default();
                let width = w.width().unwrap_or(0);
                let height = w.height().unwrap_or(0);
                if !title.is_empty() && width > 0 && height > 0 {
                    list.push(TargetInfo {
                        id: w.id().unwrap_or(0) as usize,
                        label: format!(
                            "🪟 {} - {}",
                            w.app_name().unwrap_or_default(),
                            title
                        ),
                        is_window: true,
                        width: width as usize,
                        height: height as usize,
                    });
                }
            }
        }
        list
    }

    fn capture_display(&self, id: usize) -> Option<RgbaImage> {
        use xcap::Monitor;
        if let Ok(monitors) = Monitor::all() {
            if let Some(m) = monitors.into_iter().find(|mon| mon.id().unwrap_or(0) as usize == id) {
                return m.capture_image().ok();
            }
            // Fallback to any available monitor
            if let Ok(mut mons) = Monitor::all() {
                if let Some(m) = mons.pop() {
                    return m.capture_image().ok();
                }
            }
        }
        None
    }

    fn capture_window(&self, id: usize) -> Option<RgbaImage> {
        use xcap::Window;
        if let Ok(windows) = Window::all() {
            if let Some(w) = windows.into_iter().find(|win| win.id().unwrap_or(0) as usize == id) {
                return w.capture_image().ok();
            }
        }
        None
    }
}

// ─── ScapBackend (placeholder — feature flag) ─────────────────────────────────

#[cfg(feature = "scap")]
pub struct ScapBackend;

#[cfg(feature = "scap")]
impl ScreenCaptureBackend for ScapBackend {
    fn list_displays(&self) -> Vec<TargetInfo> {
        Vec::new()
    }

    fn list_windows(&self) -> Vec<TargetInfo> {
        Vec::new()
    }

    fn capture_display(&self, _id: usize) -> Option<RgbaImage> {
        None
    }

    fn capture_window(&self, _id: usize) -> Option<RgbaImage> {
        None
    }
}

// ─── ScreenCapture ────────────────────────────────────────────────────────────

pub struct ScreenCapture {
    pub target_id: usize,
    pub is_window: bool,
    backend: Box<dyn ScreenCaptureBackend>,
}

impl ScreenCapture {
    pub fn new() -> Self {
        #[cfg(feature = "scap")]
        let backend: Box<dyn ScreenCaptureBackend> = Box::new(ScapBackend);
        #[cfg(not(feature = "scap"))]
        let backend: Box<dyn ScreenCaptureBackend> = Box::new(XcapBackend);

        Self {
            target_id: 0,
            is_window: false,
            backend,
        }
    }

    /// Create a ScreenCapture with a custom backend (useful for testing)
    #[allow(dead_code)]
    pub fn with_backend(backend: Box<dyn ScreenCaptureBackend>) -> Self {
        Self {
            target_id: 0,
            is_window: false,
            backend,
        }
    }

    /// List all available displays (delegates to XcapBackend by default)
    pub fn list_displays() -> Vec<TargetInfo> {
        XcapBackend.list_displays()
    }

    /// List all available windows (delegates to XcapBackend by default)
    pub fn list_windows() -> Vec<TargetInfo> {
        XcapBackend.list_windows()
    }

    /// Capture a frame from the current target
    pub fn capture_frame(&mut self) -> Option<RgbaImage> {
        if self.is_window {
            self.backend.capture_window(self.target_id)
        } else {
            self.backend.capture_display(self.target_id)
        }
    }

    pub fn get_pixel_color(frame: &RgbaImage, x: usize, y: usize) -> Option<PixelColor> {
        if x < frame.width() as usize && y < frame.height() as usize {
            let pixel = frame.get_pixel(x as u32, y as u32);
            Some(PixelColor {
                r: pixel[0],
                g: pixel[1],
                b: pixel[2],
            })
        } else {
            None
        }
    }

    /// Compare a specific region between two RgbaImage frames
    pub fn compare_region_pixels(
        baseline: &RgbaImage,
        current: &RgbaImage,
        rx: usize,
        ry: usize,
        rw: usize,
        rh: usize,
        sample_step: usize,
        threshold: f64,
    ) -> DiffResult {
        let mut total_diff: u64 = 0;
        let mut max_diff: u32 = 0;
        let mut samples: u64 = 0;
        let step = sample_step.max(1) as u32;

        let bw = baseline.width();
        let bh = baseline.height();
        let cw = current.width();
        let ch = current.height();

        let end_x = (rx as u32 + rw as u32).min(bw).min(cw);
        let end_y = (ry as u32 + rh as u32).min(bh).min(ch);
        let start_x = rx as u32;
        let start_y = ry as u32;

        if start_x >= end_x || start_y >= end_y {
            return DiffResult { avg_diff: 0.0, max_diff: 0, triggered: false };
        }

        for y in (start_y..end_y).step_by(step as usize) {
            for x in (start_x..end_x).step_by(step as usize) {
                let pb = baseline.get_pixel(x, y);
                let pc = current.get_pixel(x, y);

                let dr = (pb[0] as i32 - pc[0] as i32).unsigned_abs();
                let dg = (pb[1] as i32 - pc[1] as i32).unsigned_abs();
                let db = (pb[2] as i32 - pc[2] as i32).unsigned_abs();
                let pixel_diff = dr + dg + db;

                total_diff += pixel_diff as u64;
                if pixel_diff > max_diff { max_diff = pixel_diff; }
                samples += 1;
            }
        }

        let avg_diff = if samples > 0 { total_diff as f64 / samples as f64 } else { 0.0 };

        DiffResult {
            avg_diff,
            max_diff,
            triggered: avg_diff > threshold,
        }
    }
}
