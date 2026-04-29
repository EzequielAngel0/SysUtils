use crate::{app::SysUtilsApp, models::LogLevel};
use crate::logic::pulse::PulseLogic;
use crate::logic::hardware::HardwareLogic;
use eframe::egui;

impl SysUtilsApp {
    pub(crate) fn render_pulse_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("⚡ Pulse Emitter").size(18.0).color(egui::Color32::from_rgb(160, 120, 255)));
        ui.label(egui::RichText::new("Emulación de clics/teclas automáticas vía ESP32").size(11.0).color(egui::Color32::from_rgb(120, 120, 140)));
        ui.add_space(12.0);

        let connected = self.hw.is_connected();
        let running = self.pulse_active;

        if running {
            ui.label(egui::RichText::new("⚠ Detén el módulo para modificar la configuración.")
                .size(11.0).color(egui::Color32::from_rgb(220, 180, 60)));
            ui.add_space(6.0);
        }

        ui.add_enabled_ui(!running, |ui| {
            // ── Delay mínimo ─────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Delay mínimo (ms):");
                if ui.add(egui::DragValue::new(&mut self.config.pulse_min_delay).range(1..=5000)).changed() {
                    self.mark_dirty();
                    if connected { self.send_pulse_config(); }
                }
            });
            ui.label(egui::RichText::new("  Tiempo mínimo entre acciones consecutivas.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
            ui.add_space(6.0);

            // ── Delay máximo ─────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Delay máximo (ms):");
                if ui.add(egui::DragValue::new(&mut self.config.pulse_max_delay).range(1..=5000)).changed() {
                    self.mark_dirty();
                    if connected { self.send_pulse_config(); }
                }
            });
            ui.label(egui::RichText::new("  Tiempo máximo entre acciones consecutivas.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
            ui.add_space(6.0);

            // ── Tipo de Input ────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Tipo de input:");
                let mut changed = false;
                if ui.selectable_value(&mut self.config.pulse_input_type, "mouse".into(), "🖱 Ratón").changed() { changed = true; }
                if ui.selectable_value(&mut self.config.pulse_input_type, "keyboard".into(), "⌨ Teclado").changed() { changed = true; }
                if changed { self.mark_dirty(); if connected { self.send_pulse_config(); } }
            });
            ui.label(egui::RichText::new("  Selecciona si el pulse emula clics de ratón o pulsaciones de teclado.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
            ui.add_space(6.0);

            if self.config.pulse_input_type == "keyboard" {
                ui.horizontal(|ui| {
                    ui.label("Tecla:");
                    if ui.add_sized([80.0, 20.0], egui::TextEdit::singleline(&mut self.config.pulse_key)
                        .hint_text("e, f, space...")).changed() {
                        self.mark_dirty();
                        if connected { self.send_pulse_config(); }
                    }
                });
                ui.label(egui::RichText::new("  Letra, número o tecla especial a pulsar (ej: e, f, space, enter).").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
            } else {
                ui.horizontal(|ui| {
                    ui.label("Botón:");
                    let mut changed = false;
                    if ui.selectable_value(&mut self.config.pulse_target, "L".into(), "Izquierdo").changed() { changed = true; }
                    if ui.selectable_value(&mut self.config.pulse_target, "R".into(), "Derecho").changed() { changed = true; }
                    if ui.selectable_value(&mut self.config.pulse_target, "M".into(), "Medio").changed() { changed = true; }
                    if changed { self.mark_dirty(); if connected { self.send_pulse_config(); } }
                });
                ui.label(egui::RichText::new("  Qué botón del ratón se simulará.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
            }
            ui.add_space(6.0);

            // ── Modo ─────────────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Modo:");
                let mut changed = false;
                if ui.selectable_value(&mut self.config.pulse_mode, "PULSE".into(), "Spam").changed() { changed = true; }
                if ui.selectable_value(&mut self.config.pulse_mode, "HOLD".into(), "Hold").changed() { changed = true; }
                if changed { self.mark_dirty(); if connected { self.send_pulse_config(); } }
            });
            ui.label(egui::RichText::new("  Spam = Repeticiones rápidas, Hold = Mantener pulsado.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
            ui.add_space(6.0);

            // ── Hotkey ───────────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Hotkey:");
                let is_valid = crate::hotkey_engine::HotkeyEngine::is_valid_key(&self.config.pulse_hotkey);
                let hotkey_text = if self.config.pulse_hotkey.is_empty() { "Ninguna".to_string() } else { self.config.pulse_hotkey.clone() };
                let is_assigning = self.assigning_hotkey_for.as_deref() == Some("pulse");
                let btn_text = if is_assigning { "Presiona tecla..." } else { &hotkey_text };
                let btn_color = if is_assigning { egui::Color32::from_rgb(255, 200, 100) } else { egui::Color32::WHITE };
                let mut frame = egui::Frame::new();
                if !is_valid { frame = frame.fill(egui::Color32::from_rgba_premultiplied(200, 60, 60, 40)); }
                frame.show(ui, |ui| {
                    let mut btn = ui.add_sized([120.0, 20.0], egui::Button::new(egui::RichText::new(btn_text).color(btn_color)));
                    if !is_valid { btn = btn.on_hover_text("Hotkey inválida — usa formato: F6, Ctrl+F6, MouseLeft"); }
                    if btn.clicked() { self.start_assigning_hotkey("pulse"); }
                });
                let clear_btn = ui.add_enabled(!self.config.pulse_hotkey.is_empty(), egui::Button::new("✕"));
                if clear_btn.on_hover_text("Quitar hotkey").clicked() {
                    self.config.pulse_hotkey.clear();
                    self.apply_hotkeys();
                    self.mark_dirty();
                }
            });
            ui.label(egui::RichText::new("  Haz clic para asignar una tecla. Soporta botones de ratón y teclado.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
        }); // end add_enabled_ui

        if connected && !running {
            ui.add_space(4.0);
            ui.label(egui::RichText::new("  ✓ Cambios se aplican al ESP32 automáticamente.")
                .size(10.0).color(egui::Color32::from_rgb(80, 200, 120)));
        }

        ui.add_space(16.0);

        // ── Acciones ─────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            if ui.add_sized([160.0, 36.0], egui::Button::new(
                egui::RichText::new("Aplicar Config").size(13.0).color(egui::Color32::WHITE)
            ).fill(egui::Color32::from_rgb(60, 40, 140))).clicked() {
                self.send_pulse_config();
                self.config.save();
                self.apply_hotkeys();
                self.set_status("✓ Config aplicada");
                self.logs.log(LogLevel::Info, "Pulse", "Configuración aplicada al ESP32");
            }

            let (text, color) = if running {
                ("⏹ DETENER", egui::Color32::from_rgb(200, 50, 50))
            } else {
                ("▶ INICIAR", egui::Color32::from_rgb(50, 170, 70))
            };
            if ui.add_sized([140.0, 36.0], egui::Button::new(
                egui::RichText::new(text).size(14.0).strong().color(egui::Color32::WHITE)
            ).fill(color)).clicked() {
                if !running {
                    if connected { self.send_pulse_config(); }
                    self.config.save();
                    self.apply_hotkeys();
                }
                self.toggle_pulse();
            }
        });
    }

    fn send_pulse_config(&self) {
        let _ = self.hw.send(&format!("DELAY:{}:{}", self.config.pulse_min_delay, self.config.pulse_max_delay));
        if self.config.pulse_input_type == "keyboard" {
            let _ = self.hw.send(&format!("PULSE_KEY:{}", self.config.pulse_key));
        } else {
            let _ = self.hw.send(&format!("TARGET_BTN:{}", self.config.pulse_target));
        }
        let _ = self.hw.send(&format!("MODE:{}", self.config.pulse_mode));
    }
}
