use crate::app::SysUtilsApp;
use crate::models::LogLevel;
use crate::logic::hardware::HardwareLogic;

pub trait PulseLogic {
    fn toggle_pulse(&mut self);
}

impl PulseLogic for SysUtilsApp {
    fn toggle_pulse(&mut self) {
        if !self.hw.is_connected() {
            self.set_status("⚠ Hardware no conectado");
            self.logs.log(LogLevel::Warning, "Pulse", "Intento sin hardware conectado");
            return;
        }
        self.pulse_active = !self.pulse_active;
        self.config.pulse_enabled = self.pulse_active;
        if self.pulse_active {
            // Configure based on input type
            if self.config.pulse_input_type == "keyboard" {
                let _ = self.hw.send(&format!("PULSE_KEY:{}", self.config.pulse_key));
                let _ = self.hw.send("START");
                self.set_status("⚡ Pulse ACTIVO (Teclado)");
                self.logs.log(LogLevel::Action, "Pulse", &format!(
                    "Activado [{}ms-{}ms] tecla={} modo={}",
                    self.config.pulse_min_delay, self.config.pulse_max_delay,
                    self.config.pulse_key, self.config.pulse_mode
                ));
            } else {
                let _ = self.hw.send("START");
                self.set_status("⚡ Pulse ACTIVO (Ratón)");
                self.logs.log(LogLevel::Action, "Pulse", &format!(
                    "Activado [{}ms-{}ms] btn={} modo={}",
                    self.config.pulse_min_delay, self.config.pulse_max_delay,
                    self.config.pulse_target, self.config.pulse_mode
                ));
            }
        } else {
            let _ = self.hw.send("STOP");
            self.set_status("Pulse INACTIVO");
            self.logs.log(LogLevel::Action, "Pulse", "Detenido");
        }
    }
}
