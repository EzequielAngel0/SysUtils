use crate::app::SysUtilsApp;
use crate::logic::panic::PanicLogic;
use crate::logic::hardware::HardwareLogic;
use eframe::egui;

impl SysUtilsApp {
    pub(crate) fn render_panic_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("🛡 Panic Switch").size(18.0).color(egui::Color32::from_rgb(255, 100, 100)));
        ui.label(egui::RichText::new("Detecta cambios en la pantalla y suelta todas las teclas automáticamente").size(11.0).color(egui::Color32::from_rgb(120, 120, 140)));
        ui.add_space(12.0);

        // ── Umbral de cambio ─────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Umbral de cambio:");
            if ui.add_enabled(!self.panic_active, egui::DragValue::new(&mut self.config.panic_threshold).range(10.0..=500.0).speed(1.0)).changed() {
                self.mark_dirty();
            }
        });
        ui.label(egui::RichText::new("  Sensibilidad. Menor = Más sensible, Mayor = Menos sensible.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
        ui.add_space(6.0);

        // ── Intervalo de revisión ────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Intervalo de revisión (ms):");
            if ui.add_enabled(!self.panic_active, egui::DragValue::new(&mut self.config.panic_check_interval_ms).range(200..=10000)).changed() {
                self.mark_dirty();
            }
        });
        ui.label(egui::RichText::new("  Cada cuánto tiempo se escanea la pantalla en busca de cambios.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
        ui.add_space(6.0);

        if self.panic_active {
            ui.label(egui::RichText::new("⚠ Detén la vigilancia para modificar la configuración.")
                .size(11.0).color(egui::Color32::from_rgb(220, 180, 60)));
            ui.add_space(4.0);
        }

        // ── Hotkey ───────────────────────────────────────────────────────────
        ui.add_enabled_ui(!self.panic_active, |ui| {
        ui.horizontal(|ui| {
            ui.label("Hotkey:");
            
            let is_valid = crate::hotkey_engine::HotkeyEngine::is_valid_key(&self.config.panic_hotkey);
            let hotkey_text = if self.config.panic_hotkey.is_empty() {
                "Ninguna".to_string()
            } else {
                self.config.panic_hotkey.clone()
            };
            let is_assigning = self.assigning_hotkey_for.as_deref() == Some("panic");
            let btn_text = if is_assigning { "Presiona tecla..." } else { &hotkey_text };
            let btn_color = if is_assigning { egui::Color32::from_rgb(255, 200, 100) } else { egui::Color32::WHITE };

            let mut frame = egui::Frame::new();
            if !is_valid {
                frame = frame.fill(egui::Color32::from_rgba_premultiplied(200, 60, 60, 40));
            }
            
            frame.show(ui, |ui| {
                let mut btn = ui.add_sized([120.0, 20.0], egui::Button::new(
                    egui::RichText::new(btn_text).color(btn_color)
                ));
                
                if !is_valid {
                    btn = btn.on_hover_text("Hotkey inválida — usa formato: F6, Ctrl+F6, MouseLeft");
                }
                
                if btn.clicked() {
                    self.start_assigning_hotkey("panic");
                }
            });
            
            // Clear button
            let clear_btn = ui.add_enabled(!self.config.panic_hotkey.is_empty(), egui::Button::new("✕"));
            if clear_btn.on_hover_text("Quitar hotkey").clicked() {
                self.config.panic_hotkey.clear();
                self.apply_hotkeys();
                self.mark_dirty();
            }
        });
        ui.label(egui::RichText::new("  Haz clic para asignar una tecla. Soporta botones de ratón y teclado.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
        }); // end add_enabled_ui

        ui.add_space(8.0);
        let status = self.panic_status.lock().unwrap().clone();
        let has_ref = *self.panic_has_reference.lock().unwrap();

        ui.label(egui::RichText::new(format!("Estado: {}", status)).size(12.0).color(egui::Color32::from_rgb(200, 180, 180)));

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        ui.label(egui::RichText::new("Paso 1: Captura de Referencia").size(14.0).strong().color(egui::Color32::from_rgb(200, 200, 220)));
        ui.label(
            egui::RichText::new("Navega a la pantalla que quieres vigilar, luego presiona el botón. Tienes 3 segundos para posicionarte.")
                .size(10.0).italics().color(egui::Color32::from_rgb(120, 120, 150))
        );
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let ref_color = if has_ref {
                egui::Color32::from_rgb(50, 150, 80)
            } else {
                egui::Color32::from_rgb(180, 130, 40)
            };
            let ref_text = if has_ref { "📸 Re-capturar Referencia" } else { "📸 Capturar Referencia" };

            if ui.add_sized([220.0, 36.0], egui::Button::new(
                egui::RichText::new(ref_text).size(13.0).color(egui::Color32::WHITE)
            ).fill(ref_color)).clicked() {
                self.capture_panic_reference();
            }

            if has_ref {
                ui.label(egui::RichText::new("✓ Referencia lista").size(11.0).color(egui::Color32::from_rgb(80, 200, 100)));
            }
        });

        ui.add_space(12.0);
        ui.label(egui::RichText::new("Paso 2: Activar Vigilancia").size(14.0).strong().color(egui::Color32::from_rgb(200, 200, 220)));
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            // Botón Aplicar Config (siempre visible)
            if ui.add_sized([160.0, 36.0], egui::Button::new(
                egui::RichText::new("Aplicar Config").size(13.0).color(egui::Color32::WHITE)
            ).fill(egui::Color32::from_rgb(60, 40, 140))).clicked() {
                self.config.save();
                self.apply_hotkeys();
                self.set_status("✓ Config aplicada");
            }

            let (text, color) = if self.panic_active {
                ("⏹ DESACTIVAR", egui::Color32::from_rgb(200, 50, 50))
            } else {
                ("▶ ACTIVAR VIGILANCIA", egui::Color32::from_rgb(200, 140, 40))
            };
            if ui.add_sized([220.0, 40.0], egui::Button::new(
                egui::RichText::new(text).size(14.0).strong().color(egui::Color32::WHITE)
            ).fill(color)).clicked() {
                self.config.save();
                self.apply_hotkeys();
                self.toggle_panic();
            }
        });
    }
}
