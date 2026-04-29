use crate::app::SysUtilsApp;
use crate::models::LogLevel;
use crate::logic::hardware::HardwareLogic;
use crate::screen_capture::{ScreenCapture, PixelColor};
use std::time::Duration;

pub trait MonitorLogic {
    fn capture_monitor_reference(&mut self);
    fn start_monitor(&mut self);
    fn stop_monitor(&mut self);
}

impl MonitorLogic for SysUtilsApp {
    fn capture_monitor_reference(&mut self) {
        let reference = self.monitor_reference_pixels.clone();
        let has_ref = self.monitor_has_reference.clone();
        let status = self.monitor_status.clone();
        let logs = self.logs.clone();
        let target_id = self.monitor_target_id;
        let is_window = self.monitor_is_window;
        let ref_w = self.monitor_ref_width.clone();
        let ref_h = self.monitor_ref_height.clone();

        std::thread::spawn(move || {
            *status.lock().unwrap() = "Capturando en 3s...".into();
            logs.log(LogLevel::Info, "Monitor", "Capturando referencia en 3 segundos...");
            std::thread::sleep(Duration::from_secs(3));

            let mut capturer = ScreenCapture::new();
            capturer.target_id = target_id;
            capturer.is_window = is_window;

            for _ in 0..30 {
                if let Some(frame) = capturer.capture_frame() {
                    *ref_w.lock().unwrap() = frame.width() as usize;
                    *ref_h.lock().unwrap() = frame.height() as usize;
                    *reference.lock().unwrap() = Some(frame.into_raw());
                    *has_ref.lock().unwrap() = true;
                    *status.lock().unwrap() = "✓ Referencia capturada".into();
                    logs.log(LogLevel::Action, "Monitor", "Referencia capturada");
                    return;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            *status.lock().unwrap() = "Error: no se pudo capturar".into();
        });
        
        // Start preview thread (F2)
        *self.monitor_preview_stop.lock().unwrap() = false;
        
        let preview_pixels = self.monitor_preview_pixels.clone();
        let preview_w = self.monitor_preview_width.clone();
        let preview_h = self.monitor_preview_height.clone();
        let preview_stop = self.monitor_preview_stop.clone();
        let preview_gen = self.monitor_preview_generation.clone();
        let target_id = self.monitor_target_id;
        let is_window = self.monitor_is_window;
        let logs = self.logs.clone();
        
        std::thread::spawn(move || {
            let mut capturer = ScreenCapture::new();
            capturer.target_id = target_id;
            capturer.is_window = is_window;
            
            loop {
                if *preview_stop.lock().unwrap() { break; }
                
                if let Some(frame) = capturer.capture_frame() {
                    *preview_w.lock().unwrap() = frame.width() as usize;
                    *preview_h.lock().unwrap() = frame.height() as usize;
                    *preview_pixels.lock().unwrap() = Some(frame.into_raw());
                    *preview_gen.lock().unwrap() += 1;
                } else {
                    logs.log(LogLevel::Warning, "Monitor", "Preview capture failed");
                }
                
                std::thread::sleep(Duration::from_millis(500));
            }
        });
    }

    fn start_monitor(&mut self) {
        if !self.hw.is_connected() {
            self.set_status("⚠ Hardware no conectado");
            return;
        }
        if !*self.monitor_has_reference.lock().unwrap() {
            self.set_status("⚠ Primero captura una referencia");
            self.logs.log(LogLevel::Warning, "Monitor", "No hay referencia capturada");
            return;
        }

        self.monitor_active = true;
        self.config.monitor_enabled = true;
        *self.monitor_stop.lock().unwrap() = false;

        let stop_flag = self.monitor_stop.clone();
        let status = self.monitor_status.clone();
        let hw = self.hw.clone();
        let logs = self.logs.clone();
        
        let tolerance = self.config.monitor_tolerance;
        let sample_step = self.config.monitor_sample_step;
        let duration_ms = self.config.monitor_duration_ms;
        let target_id = self.monitor_target_id;
        let is_window = self.monitor_is_window;
        let monitor_mode = self.config.monitor_mode.clone();
        let click_action = self.config.monitor_click_action.clone();
        let condition = self.config.monitor_condition.clone();
        let action_type = self.config.monitor_action_type.clone();
        let action_key = self.config.monitor_action_key.clone();
        
        // Target color for condition modes
        let target_color = PixelColor {
            r: self.config.monitor_target_color_r,
            g: self.config.monitor_target_color_g,
            b: self.config.monitor_target_color_b,
        };
        let color_tolerance = self.config.monitor_color_tolerance as u32;
        
        let pixel_x = self.config.monitor_pixel_x as usize;
        let pixel_y = self.config.monitor_pixel_y as usize;
        let region_x = self.config.monitor_region_x as usize;
        let region_y = self.config.monitor_region_y as usize;
        let region_w = self.config.monitor_region_w as usize;
        let region_h = self.config.monitor_region_h as usize;
        
        let baseline_raw = self.monitor_reference_pixels.lock().unwrap().clone();
        let ref_width = *self.monitor_ref_width.lock().unwrap();
        let ref_height = *self.monitor_ref_height.lock().unwrap();

        std::thread::spawn(move || {
            let reference_bytes = match baseline_raw {
                Some(r) => r,
                None => {
                    *status.lock().unwrap() = "Error: sin referencia".into();
                    return;
                }
            };
            
            let mut baseline_img = match image::RgbaImage::from_raw(ref_width as u32, ref_height as u32, reference_bytes) {
                Some(img) => img,
                None => {
                    *status.lock().unwrap() = "Error: buffer de referencia inválido".into();
                    return;
                }
            };

            let mut capturer = ScreenCapture::new();
            capturer.target_id = target_id;
            capturer.is_window = is_window;

            *status.lock().unwrap() = "Monitoreando...".into();
            let msg = match (monitor_mode.as_str(), condition.as_str()) {
                ("PIXEL", "color_appear") => format!("Pixel ({},{}) — esperando color RGB({},{},{})", pixel_x, pixel_y, target_color.r, target_color.g, target_color.b),
                ("PIXEL", "color_disappear") => format!("Pixel ({},{}) — esperando desaparición de color", pixel_x, pixel_y),
                ("PIXEL", _) => format!("Monitoreando pixel ({}, {})", pixel_x, pixel_y),
                ("REGION", _) => format!("Monitoreando región {}x{} [{}]", region_w, region_h, condition),
                _ => format!("Monitoreando completa [{}]", condition),
            };
            logs.log(LogLevel::Info, "Monitor", &msg);

            // Action helper
            let execute_action = |hw: &std::sync::Arc<crate::hw_link::HwLink>, logs: &crate::models::LogBuffer| {
                if action_type == "key_press" {
                    logs.log(LogLevel::Action, "Monitor", &format!("¡Cambio detectado! → Tecla '{}'", action_key));
                    let _ = hw.send(&format!("KEY_DOWN:{}", action_key));
                    std::thread::sleep(Duration::from_millis(duration_ms as u64));
                    let _ = hw.send(&format!("KEY_UP:{}", action_key));
                } else {
                    let hw_click_char = match click_action.as_str() {
                        "Right" => "R",
                        "Middle" => "M",
                        _ => "L",
                    };
                    logs.log(LogLevel::Action, "Monitor", "¡Cambio detectado! → Clic ejecutado");
                    let _ = hw.send(&format!("CLK_DOWN:{}", hw_click_char));
                    std::thread::sleep(Duration::from_millis(duration_ms as u64));
                    let _ = hw.send(&format!("CLK_UP:{}", hw_click_char));
                }
            };

            loop {
                if *stop_flag.lock().unwrap() { break; }
                std::thread::sleep(Duration::from_millis(50));
                if *stop_flag.lock().unwrap() { break; }

                if let Some(current_img) = capturer.capture_frame() {
                    let triggered = match condition.as_str() {
                        // ── Color Appear: target color is NOW present ──────────
                        "color_appear" => {
                            match monitor_mode.as_str() {
                                "PIXEL" => {
                                    if let Some(cp) = ScreenCapture::get_pixel_color(&current_img, pixel_x, pixel_y) {
                                        cp.distance(&target_color) <= color_tolerance
                                    } else { false }
                                }
                                "REGION" => {
                                    has_color_in_region(&current_img, &target_color, color_tolerance,
                                        region_x, region_y, region_w, region_h, sample_step)
                                }
                                _ => {
                                    has_color_in_region(&current_img, &target_color, color_tolerance,
                                        0, 0, ref_width, ref_height, sample_step)
                                }
                            }
                        }
                        // ── Color Disappear: target color WAS present in reference but is GONE now ──
                        "color_disappear" => {
                            match monitor_mode.as_str() {
                                "PIXEL" => {
                                    // PIXEL mode: fixed reference (no sliding)
                                    if let Some(cp) = ScreenCapture::get_pixel_color(&current_img, pixel_x, pixel_y) {
                                        // Was close to target in reference, now it's different
                                        if let Some(bp) = ScreenCapture::get_pixel_color(&baseline_img, pixel_x, pixel_y) {
                                            bp.distance(&target_color) <= color_tolerance && cp.distance(&target_color) > color_tolerance
                                        } else { false }
                                    } else { false }
                                }
                                "REGION" => {
                                    // REGION mode: sliding reference
                                    let is_present = has_color_in_region(&current_img, &target_color, color_tolerance,
                                        region_x, region_y, region_w, region_h, sample_step);
                                    
                                    if is_present {
                                        // Color IS present → update sliding reference
                                        baseline_img = current_img.clone();
                                        logs.log(LogLevel::Info, "Monitor", "Referencia actualizada (color presente)");
                                        false  // Don't trigger yet
                                    } else {
                                        // Color is NOT present → check if it WAS present in baseline
                                        let was_present = has_color_in_region(&baseline_img, &target_color, color_tolerance,
                                            region_x, region_y, region_w, region_h, sample_step);
                                        was_present  // Trigger if it disappeared
                                    }
                                }
                                _ => {
                                    // FULLSCREEN mode: sliding reference
                                    let is_present = has_color_in_region(&current_img, &target_color, color_tolerance,
                                        0, 0, ref_width, ref_height, sample_step);
                                    
                                    if is_present {
                                        // Color IS present → update sliding reference
                                        baseline_img = current_img.clone();
                                        logs.log(LogLevel::Info, "Monitor", "Referencia actualizada (color presente)");
                                        false  // Don't trigger yet
                                    } else {
                                        // Color is NOT present → check if it WAS present in baseline
                                        let was_present = has_color_in_region(&baseline_img, &target_color, color_tolerance,
                                            0, 0, ref_width, ref_height, sample_step);
                                        was_present  // Trigger if it disappeared
                                    }
                                }
                            }
                        }
                        // ── Change (default): any pixel difference above tolerance ──
                        _ => {
                            match monitor_mode.as_str() {
                                "PIXEL" => {
                                    if let (Some(bp), Some(cp)) = (
                                        ScreenCapture::get_pixel_color(&baseline_img, pixel_x, pixel_y),
                                        ScreenCapture::get_pixel_color(&current_img, pixel_x, pixel_y)
                                    ) {
                                        bp.distance(&cp) > tolerance as u32
                                    } else { false }
                                }
                                "REGION" => {
                                    let diff = ScreenCapture::compare_region_pixels(&baseline_img, &current_img,
                                        region_x, region_y, region_w, region_h, sample_step, tolerance as f64);
                                    diff.triggered
                                }
                                _ => {
                                    let diff = ScreenCapture::compare_region_pixels(&baseline_img, &current_img,
                                        0, 0, ref_width, ref_height, sample_step, tolerance as f64);
                                    diff.triggered
                                }
                            }
                        }
                    };

                    if triggered {
                        execute_action(&hw, &logs);
                        *status.lock().unwrap() = "Acción ejecutada".into();
                        std::thread::sleep(Duration::from_secs(1));
                        *status.lock().unwrap() = "Monitoreando...".into();
                    }
                }
            }
            *status.lock().unwrap() = "Inactivo".into();
        });

        self.logs.log(LogLevel::Action, "Monitor", "Iniciado");
        self.set_status("👁 Monitor ACTIVO");
    }

    fn stop_monitor(&mut self) {
        *self.monitor_stop.lock().unwrap() = true;
        *self.monitor_preview_stop.lock().unwrap() = true; // F2: Stop preview thread
        self.monitor_active = false;
        self.config.monitor_enabled = false;
        self.logs.log(LogLevel::Action, "Monitor", "Detenido");
        self.set_status("Monitor INACTIVO");
    }
}

/// Check if a target color exists anywhere in a region
fn has_color_in_region(
    img: &image::RgbaImage,
    target: &PixelColor,
    tolerance: u32,
    rx: usize, ry: usize, rw: usize, rh: usize,
    sample_step: usize,
) -> bool {
    let step = sample_step.max(1) as u32;
    let end_x = (rx as u32 + rw as u32).min(img.width());
    let end_y = (ry as u32 + rh as u32).min(img.height());
    let start_x = rx as u32;
    let start_y = ry as u32;

    for y in (start_y..end_y).step_by(step as usize) {
        for x in (start_x..end_x).step_by(step as usize) {
            let p = img.get_pixel(x, y);
            let c = PixelColor { r: p[0], g: p[1], b: p[2] };
            if c.distance(target) <= tolerance {
                return true;
            }
        }
    }
    false
}
