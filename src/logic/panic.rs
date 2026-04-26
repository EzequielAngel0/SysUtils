use crate::app::SysUtilsApp;
use crate::models::LogLevel;
use crate::logic::hardware::HardwareLogic;
use crate::screen_capture::ScreenCapture;
use std::time::Duration;

pub trait PanicLogic {
    fn capture_panic_reference(&mut self);
    fn toggle_panic(&mut self);
}

impl PanicLogic for SysUtilsApp {
    fn capture_panic_reference(&mut self) {
        let reference = self.panic_reference_pixels.clone();
        let has_ref = self.panic_has_reference.clone();
        let status = self.panic_status.clone();
        let logs = self.logs.clone();

        std::thread::spawn(move || {
            *status.lock().unwrap() = "Capturando referencia en 3s...".into();
            logs.log(LogLevel::Info, "Panic", "Capturando referencia en 3 segundos...");
            std::thread::sleep(Duration::from_secs(3));

            let mut capturer = ScreenCapture::new();
            // Default to primary monitor
            capturer.target_id = 0;
            capturer.is_window = false;

            for _ in 0..20 {
                if let Some(frame) = capturer.capture_frame() {
                    *reference.lock().unwrap() = Some(frame.into_raw());
                    *has_ref.lock().unwrap() = true;
                    *status.lock().unwrap() = "✓ Referencia capturada".into();
                    logs.log(LogLevel::Action, "Panic", "Referencia de pantalla capturada exitosamente");
                    return;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            *status.lock().unwrap() = "Error: no se pudo capturar".into();
        });
    }

    fn toggle_panic(&mut self) {
        if self.panic_active {
            *self.panic_stop.lock().unwrap() = true;
            self.panic_active = false;
            self.config.panic_enabled = false;
            self.set_status("Panic Switch INACTIVO");
            self.logs.log(LogLevel::Action, "Panic", "Desactivado");
            return;
        }

        if !*self.panic_has_reference.lock().unwrap() {
            self.set_status("⚠ Primero captura una referencia");
            self.logs.log(LogLevel::Warning, "Panic", "No hay referencia capturada");
            return;
        }

        self.panic_active = true;
        self.config.panic_enabled = true;
        *self.panic_stop.lock().unwrap() = false;

        let stop_flag = self.panic_stop.clone();
        let panic_status = self.panic_status.clone();
        let hw = self.hw.clone();
        let logs = self.logs.clone();
        let threshold = self.config.panic_threshold;
        let check_interval = self.config.panic_check_interval_ms;
        let reference = self.panic_reference_pixels.lock().unwrap().clone();

        std::thread::spawn(move || {
            let baseline = match reference {
                Some(r) => r,
                None => {
                    *panic_status.lock().unwrap() = "Error: sin referencia".into();
                    return;
                }
            };

            let mut capturer = ScreenCapture::new();
            capturer.target_id = 0;
            capturer.is_window = false;

            *panic_status.lock().unwrap() = "🛡 Vigilando...".into();
            logs.log(LogLevel::Info, "Panic", &format!("Vigilando pantalla (umbral: {:.0})", threshold));

            loop {
                if *stop_flag.lock().unwrap() { break; }
                std::thread::sleep(Duration::from_millis(check_interval));
                if *stop_flag.lock().unwrap() { break; }

                if let Some(current) = capturer.capture_frame() {
                    let w = current.width() as usize;
                    let h = current.height() as usize;
                    // Provide a valid RgbaImage for baseline
                    if let Some(baseline_img) = image::RgbaImage::from_raw(w as u32, h as u32, baseline.clone()) {
                        let diff = ScreenCapture::compare_region_pixels(&baseline_img, &current, 0, 0, w, h, 50, threshold);
                        if diff.triggered {
                        logs.log(LogLevel::Warning, "Panic", &format!("⚠ CAMBIO DETECTADO (diff: {:.1}) — Soltando todo", diff.avg_diff));
                        let _ = hw.send("STOP");
                        let _ = hw.send("CLK_UP:L");
                        let _ = hw.send("CLK_UP:R");
                        *panic_status.lock().unwrap() = format!("⚠ ALERTA (diff: {:.1})", diff.avg_diff);
                        std::thread::sleep(Duration::from_secs(5));
                        *panic_status.lock().unwrap() = "🛡 Vigilando...".into();
                    }
                    }
                }
            }
            *panic_status.lock().unwrap() = "Inactivo".into();
        });

        self.set_status("🛡 Panic Switch ACTIVO");
        self.logs.log(LogLevel::Action, "Panic", "Activado");
    }
}
