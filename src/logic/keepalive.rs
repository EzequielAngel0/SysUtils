use crate::app::SysUtilsApp;
use crate::models::LogLevel;
use crate::logic::hardware::HardwareLogic;
use rand::Rng;
use std::time::{Duration, Instant};

pub trait KeepAliveLogic {
    fn start_keepalive(&mut self);
    fn stop_keepalive(&mut self);
}

impl KeepAliveLogic for SysUtilsApp {
    fn start_keepalive(&mut self) {
        if !self.hw.is_connected() {
            self.set_status("⚠ Hardware no conectado");
            return;
        }
        self.keepalive_active = true;
        self.config.keepalive_enabled = true;
        *self.keepalive_stop.lock().unwrap() = false;

        let hw = self.hw.clone();
        let stop_flag = self.keepalive_stop.clone();
        let logs = self.logs.clone();
        let keys: Vec<String> = self.config.keepalive_keys
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        let hold_ms = self.config.keepalive_hold_ms;
        let interval_min_ms = (self.config.keepalive_interval_min * 60_000.0) as u64;
        let interval_max_ms = (self.config.keepalive_interval_max * 60_000.0) as u64;

        std::thread::spawn(move || {
            let mut rng = rand::thread_rng();
            loop {
                // Randomize interval between min and max
                let wait_ms = rng.gen_range(interval_min_ms..=interval_max_ms.max(interval_min_ms));
                let wait_secs = wait_ms as f64 / 1000.0;
                logs.log(LogLevel::Info, "KeepAlive", &format!("Próxima activación en {:.0}s", wait_secs));

                let start = Instant::now();
                while start.elapsed().as_millis() < wait_ms as u128 {
                    if *stop_flag.lock().unwrap() { return; }
                    std::thread::sleep(Duration::from_millis(500));
                }
                if *stop_flag.lock().unwrap() { return; }

                for k in &keys {
                    if *stop_flag.lock().unwrap() { return; }
                    logs.log(LogLevel::Action, "KeepAlive", &format!("Presionando '{}'", k));
                    let _ = hw.send(&format!("KEY_DOWN:{}", k));
                    std::thread::sleep(Duration::from_millis(hold_ms as u64));
                    let _ = hw.send(&format!("KEY_UP:{}", k));
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        });

        self.set_status("♻ KeepAlive ACTIVO");
        self.logs.log(LogLevel::Action, "KeepAlive", &format!("Iniciado [{:.1}-{:.1} min]", self.config.keepalive_interval_min, self.config.keepalive_interval_max));
    }

    fn stop_keepalive(&mut self) {
        *self.keepalive_stop.lock().unwrap() = true;
        self.keepalive_active = false;
        self.config.keepalive_enabled = false;
        self.set_status("KeepAlive INACTIVO");
        self.logs.log(LogLevel::Action, "KeepAlive", "Detenido");
    }
}
