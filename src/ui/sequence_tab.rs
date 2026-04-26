use crate::app::SysUtilsApp;
use crate::models::{MacroEvent, LogLevel};
use crate::logic::sequence::SequenceLogic;
use eframe::egui;

impl SysUtilsApp {
    pub(crate) fn render_sequence_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("🎬 Sequence Flow").size(18.0).color(egui::Color32::from_rgb(255, 180, 80)));
        ui.label(egui::RichText::new("Graba y reproduce secuencias de teclado y ratón").size(11.0).color(egui::Color32::from_rgb(120, 120, 140)));
        ui.add_space(12.0);

        // ── Controls row ─────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            let (rec_text, rec_color) = if self.sequence_recording {
                ("⏹ Detener", egui::Color32::from_rgb(200, 50, 50))
            } else {
                ("⏺ Grabar", egui::Color32::from_rgb(200, 60, 60))
            };
            if ui.add_sized([100.0, 32.0], egui::Button::new(
                egui::RichText::new(rec_text).size(12.0).color(egui::Color32::WHITE)
            ).fill(rec_color)).clicked() {
                if self.sequence_recording { self.stop_recording(); }
                else { self.start_recording(); }
            }

            let play_enabled = !self.sequence_events.lock().unwrap().is_empty() && !self.sequence_recording;
            let (play_text, play_color) = if self.sequence_playing {
                ("⏹ Parar", egui::Color32::from_rgb(200, 50, 50))
            } else {
                ("▶ Reproducir", egui::Color32::from_rgb(50, 150, 70))
            };
            let play_btn = ui.add_sized([110.0, 32.0], egui::Button::new(
                egui::RichText::new(play_text).size(12.0).color(egui::Color32::WHITE)
            ).fill(if play_enabled || self.sequence_playing { play_color } else { egui::Color32::from_rgb(50, 50, 60) }));
            if play_btn.clicked() && (play_enabled || self.sequence_playing) {
                if self.sequence_playing { self.stop_playback(); }
                else { self.play_sequence(); }
            }
        });

        ui.add_space(8.0);

        // ── Loops ────────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Loops:");
            ui.add_sized([50.0, 20.0], egui::TextEdit::singleline(&mut self.sequence_loops));
        });
        ui.label(egui::RichText::new("  Cantidad de repeticiones (0 = infinito).").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
        ui.add_space(6.0);

        // ── Hotkey Grabar ────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Hotkey Grabar:");
            let hotkey_record_text = if self.config.sequence_hotkey_record.is_empty() {
                "Ninguna".to_string()
            } else {
                self.config.sequence_hotkey_record.clone()
            };
            let is_assigning_record = self.assigning_hotkey_for.as_deref() == Some("sequence_record");
            let btn_record_text = if is_assigning_record { "Presiona tecla..." } else { &hotkey_record_text };
            let btn_record_color = if is_assigning_record { egui::Color32::from_rgb(255, 200, 100) } else { egui::Color32::WHITE };

            if ui.add_sized([120.0, 20.0], egui::Button::new(
                egui::RichText::new(btn_record_text).color(btn_record_color)
            )).clicked() {
                self.assigning_hotkey_for = Some("sequence_record".to_string());
            }
        });
        ui.label(egui::RichText::new("  Haz clic para asignar la tecla de grabar. Soporta ratón y teclado.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
        ui.add_space(6.0);

        // ── Hotkey Reproducir ────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Hotkey Reproducir:");
            let hotkey_play_text = if self.config.sequence_hotkey_play.is_empty() {
                "Ninguna".to_string()
            } else {
                self.config.sequence_hotkey_play.clone()
            };
            let is_assigning_play = self.assigning_hotkey_for.as_deref() == Some("sequence_play");
            let btn_play_text = if is_assigning_play { "Presiona tecla..." } else { &hotkey_play_text };
            let btn_play_color = if is_assigning_play { egui::Color32::from_rgb(255, 200, 100) } else { egui::Color32::WHITE };

            if ui.add_sized([120.0, 20.0], egui::Button::new(
                egui::RichText::new(btn_play_text).color(btn_play_color)
            )).clicked() {
                self.assigning_hotkey_for = Some("sequence_play".to_string());
            }
        });
        ui.label(egui::RichText::new("  Haz clic para asignar la tecla de reproducir. Soporta ratón y teclado.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));

        ui.add_space(8.0);

        // ── Save / Load / Clear ──────────────────────────────────────────────
        ui.horizontal(|ui| {
            if ui.button("💾 Guardar").clicked() { self.save_sequence(); }
            if ui.button("📂 Cargar").clicked() { self.load_sequence(); }
            if ui.button("🗑 Limpiar").clicked() {
                self.sequence_events.lock().unwrap().clear();
                self.logs.log(LogLevel::Info, "Sequence", "Eventos limpiados");
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // ── Event list ───────────────────────────────────────────────────────
        let events = self.sequence_events.lock().unwrap().clone();
        ui.label(egui::RichText::new(format!("{} evento(s)", events.len())).size(12.0).color(egui::Color32::from_rgb(160, 160, 180)));

        if events.is_empty() {
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("Sin eventos grabados").size(14.0).color(egui::Color32::from_rgb(80, 80, 100)));
                ui.label(egui::RichText::new("Presiona ⏺ Grabar para empezar a capturar").size(11.0).color(egui::Color32::from_rgb(80, 80, 100)));
            });
        } else {
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                for (i, evt) in events.iter().enumerate() {
                    let txt = format!("{:4}  {}", i + 1, evt.display());
                    let color = match evt {
                        MacroEvent::Delay(_)     => egui::Color32::from_rgb(100, 100, 120),
                        MacroEvent::KeyDown(_)   => egui::Color32::from_rgb(140, 180, 255),
                        MacroEvent::KeyUp(_)     => egui::Color32::from_rgb(100, 140, 200),
                        MacroEvent::MouseDown(_) => egui::Color32::from_rgb(255, 160, 120),
                        MacroEvent::MouseUp(_)   => egui::Color32::from_rgb(200, 130, 100),
                        MacroEvent::MouseMove(_, _) => egui::Color32::from_rgb(80, 80, 100),
                    };
                    ui.label(egui::RichText::new(txt).size(11.0).monospace().color(color));
                }
            });
        }
    }
}
