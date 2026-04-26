use crate::app::SysUtilsApp;
use crate::models::LogLevel;
use crate::logic::monitor::MonitorLogic;
use crate::screen_capture::ScreenCapture;
use eframe::egui;

impl SysUtilsApp {
    pub(crate) fn render_monitor_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("👁 Diagnostics").size(18.0).color(egui::Color32::from_rgb(100, 180, 255)));
        ui.label(egui::RichText::new("Detecta cambios en la pantalla y ejecuta acciones automáticas").size(11.0).color(egui::Color32::from_rgb(120, 120, 140)));
        ui.add_space(10.0);

        let status = self.monitor_status.lock().unwrap().clone();
        let has_ref = *self.monitor_has_reference.lock().unwrap();

        // ── Step 1: Display Selection ───────────────────────────────────────────
        ui.label(egui::RichText::new("Paso 1: Seleccionar Pantalla").size(14.0).strong().color(egui::Color32::from_rgb(200, 200, 220)));
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let targets = self.monitor_targets.clone();
            
            // Find current
            let current_target = targets.iter().find(|t| t.id == self.config.monitor_target_id && t.is_window == self.config.monitor_is_window);
            let selected_label = current_target.map(|t| t.label.clone()).unwrap_or_else(|| "Selecciona ventana/pantalla...".into());

            egui::ComboBox::from_id_salt("target_select")
                .selected_text(&selected_label)
                .width(350.0)
                .show_ui(ui, |ui| {
                    for t in &targets {
                        if ui.selectable_label(
                            self.config.monitor_target_id == t.id && self.config.monitor_is_window == t.is_window,
                            &t.label
                        ).clicked() {
                            self.config.monitor_target_id = t.id;
                            self.config.monitor_is_window = t.is_window;
                            self.config.save();
                        }
                    }
                });

            if ui.button("🔄 Refrescar").clicked() {
                let mut all_targets = ScreenCapture::list_displays();
                all_targets.extend(ScreenCapture::list_windows());
                self.monitor_targets = all_targets;
                self.logs.log(LogLevel::Info, "Monitor", &format!("{} fuente(s) detectada(s)", self.monitor_targets.len()));
            }
        });

        ui.add_space(8.0);

        // ── Step 2: Capture Reference ───────────────────────────────────────────
        ui.label(egui::RichText::new("Paso 2: Capturar Referencia").size(14.0).strong().color(egui::Color32::from_rgb(200, 200, 220)));
        ui.label(
            egui::RichText::new("Posiciona lo que quieres vigilar en pantalla, luego captura. Tienes 3 segundos.")
                .size(10.0).italics().color(egui::Color32::from_rgb(120, 120, 150))
        );
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let ref_btn_color = if has_ref {
                egui::Color32::from_rgb(50, 150, 80)
            } else {
                egui::Color32::from_rgb(80, 130, 200)
            };
            let ref_btn_text = if has_ref { "📸 Re-capturar" } else { "📸 Capturar Referencia" };

            if ui.add_sized([180.0, 34.0], egui::Button::new(
                egui::RichText::new(ref_btn_text).size(13.0).color(egui::Color32::WHITE)
            ).fill(ref_btn_color)).clicked() {
                self.monitor_preview_texture = None; // Clear old preview
                self.capture_monitor_reference();
            }

            ui.label(
                egui::RichText::new(&status)
                    .size(11.0)
                    .color(if has_ref { egui::Color32::from_rgb(80, 200, 120) } else { egui::Color32::from_rgb(160, 160, 180) })
            );
        });

        // Show thumbnail preview of the reference if we have one
        if has_ref {
            let ref_pixels = self.monitor_reference_pixels.lock().unwrap();
            let ref_w = *self.monitor_ref_width.lock().unwrap();
            let ref_h = *self.monitor_ref_height.lock().unwrap();

        if ref_w > 0 && ref_h > 0 {
            // Build or update the texture
            if self.monitor_preview_texture.is_none() {
                if let Some(ref rgba) = *ref_pixels {
                    let image = egui::ColorImage::from_rgba_unmultiplied([ref_w, ref_h], rgba);
                    let texture = ui.ctx().load_texture(
                        "monitor_preview",
                        image,
                        egui::TextureOptions::LINEAR,
                    );
                    self.monitor_preview_texture = Some(texture);
                }
            }

            if let Some(ref texture) = self.monitor_preview_texture {
                ui.add_space(6.0);
                    let max_width = 400.0_f32;
                    let scale = max_width / ref_w as f32;
                    let preview_h = (ref_h as f32 * scale).min(200.0);
                    let preview_w = if preview_h < ref_h as f32 * scale {
                        ref_w as f32 * (preview_h / ref_h as f32)
                    } else {
                        max_width
                    };

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Vista previa:").size(11.0).color(egui::Color32::from_rgb(120, 120, 140)));
                        if ui.button("➖ Zoom").clicked() { self.monitor_zoom = (self.monitor_zoom - 0.2).max(0.5); }
                        if ui.button("➕ Zoom").clicked() { self.monitor_zoom = (self.monitor_zoom + 0.2).min(5.0); }
                        if ui.button("Reset").clicked() { self.monitor_zoom = 1.0; }
                    });

                    egui::ScrollArea::both()
                        .max_height(250.0)
                        .max_width(400.0)
                        .show(ui, |ui| {
                            let scaled_w = preview_w * self.monitor_zoom;
                            let scaled_h = preview_h * self.monitor_zoom;

                            egui::Frame::new()
                                .fill(egui::Color32::from_rgb(20, 20, 30))
                                .corner_radius(egui::CornerRadius::same(4))
                                .inner_margin(egui::Margin::same(4))
                                .show(ui, |ui| {
                                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(scaled_w, scaled_h), egui::Sense::click_and_drag());
                            
                            // Draw image
                            ui.painter().image(
                                texture.id(),
                                rect,
                                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                egui::Color32::WHITE
                            );

                            if resp.hovered() || resp.dragged() {
                                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Crosshair);
                            }

                            // Region dragging logic
                            if resp.drag_started() {
                                if let Some(pos) = resp.interact_pointer_pos() {
                                    self.monitor_drag_start = Some(pos);
                                }
                            }
                            if resp.dragged() {
                                if let (Some(start), Some(curr)) = (self.monitor_drag_start, resp.interact_pointer_pos()) {
                                    // Draw transparent blue overlay
                                    let drag_rect = egui::Rect::from_min_max(start, curr);
                                    let fill_color = egui::Color32::from_rgba_premultiplied(100, 180, 255, 60);
                                    let stroke_color = egui::Color32::from_rgb(100, 180, 255);
                                    ui.painter().rect(drag_rect, 0.0, fill_color, egui::Stroke::new(1.0, stroke_color), egui::StrokeKind::Middle);
                                }
                            }
                            if resp.drag_stopped() {
                                if let (Some(start), Some(curr)) = (self.monitor_drag_start, resp.interact_pointer_pos()) {
                                    self.monitor_drag_start = None;
                                    
                                    let min_x = start.x.min(curr.x);
                                    let max_x = start.x.max(curr.x);
                                    let min_y = start.y.min(curr.y);
                                    let max_y = start.y.max(curr.y);
                                    
                                    let w = max_x - min_x;
                                    let h = max_y - min_y;

                                    if w > 5.0 && h > 5.0 {
                                        // Valid region drag
                                        let scale_x = ref_w as f32 / preview_w;
                                        let scale_y = ref_h as f32 / preview_h;
                                        
                                        let rx = ((min_x - rect.left()) * scale_x) as u32;
                                        let ry = ((min_y - rect.top())  * scale_y) as u32;
                                        let rw = (w * scale_x) as u32;
                                        let rh = (h * scale_y) as u32;

                                        self.config.monitor_region_x = rx.clamp(0, ref_w as u32 - 1);
                                        self.config.monitor_region_y = ry.clamp(0, ref_h as u32 - 1);
                                        self.config.monitor_region_w = rw.clamp(1, ref_w as u32 - rx);
                                        self.config.monitor_region_h = rh.clamp(1, ref_h as u32 - ry);
                                        self.config.monitor_mode = "REGION".into();

                                        self.logs.log(LogLevel::Info, "Monitor", &format!("Región seleccionada: ({}, {}) {}x{}", 
                                            self.config.monitor_region_x, self.config.monitor_region_y, 
                                            self.config.monitor_region_w, self.config.monitor_region_h));
                                    } else {
                                        // Treat as a single click (Pixel mod)
                                        let rel_x = curr.x - rect.left();
                                        let rel_y = curr.y - rect.top();
                                        
                                        let click_x = ((rel_x / preview_w) * ref_w as f32) as u32;
                                        let click_y = ((rel_y / preview_h) * ref_h as f32) as u32;
                                        
                                        self.config.monitor_pixel_x = click_x.clamp(0, ref_w as u32 - 1);
                                        self.config.monitor_pixel_y = click_y.clamp(0, ref_h as u32 - 1);
                                        self.config.monitor_mode = "PIXEL".into();
                                        
                                        self.logs.log(LogLevel::Info, "Monitor", &format!("Pixel seleccionado: ({}, {})", click_x, click_y));
                                    }
                                }
                            }

                            // Draw current region visually if in mode
                            if self.config.monitor_mode == "REGION" {
                                let scale_x = preview_w / ref_w as f32;
                                let scale_y = preview_h / ref_h as f32;
                                let rx = rect.left() + (self.config.monitor_region_x as f32 * scale_x);
                                let ry = rect.top()  + (self.config.monitor_region_y as f32 * scale_y);
                                let rw = self.config.monitor_region_w as f32 * scale_x;
                                let rh = self.config.monitor_region_h as f32 * scale_y;
                                
                                let region_rect = egui::Rect::from_min_size(egui::pos2(rx, ry), egui::vec2(rw, rh));
                                let fill_color = egui::Color32::from_rgba_premultiplied(100, 255, 100, 40);
                                let stroke_color = egui::Color32::from_rgb(100, 255, 100);
                                ui.painter().rect(region_rect, 0.0, fill_color, egui::Stroke::new(1.0, stroke_color), egui::StrokeKind::Middle);
                            } else if self.config.monitor_mode == "PIXEL" {
                                let scale_x = preview_w / ref_w as f32;
                                let scale_y = preview_h / ref_h as f32;
                                let cx = rect.left() + (self.config.monitor_pixel_x as f32 * scale_x);
                                let cy = rect.top()  + (self.config.monitor_pixel_y as f32 * scale_y);
                                
                                ui.painter().circle_stroke(
                                    egui::pos2(cx, cy), 
                                    4.0, 
                                    egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 100, 100))
                                );
                            }
                        });
                        });
                    ui.label(
                        egui::RichText::new(format!("Resolución: {}x{} (Haz clic=Pixel, Arrastra=Región)", ref_w, ref_h))
                            .size(10.0).color(egui::Color32::from_rgb(100, 100, 130))
                    );
                }
            }
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // ── Step 3: Monitor Mode Config ───────────────────────────────────────
        ui.label(egui::RichText::new("Paso 3: Modo de Monitoreo").size(14.0).strong().color(egui::Color32::from_rgb(200, 200, 220)));

        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.config.monitor_mode, "FULLSCREEN".into(), "Pantalla Completa");
            ui.selectable_value(&mut self.config.monitor_mode, "REGION".into(), "Región");
            ui.selectable_value(&mut self.config.monitor_mode, "PIXEL".into(), "Píxel Único");
        });

        ui.add_space(4.0);

        if self.config.monitor_mode == "REGION" {
            egui::Grid::new("region_grid").num_columns(4).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label("X:"); ui.add(egui::DragValue::new(&mut self.config.monitor_region_x).range(0..=9999));
                ui.label("Y:"); ui.add(egui::DragValue::new(&mut self.config.monitor_region_y).range(0..=9999));
                ui.end_row();
                ui.label("W:"); ui.add(egui::DragValue::new(&mut self.config.monitor_region_w).range(1..=9999));
                ui.label("H:"); ui.add(egui::DragValue::new(&mut self.config.monitor_region_h).range(1..=9999));
                ui.end_row();
            });
        }

        if self.config.monitor_mode == "PIXEL" {
            egui::Grid::new("pixel_grid").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label("Coordenada X:");
                ui.add(egui::DragValue::new(&mut self.config.monitor_pixel_x).range(0..=9999));
                ui.end_row();

                ui.label("Coordenada Y:");
                ui.add(egui::DragValue::new(&mut self.config.monitor_pixel_y).range(0..=9999));
                ui.end_row();
            });

            // Show the pixel color from the reference if available
            if has_ref {
                let ref_pixels = self.monitor_reference_pixels.lock().unwrap();
                let ref_w = *self.monitor_ref_width.lock().unwrap();
                let ref_h = *self.monitor_ref_height.lock().unwrap();
                if let Some(ref rgba) = *ref_pixels {
                    let px = self.config.monitor_pixel_x as usize;
                    let py = self.config.monitor_pixel_y as usize;
                    if px < ref_w && py < ref_h {
                        let offset = (py * ref_w + px) * 4;
                        if offset + 3 < rgba.len() {
                            let r = rgba[offset];
                            let g = rgba[offset + 1];
                            let b = rgba[offset + 2];
                            self.monitor_pixel_color_str = format!("RGB({}, {}, {})", r, g, b);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Color en referencia:").size(11.0));
                                let color_rect = egui::Color32::from_rgb(r, g, b);
                                let (resp, painter) = ui.allocate_painter(egui::vec2(20.0, 14.0), egui::Sense::hover());
                                painter.rect_filled(resp.rect, egui::CornerRadius::same(2), color_rect);
                                ui.label(egui::RichText::new(&self.monitor_pixel_color_str).size(10.0).monospace().color(egui::Color32::from_rgb(160, 160, 180)));
                            });
                        }
                    }
                }
            }
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // ── Configuration ───────────────────────────────────────────────────────
        ui.label(egui::RichText::new("Configuración").size(14.0).strong().color(egui::Color32::from_rgb(200, 200, 220)));
        ui.add_space(4.0);

        // ── Condición de monitoreo ────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Condición:");
            if ui.selectable_value(&mut self.config.monitor_condition, "change".into(), "Cambio").changed() {
                self.mark_dirty();
            }
            if ui.selectable_value(&mut self.config.monitor_condition, "color_appear".into(), "Color aparece").changed() {
                self.mark_dirty();
            }
            if ui.selectable_value(&mut self.config.monitor_condition, "color_disappear".into(), "Color desaparece").changed() {
                self.mark_dirty();
            }
        });
        ui.label(egui::RichText::new("  Cambio = cualquier diferencia. Color = reacciona a un color específico.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
        ui.add_space(6.0);

        // ── Color objetivo (solo para modos de color) ────────────────────────
        if self.config.monitor_condition == "color_appear" || self.config.monitor_condition == "color_disappear" {
            ui.horizontal(|ui| {
                ui.label("Color objetivo:");
                ui.label(egui::RichText::new("R:").size(11.0));
                let mut r = self.config.monitor_target_color_r as u32;
                if ui.add(egui::DragValue::new(&mut r).range(0..=255)).changed() {
                    self.config.monitor_target_color_r = r as u8;
                    self.mark_dirty();
                }
                ui.label(egui::RichText::new("G:").size(11.0));
                let mut g = self.config.monitor_target_color_g as u32;
                if ui.add(egui::DragValue::new(&mut g).range(0..=255)).changed() {
                    self.config.monitor_target_color_g = g as u8;
                    self.mark_dirty();
                }
                ui.label(egui::RichText::new("B:").size(11.0));
                let mut b = self.config.monitor_target_color_b as u32;
                if ui.add(egui::DragValue::new(&mut b).range(0..=255)).changed() {
                    self.config.monitor_target_color_b = b as u8;
                    self.mark_dirty();
                }
                // Color preview swatch
                let preview_color = egui::Color32::from_rgb(
                    self.config.monitor_target_color_r,
                    self.config.monitor_target_color_g,
                    self.config.monitor_target_color_b,
                );
                let (resp, painter) = ui.allocate_painter(egui::vec2(24.0, 16.0), egui::Sense::hover());
                painter.rect_filled(resp.rect, egui::CornerRadius::same(3), preview_color);
                painter.rect_stroke(resp.rect, egui::CornerRadius::same(3), egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 100)), egui::StrokeKind::Outside);
            });
            ui.label(egui::RichText::new("  El color RGB que el monitor buscará en la pantalla.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Tolerancia de color:");
                if ui.add(egui::DragValue::new(&mut self.config.monitor_color_tolerance).range(1..=255)).changed() {
                    self.mark_dirty();
                }
            });
            ui.label(egui::RichText::new("  Margen de diferencia aceptable para considerar un color como coincidente.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
            ui.add_space(6.0);
        }

        // ── Tolerancia de cambio (solo modo change) ──────────────────────────
        if self.config.monitor_condition == "change" {
            ui.horizontal(|ui| {
                ui.label("Tolerancia de cambio (0-765):");
                if ui.add(egui::DragValue::new(&mut self.config.monitor_tolerance).range(1..=765)).changed() {
                    self.mark_dirty();
                }
            });
            ui.label(egui::RichText::new("  Límite de diferencia de color antes de reaccionar.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
            ui.add_space(6.0);
        }

        // ── Paso de muestreo ─────────────────────────────────────────────────
        if self.config.monitor_mode != "PIXEL" {
            ui.horizontal(|ui| {
                ui.label("Paso de muestreo (píxeles):");
                if ui.add(egui::DragValue::new(&mut self.config.monitor_sample_step).range(1..=100)).changed() {
                    self.mark_dirty();
                }
            });
            ui.label(egui::RichText::new("  Aumentar hace que lea menos píxeles (usa menos CPU).").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
            ui.add_space(6.0);
        }

        // ── Duración de acción ───────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Duración de acción (ms):");
            if ui.add(egui::DragValue::new(&mut self.config.monitor_duration_ms).range(50..=5000)).changed() {
                self.mark_dirty();
            }
        });
        ui.label(egui::RichText::new("  Cuánto tiempo mantendrá la tecla/botón al activarse.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
        ui.add_space(6.0);

        // ── Tipo de acción ───────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Tipo de acción:");
            if ui.selectable_value(&mut self.config.monitor_action_type, "mouse_click".into(), "🖱 Clic de ratón").changed() {
                self.mark_dirty();
            }
            if ui.selectable_value(&mut self.config.monitor_action_type, "key_press".into(), "⌨ Tecla").changed() {
                self.mark_dirty();
            }
        });
        ui.label(egui::RichText::new("  Qué tipo de acción se ejecuta al detectar el evento.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
        ui.add_space(4.0);

        if self.config.monitor_action_type == "key_press" {
            // ── Tecla de acción ──────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Tecla a presionar:");
                if ui.add_sized([80.0, 20.0], egui::TextEdit::singleline(&mut self.config.monitor_action_key)
                    .hint_text("e, f, space..."))
                    .changed() {
                    self.mark_dirty();
                }
            });
            ui.label(egui::RichText::new("  La tecla del teclado que se presionará (enviada al ESP32).").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
        } else {
            // ── Botón de ratón ───────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Botón de clic:");
                egui::ComboBox::from_id_salt("monitor_click_action")
                    .selected_text(match self.config.monitor_click_action.as_str() {
                        "Right" => "Clic Derecho",
                        "Middle" => "Clic Central",
                        _ => "Clic Izquierdo",
                    })
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut self.config.monitor_click_action, "Left".into(), "Clic Izquierdo").changed() {
                            self.mark_dirty();
                        }
                        if ui.selectable_value(&mut self.config.monitor_click_action, "Right".into(), "Clic Derecho").changed() {
                            self.mark_dirty();
                        }
                        if ui.selectable_value(&mut self.config.monitor_click_action, "Middle".into(), "Clic Central").changed() {
                            self.mark_dirty();
                        }
                    });
            });
            ui.label(egui::RichText::new("  Qué botón del ratón presionará.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));
        }
        ui.add_space(6.0);

        // ── Hotkey ───────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Hotkey:");
            let hotkey_text = if self.config.monitor_hotkey.is_empty() {
                "Ninguna".to_string()
            } else {
                self.config.monitor_hotkey.clone()
            };
            let is_assigning = self.assigning_hotkey_for.as_deref() == Some("monitor");
            let btn_text = if is_assigning { "Presiona tecla..." } else { &hotkey_text };
            let btn_color = if is_assigning { egui::Color32::from_rgb(255, 200, 100) } else { egui::Color32::WHITE };
            
            if ui.add_sized([120.0, 20.0], egui::Button::new(
                egui::RichText::new(btn_text).color(btn_color)
            )).clicked() {
                self.assigning_hotkey_for = Some("monitor".to_string());
            }
        });
        ui.label(egui::RichText::new("  Haz clic para asignar una tecla. Soporta botones de ratón y teclado.").size(10.0).color(egui::Color32::from_rgb(120, 120, 140)));

        ui.add_space(12.0);

        // ── Start/Stop ──────────────────────────────────────────────────────────
        let (text, btn_color) = if self.monitor_active {
            ("⏹ DETENER MONITOREO", egui::Color32::from_rgb(200, 50, 50))
        } else {
            ("▶ INICIAR MONITOREO", egui::Color32::from_rgb(50, 130, 200))
        };
        if ui.add_sized([220.0, 40.0], egui::Button::new(
            egui::RichText::new(text).size(14.0).strong().color(egui::Color32::WHITE)
        ).fill(btn_color)).clicked() {
            if self.monitor_active { self.stop_monitor(); }
            else { self.config.save(); self.apply_hotkeys(); self.start_monitor(); }
        }
    }
}
