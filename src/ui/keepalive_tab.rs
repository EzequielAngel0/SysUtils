use crate::app::SysUtilsApp;
use crate::logic::keepalive::KeepAliveLogic;
use crate::logic::hardware::HardwareLogic;
use eframe::egui;

impl SysUtilsApp {
    pub(crate) fn render_keepalive_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("♻ Session KeepAlive").size(18.0).color(egui::Color32::from_rgb(80, 200, 160)));
        ui.label(egui::RichText::new("Presiona teclas periódicamente para evitar AFK").size(11.0).color(egui::Color32::from_rgb(120, 120, 140)));
        ui.add_space(12.0);

        let running = self.keepalive_active;

        if running {
            ui.label(egui::RichText::new("⚠ Detén el módulo para modificar la configuración.")
                .size(11.0).color(egui::Color32::from_rgb(220, 180, 60)));
            ui.add_space(6.0);
        }

        ui.add_enabled_ui(!running, |ui| {
            // ── Teclas ───────────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Teclas:");
                if ui.add_sized([200.0, 20.0], egui::TextEdit::singleline(&mut self.config.keepalive_keys)).changed() {
                    self.mark_dirty();
                }
            });
            ui.label(egui::RichText::new("  Teclas al azar para simular, separadas por coma. Ej: w,a,s,d").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
            ui.add_space(6.0);

            // ── Duración de presión ──────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Duración de presión (ms):");
                if ui.add(egui::DragValue::new(&mut self.config.keepalive_hold_ms).range(50..=5000)).changed() {
                    self.mark_dirty();
                }
            });
            ui.label(egui::RichText::new("  Cuánto tiempo se mantiene pulsada la tecla.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
            ui.add_space(6.0);

            // ── Intervalo mínimo ─────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Intervalo mínimo (min):");
                if ui.add(egui::DragValue::new(&mut self.config.keepalive_interval_min).range(0.1..=60.0).speed(0.1)).changed() {
                    self.mark_dirty();
                }
            });
            ui.label(egui::RichText::new("  Tiempo mínimo antes de la siguiente pulsación.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
            ui.add_space(6.0);

            // ── Intervalo máximo ─────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Intervalo máximo (min):");
                if ui.add(egui::DragValue::new(&mut self.config.keepalive_interval_max).range(0.1..=60.0).speed(0.1)).changed() {
                    self.mark_dirty();
                }
            });
            ui.label(egui::RichText::new("  Tiempo máximo de espera para mayor aleatoriedad.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
            ui.add_space(6.0);

            // ── Hotkey ───────────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Hotkey:");
                let is_valid = crate::hotkey_engine::HotkeyEngine::is_valid_key(&self.config.keepalive_hotkey);
                let hotkey_text = if self.config.keepalive_hotkey.is_empty() { "Ninguna".to_string() } else { self.config.keepalive_hotkey.clone() };
                let is_assigning = self.assigning_hotkey_for.as_deref() == Some("keepalive");
                let btn_text = if is_assigning { "Presiona tecla..." } else { &hotkey_text };
                let btn_color = if is_assigning { egui::Color32::from_rgb(255, 200, 100) } else { egui::Color32::WHITE };
                let mut frame = egui::Frame::new();
                if !is_valid { frame = frame.fill(egui::Color32::from_rgba_premultiplied(200, 60, 60, 40)); }
                frame.show(ui, |ui| {
                    let mut btn = ui.add_sized([120.0, 20.0], egui::Button::new(egui::RichText::new(btn_text).color(btn_color)));
                    if !is_valid { btn = btn.on_hover_text("Hotkey inválida — usa formato: F6, Ctrl+F6, MouseLeft"); }
                    if btn.clicked() { self.start_assigning_hotkey("keepalive"); }
                });
                let clear_btn = ui.add_enabled(!self.config.keepalive_hotkey.is_empty(), egui::Button::new("✕"));
                if clear_btn.on_hover_text("Quitar hotkey").clicked() {
                    self.config.keepalive_hotkey.clear();
                    self.apply_hotkeys();
                    self.mark_dirty();
                }
            });
            ui.label(egui::RichText::new("  Haz clic para asignar una tecla. Soporta botones de ratón y teclado.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
        }); // end add_enabled_ui

        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!(
                "Cada activación será entre {:.1} y {:.1} minutos (aleatorio)",
                self.config.keepalive_interval_min, self.config.keepalive_interval_max
            ))
            .size(11.0).italics().color(egui::Color32::from_rgb(120, 160, 140))
        );

        ui.add_space(12.0);

        // ── Acciones ─────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            // Botón Aplicar Config (siempre visible)
            if ui.add_sized([160.0, 36.0], egui::Button::new(
                egui::RichText::new("Aplicar Config").size(13.0).color(egui::Color32::WHITE)
            ).fill(egui::Color32::from_rgb(60, 40, 140))).clicked() {
                self.config.save();
                self.apply_hotkeys();
                self.set_status("✓ Config aplicada");
            }

            let (text, color) = if running {
                ("⏹ DETENER", egui::Color32::from_rgb(200, 50, 50))
            } else {
                ("▶ INICIAR", egui::Color32::from_rgb(50, 170, 70))
            };
            if ui.add_sized([200.0, 40.0], egui::Button::new(
                egui::RichText::new(text).size(14.0).strong().color(egui::Color32::WHITE)
            ).fill(color)).clicked() {
                if running { self.stop_keepalive(); }
                else { self.config.save(); self.apply_hotkeys(); self.start_keepalive(); }
            }
        });
    }
}
