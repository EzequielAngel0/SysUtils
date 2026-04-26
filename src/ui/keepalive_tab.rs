use crate::app::SysUtilsApp;
use crate::logic::keepalive::KeepAliveLogic;
use eframe::egui;

impl SysUtilsApp {
    pub(crate) fn render_keepalive_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("♻ Session KeepAlive").size(18.0).color(egui::Color32::from_rgb(80, 200, 160)));
        ui.label(egui::RichText::new("Presiona teclas periódicamente para evitar AFK").size(11.0).color(egui::Color32::from_rgb(120, 120, 140)));
        ui.add_space(12.0);

        // ── Teclas ───────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Teclas:");
            if ui.add_sized([200.0, 20.0], egui::TextEdit::singleline(&mut self.config.keepalive_keys)).changed() {
                self.mark_dirty();
            }
        });
        ui.label(egui::RichText::new("  Teclas al azar para simular, separadas por coma. Ej: w,a,s,d").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
        ui.add_space(6.0);

        // ── Duración de presión ──────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Duración de presión (ms):");
            if ui.add(egui::DragValue::new(&mut self.config.keepalive_hold_ms).range(50..=5000)).changed() {
                self.mark_dirty();
            }
        });
        ui.label(egui::RichText::new("  Cuánto tiempo se mantiene pulsada la tecla.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
        ui.add_space(6.0);

        // ── Intervalo mínimo ─────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Intervalo mínimo (min):");
            if ui.add(egui::DragValue::new(&mut self.config.keepalive_interval_min).range(0.1..=60.0).speed(0.1)).changed() {
                self.mark_dirty();
            }
        });
        ui.label(egui::RichText::new("  Tiempo mínimo antes de la siguiente pulsación.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
        ui.add_space(6.0);

        // ── Intervalo máximo ─────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Intervalo máximo (min):");
            if ui.add(egui::DragValue::new(&mut self.config.keepalive_interval_max).range(0.1..=60.0).speed(0.1)).changed() {
                self.mark_dirty();
            }
        });
        ui.label(egui::RichText::new("  Tiempo máximo de espera para mayor aleatoriedad.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
        ui.add_space(6.0);

        // ── Hotkey ───────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Hotkey:");
            let hotkey_text = if self.config.keepalive_hotkey.is_empty() {
                "Ninguna".to_string()
            } else {
                self.config.keepalive_hotkey.clone()
            };
            let is_assigning = self.assigning_hotkey_for.as_deref() == Some("keepalive");
            let btn_text = if is_assigning { "Presiona tecla..." } else { &hotkey_text };
            let btn_color = if is_assigning { egui::Color32::from_rgb(255, 200, 100) } else { egui::Color32::WHITE };
            
            if ui.add_sized([120.0, 20.0], egui::Button::new(
                egui::RichText::new(btn_text).color(btn_color)
            )).clicked() {
                self.assigning_hotkey_for = Some("keepalive".to_string());
            }
        });
        ui.label(egui::RichText::new("  Haz clic para asignar una tecla. Soporta botones de ratón y teclado.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));

        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!(
                "Cada activación será entre {:.1} y {:.1} minutos (aleatorio)",
                self.config.keepalive_interval_min, self.config.keepalive_interval_max
            ))
            .size(11.0).italics().color(egui::Color32::from_rgb(120, 160, 140))
        );

        ui.add_space(12.0);

        let (text, color) = if self.keepalive_active {
            ("⏹ DETENER", egui::Color32::from_rgb(200, 50, 50))
        } else {
            ("▶ INICIAR", egui::Color32::from_rgb(50, 170, 70))
        };
        if ui.add_sized([200.0, 40.0], egui::Button::new(
            egui::RichText::new(text).size(14.0).strong().color(egui::Color32::WHITE)
        ).fill(color)).clicked() {
            if self.keepalive_active { self.stop_keepalive(); }
            else { self.config.save(); self.apply_hotkeys(); self.start_keepalive(); }
        }
    }
}
