use crate::app::SysUtilsApp;
use crate::models::LogLevel;
use std::time::Instant;

pub trait HardwareLogic {
    fn set_status(&mut self, msg: &str);
    fn refresh_ports(&mut self);
    fn connect(&mut self);
    fn disconnect(&mut self);
}

impl HardwareLogic for SysUtilsApp {
    fn set_status(&mut self, msg: &str) {
        self.status_message = msg.to_string();
        self.status_timestamp = Instant::now();
    }

    fn refresh_ports(&mut self) {
        self.available_ports = crate::hw_link::HwLink::available_ports();
        self.logs.log(LogLevel::Info, "Serial", &format!("{} puerto(s) encontrado(s)", self.available_ports.len()));
    }

    fn connect(&mut self) {
        if self.selected_port.is_empty() {
            self.set_status("⚠ Selecciona un puerto");
            return;
        }
        match self.hw.connect(&self.selected_port) {
            Ok(_) => {
                self.config.last_port = self.selected_port.clone();
                self.config.save();
                self.set_status(&format!("✓ Conectado a {}", self.selected_port));
                self.logs.log(LogLevel::Action, "Serial", &format!("Conectado a {}", self.selected_port));

                let _ = self.hw.send(&format!("DELAY:{}:{}", self.config.pulse_min_delay, self.config.pulse_max_delay));
                let _ = self.hw.send(&format!("TARGET_BTN:{}", self.config.pulse_target));
                let _ = self.hw.send(&format!("MODE:{}", self.config.pulse_mode));
            }
            Err(e) => {
                self.set_status(&format!("✗ {}", e));
                self.logs.log(LogLevel::Error, "Serial", &format!("Error: {}", e));
            }
        }
    }

    fn disconnect(&mut self) {
        self.hw.disconnect();
        self.pulse_active = false;
        self.set_status("Desconectado");
        self.logs.log(LogLevel::Info, "Serial", "Desconectado");
    }
}
