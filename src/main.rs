// ═══════════════════════════════════════════════════════════════════════════════
// main.rs — SysUtils Native: unified 100% Rust application
// ═══════════════════════════════════════════════════════════════════════════════

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod file_logger;
mod hotkey_engine;
mod hw_link;
mod logic;
mod models;
mod notifications;
mod screen_capture;
mod stealth;
mod system_info;
mod ui;

use app::SysUtilsApp;
use eframe::egui;
use std::time::Duration;

use logic::hardware::HardwareLogic;
use logic::keepalive::KeepAliveLogic;
use logic::monitor::MonitorLogic;
use logic::panic::PanicLogic;
use logic::pulse::PulseLogic;
use logic::sequence::SequenceLogic;
use models::Tab;

impl SysUtilsApp {
    fn process_hotkeys(&mut self) {
        while let Ok(event) = self.hotkeys.event_rx.try_recv() {
            match event {
                hotkey_engine::HotkeyEvent::Triggered(id) => {
                    if id == "pulse_toggle" {
                        self.toggle_pulse();
                        let active = self.pulse_active;
                        self.notification_service.notify(crate::notifications::NotificationEvent::ModuleToggled { module: "Scheduler", active });
                    } else if id == "keepalive_toggle" {
                        if self.keepalive_active { self.stop_keepalive(); }
                        else { self.config.save(); self.start_keepalive(); }
                        let active = self.keepalive_active;
                        self.notification_service.notify(crate::notifications::NotificationEvent::ModuleToggled { module: "Background", active });
                    } else if id == "monitor_toggle" {
                        if self.monitor_active { self.stop_monitor(); }
                        else { self.config.save(); self.start_monitor(); }
                        let active = self.monitor_active;
                        self.notification_service.notify(crate::notifications::NotificationEvent::ModuleToggled { module: "Diagnostics", active });
                    } else if id == "panic_toggle" {
                        self.toggle_panic();
                        let active = self.panic_active;
                        self.notification_service.notify(crate::notifications::NotificationEvent::ModuleToggled { module: "Security", active });
                    } else if id == "seq_record" {
                        if self.sequence_recording {
                            self.stop_recording();
                        } else {
                            self.start_recording();
                        }
                    } else if id == "seq_play" {
                        if self.sequence_playing {
                            self.stop_playback();
                        } else {
                            self.play_sequence();
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Egui App
// ═══════════════════════════════════════════════════════════════════════════════
impl eframe::App for SysUtilsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(target) = self.assigning_hotkey_for.clone() {
            let mut pressed_key = None;
            // Wait at least 200ms after entering assign mode before accepting input.
            // This prevents the mouse click that opened the dialog from being captured.
            let ready = self.hotkey_assign_start
                .map(|t| t.elapsed().as_millis() >= 200)
                .unwrap_or(false);

            if ready {
                while let Ok(event) = self.hotkeys.raw_rx.try_recv() {
                    if let rdev::EventType::KeyPress(key) = event.event_type {
                        pressed_key = Some(format!("{:?}", key));
                        break;
                    } else if let rdev::EventType::ButtonPress(btn) = event.event_type {
                        pressed_key = Some(format!("{:?}", btn));
                        break;
                    }
                }
            } else {
                // Drain the channel while in the cooldown window so stale clicks don't queue up
                while self.hotkeys.raw_rx.try_recv().is_ok() {}
            }

            if let Some(pk) = pressed_key {
                match target.as_str() {
                    "pulse" => self.config.pulse_hotkey = pk,
                    "keepalive" => self.config.keepalive_hotkey = pk,
                    "monitor" => self.config.monitor_hotkey = pk,
                    "panic" => self.config.panic_hotkey = pk,
                    "sequence_record" => self.config.sequence_hotkey_record = pk,
                    "sequence_play" => self.config.sequence_hotkey_play = pk,
                    _ => {}
                }
                self.mark_dirty();
                self.apply_hotkeys();
                self.assigning_hotkey_for = None;
                self.hotkey_assign_start = None;
            }

            egui::Window::new("Asignando Hotkey")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Presiona cualquier tecla o botón del ratón...").size(16.0).color(egui::Color32::from_rgb(255, 200, 100)));
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Cancelar").clicked() {
                                self.assigning_hotkey_for = None;
                                self.hotkey_assign_start = None;
                            }
                        });
                    });
                });
        }

        // F3: Save profile dialog
        if self.show_save_profile_dialog {
            egui::Window::new("Guardar Perfil")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.label("Nombre del perfil:");
                    ui.add_space(4.0);
                    ui.text_edit_singleline(&mut self.profile_name_input);
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        if ui.button("Guardar").clicked() && !self.profile_name_input.trim().is_empty() {
                            let profile_name = self.profile_name_input.trim().to_string();
                            match self.config.save_as_profile(&profile_name) {
                                Ok(()) => {
                                    self.available_profiles = config::AppConfig::scan_profiles();
                                    self.config.active_profile = profile_name.clone();
                                    self.config.save();
                                    self.logs.log(models::LogLevel::Action, "Config", &format!("Perfil '{}' guardado", profile_name));
                                    self.show_save_profile_dialog = false;
                                    self.profile_name_input.clear();
                                }
                                Err(e) => {
                                    self.logs.log(models::LogLevel::Error, "Config", &format!("Error guardando perfil: {}", e));
                                }
                            }
                        }
                        if ui.button("Cancelar").clicked() {
                            self.show_save_profile_dialog = false;
                            self.profile_name_input.clear();
                        }
                    });
                });
        }

        self.process_hotkeys();
        self.auto_save_tick();
        self.sys_info_tick();

        // Stealth mode — aplicar solo cuando el estado cambia
        if self.config.stealth_enabled != self.stealth_mode_applied {
            let stealth_cfg = crate::stealth::StealthConfig {
                enabled: self.config.stealth_enabled,
                fake_window_title: self.config.stealth_window_title.clone(),
                fake_process_name: self.config.stealth_process_name.clone(),
            };
            crate::stealth::StealthMode::apply(&stealth_cfg, ctx);
            self.stealth_mode_applied = self.config.stealth_enabled;
        }

        ctx.request_repaint_after(Duration::from_millis(33));

        // ── DARK THEME ──────────────────────────────────────────────────────────
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(18, 18, 24);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(30, 30, 42);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 45, 65);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(80, 60, 180);
        style.visuals.panel_fill = egui::Color32::from_rgb(12, 12, 18);
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(8, 8, 12);
        ctx.set_style(style);

        // ── LEFT SIDEBAR (Navigation) ───────────────────────────────────────────
        egui::SidePanel::left("sidebar_left")
            .resizable(false)
            .exact_width(170.0)
            .show(ctx, |ui| {
                ui.add_space(12.0);
                ui.vertical_centered(|ui| {
                    ui.heading(
                        egui::RichText::new("⚙ SysUtils")
                            .size(20.0)
                            .color(egui::Color32::from_rgb(200, 200, 200))
                            .strong(),
                    );
                });
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(12.0);

                let tabs = [
                    (Tab::Pulse,     "⚡  Scheduler"),
                    (Tab::KeepAlive, "♻  Background"),
                    (Tab::Monitor,   "👁  Diagnostics"),
                    (Tab::Panic,     "🛡  Security"),
                    (Tab::Sequence,  "🎬  Workflows"),
                ];

                for (tab, label) in &tabs {
                    let is_selected = self.active_tab == *tab;
                    let btn = ui.add_sized(
                        [150.0, 34.0],
                        egui::Button::new(
                            egui::RichText::new(*label)
                                .size(13.0)
                                .color(if is_selected {
                                    egui::Color32::from_rgb(220, 220, 220)
                                } else {
                                    egui::Color32::from_rgb(140, 140, 160)
                                }),
                        )
                        .fill(if is_selected {
                            egui::Color32::from_rgb(50, 70, 100)
                        } else {
                            egui::Color32::TRANSPARENT
                        })
                        .corner_radius(egui::CornerRadius::same(6)),
                    );
                    if btn.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        self.active_tab = tab.clone();
                    }
                }

                // Connection at bottom
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    let conn_color = if self.hw.is_connected() {
                        egui::Color32::from_rgb(80, 220, 120)
                    } else {
                        egui::Color32::from_rgb(220, 80, 80)
                    };
                    ui.label(
                        egui::RichText::new(if self.hw.is_connected() { "● Conectado" } else { "○ Desconectado" })
                            .size(11.0)
                            .color(conn_color),
                    );
                    ui.add_space(4.0);
                });
            });

        // ── RIGHT SIDEBAR (Module Toggles + System Info) ────────────────────────
        let mut toggle_pulse = false;
        let mut toggle_keepalive = false;
        let mut toggle_monitor = false;
        let mut toggle_panic = false;

        let pulse_on = self.pulse_active;
        let ka_on = self.keepalive_active;
        let mon_on = self.monitor_active;
        let panic_on = self.panic_active;
        let seq_on = self.sequence_recording || self.sequence_playing;
        let active_count = [pulse_on, ka_on, mon_on, panic_on].iter().filter(|&&x| x).count();
        let log_count = self.logs.get_all().len();
        let cpu = self.sys_monitor.cpu_usage();
        let ram = self.sys_monitor.ram_usage_mb();
        let ac_warnings = self.sys_monitor.anticheat_warnings();

        egui::SidePanel::right("sidebar_right")
            .resizable(false)
            .exact_width(185.0)
            .show(ctx, |ui| {
                ui.add_space(12.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Módulos")
                            .size(16.0)
                            .color(egui::Color32::from_rgb(160, 140, 200))
                            .strong(),
                    );
                });
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(8.0);

                // Clickable module toggle cards
                if render_toggle_card(ui, "⚡ Scheduler", pulse_on, egui::Color32::from_rgb(100, 150, 255)) {
                    toggle_pulse = true;
                }
                ui.add_space(6.0);
                if render_toggle_card(ui, "♻ Background", ka_on, egui::Color32::from_rgb(80, 200, 160)) {
                    toggle_keepalive = true;
                }
                ui.add_space(6.0);
                if render_toggle_card(ui, "👁 Diagnostics", mon_on, egui::Color32::from_rgb(100, 180, 255)) {
                    toggle_monitor = true;
                }
                ui.add_space(6.0);
                if render_toggle_card(ui, "🛡 Security", panic_on, egui::Color32::from_rgb(255, 100, 100)) {
                    toggle_panic = true;
                }
                ui.add_space(6.0);

                // Sequence is info-only
                render_info_card(ui, "🎬 Workflows", seq_on, egui::Color32::from_rgb(255, 180, 80));

                // Divider
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                ui.label(egui::RichText::new("Estado General").size(12.0).color(egui::Color32::from_rgb(120, 120, 140)));
                ui.add_space(4.0);

                ui.label(
                    egui::RichText::new(format!("{} módulo(s) activo(s)", active_count))
                        .size(13.0)
                        .color(if active_count > 0 {
                            egui::Color32::from_rgb(100, 220, 140)
                        } else {
                            egui::Color32::from_rgb(120, 120, 140)
                        }),
                );

                ui.label(
                    egui::RichText::new(format!("{} log(s)", log_count))
                        .size(11.0)
                        .color(egui::Color32::from_rgb(100, 100, 120)),
                );

                // System Info
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Sistema").size(12.0).color(egui::Color32::from_rgb(120, 120, 140)));
                ui.add_space(4.0);
                
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("CPU:").size(11.0).color(egui::Color32::from_rgb(140, 140, 160)));
                    let cpu_color = if cpu > 50.0 {
                        egui::Color32::from_rgb(255, 160, 80)
                    } else if cpu > 25.0 {
                        egui::Color32::from_rgb(220, 200, 100)
                    } else {
                        egui::Color32::from_rgb(100, 200, 140)
                    };
                    ui.label(egui::RichText::new(format!("{:.1}%", cpu)).size(11.0).color(cpu_color));
                });
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("RAM:").size(11.0).color(egui::Color32::from_rgb(140, 140, 160)));
                    ui.label(egui::RichText::new(format!("{:.1} MB", ram)).size(11.0).color(egui::Color32::from_rgb(100, 180, 220)));
                });

                // Anti-cheat warnings
                if !ac_warnings.is_empty() {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("⚠ Anti-Cheat").size(12.0).strong().color(egui::Color32::from_rgb(255, 120, 80)));
                    for w in &ac_warnings {
                        ui.label(egui::RichText::new(format!("• {}", w)).size(10.0).color(egui::Color32::from_rgb(255, 160, 110)));
                    }
                }

                // Notificaciones
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                ui.label(egui::RichText::new("🔔 Notificaciones").size(12.0).color(egui::Color32::from_rgb(120, 120, 140)));
                ui.add_space(4.0);

                let mut notif_enabled = self.config.notifications_enabled;
                if ui.checkbox(&mut notif_enabled, egui::RichText::new("Activar notificaciones").size(10.0)).changed() {
                    self.config.notifications_enabled = notif_enabled;
                    self.apply_hotkeys();
                    self.mark_dirty();
                }

                if self.config.notifications_enabled {
                    let mut on_panic = self.config.notify_on_panic;
                    if ui.checkbox(&mut on_panic, egui::RichText::new("  Panic Switch").size(10.0)).changed() {
                        self.config.notify_on_panic = on_panic;
                        self.apply_hotkeys();
                        self.mark_dirty();
                    }
                    let mut on_disc = self.config.notify_on_disconnect;
                    if ui.checkbox(&mut on_disc, egui::RichText::new("  Desconexión ESP32").size(10.0)).changed() {
                        self.config.notify_on_disconnect = on_disc;
                        self.apply_hotkeys();
                        self.mark_dirty();
                    }
                    let mut on_toggle = self.config.notify_on_module_toggle;
                    if ui.checkbox(&mut on_toggle, egui::RichText::new("  Toggle de módulo").size(10.0)).changed() {
                        self.config.notify_on_module_toggle = on_toggle;
                        self.apply_hotkeys();
                        self.mark_dirty();
                    }
                }

                // Stealth
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                ui.label(egui::RichText::new("🕵 Stealth").size(12.0).color(egui::Color32::from_rgb(120, 120, 140)));
                ui.add_space(4.0);

                let mut stealth_en = self.config.stealth_enabled;
                if ui.checkbox(&mut stealth_en, egui::RichText::new("Modo stealth").size(10.0)).changed() {
                    self.config.stealth_enabled = stealth_en;
                    self.mark_dirty();
                }

                if self.config.stealth_enabled {
                    ui.label(egui::RichText::new("Título ventana:").size(10.0).color(egui::Color32::from_rgb(140, 140, 160)));
                    let title_empty = self.config.stealth_window_title.is_empty();
                    let title_resp = ui.add(
                        egui::TextEdit::singleline(&mut self.config.stealth_window_title)
                            .font(egui::TextStyle::Small)
                            .desired_width(f32::INFINITY)
                    );
                    if title_resp.changed() { self.mark_dirty(); }
                    // 8.3 Validación inline
                    if title_empty {
                        ui.label(egui::RichText::new("⚠ El título no puede estar vacío").size(9.0).color(egui::Color32::from_rgb(255, 120, 80)));
                    }

                    ui.label(egui::RichText::new("Nombre proceso (Linux):").size(10.0).color(egui::Color32::from_rgb(140, 140, 160)));
                    let proc_resp = ui.add(
                        egui::TextEdit::singleline(&mut self.config.stealth_process_name)
                            .font(egui::TextStyle::Small)
                            .desired_width(f32::INFINITY)
                    );
                    if proc_resp.changed() {
                        // Truncar a 15 chars
                        if self.config.stealth_process_name.len() > 15 {
                            self.config.stealth_process_name.truncate(15);
                        }
                        self.mark_dirty();
                    }
                }

                // Export / Import at bottom
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("📤 Export").size(10.0)).on_hover_text("Exportar config a archivo").clicked() {
                            let mut path = std::env::current_exe().unwrap_or_default();
                            path.pop();
                            path.push("sysutils_config_export.json");
                            match self.config.export_to(&path) {
                                Ok(()) => self.logs.log(models::LogLevel::Action, "Config", &format!("Exportado a {}", path.display())),
                                Err(e) => self.logs.log(models::LogLevel::Error, "Config", &format!("Error exportando: {}", e)),
                            }
                        }
                        if ui.button(egui::RichText::new("📥 Import").size(10.0)).on_hover_text("Importar config desde archivo").clicked() {
                            let mut path = std::env::current_exe().unwrap_or_default();
                            path.pop();
                            path.push("sysutils_config_export.json");
                            match config::AppConfig::import_from(&path) {
                                Ok(imported) => {
                                    self.config = imported;
                                    self.config.save();
                                    self.apply_hotkeys();
                                    self.logs.log(models::LogLevel::Action, "Config", "Configuración importada correctamente");
                                }
                                Err(e) => self.logs.log(models::LogLevel::Error, "Config", &format!("Error importando: {}", e)),
                            }
                        }
                    });
                    ui.add_space(4.0);

                    // File logging toggle
                    let mut fl = self.config.file_logging_enabled;
                    if ui.checkbox(&mut fl, egui::RichText::new("Log a archivo").size(10.0)).changed() {
                        self.config.file_logging_enabled = fl;
                        self.mark_dirty();
                    }
                });
            });

        // Apply toggle actions
        if toggle_pulse { self.toggle_pulse(); }
        if toggle_keepalive {
            if self.keepalive_active { self.stop_keepalive(); }
            else { self.config.save(); self.apply_hotkeys(); self.start_keepalive(); }
        }
        if toggle_monitor {
            if self.monitor_active { self.stop_monitor(); }
            else { self.config.save(); self.apply_hotkeys(); self.start_monitor(); }
        }
        if toggle_panic { self.config.save(); self.apply_hotkeys(); self.toggle_panic(); }

        // ── GLOBAL LOGS PANEL ───────────────────────────────────────────────────
        egui::TopBottomPanel::bottom("global_logs_panel")
            .resizable(true)
            .min_height(100.0)
            .max_height(300.0)
            .default_height(150.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("📋 Historial de Eventos")
                        .strong()
                        .color(egui::Color32::from_rgb(180, 180, 200)));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Borrar").clicked() {
                            self.logs.clear();
                        }
                        
                        let mut auto = self.log_auto_scroll;
                        if ui.checkbox(&mut auto, "Auto-scroll").changed() {
                            self.log_auto_scroll = auto;
                            self.mark_dirty();
                        }
                    });
                });
                ui.separator();
                
                egui::ScrollArea::vertical()
                    .stick_to_bottom(self.log_auto_scroll)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        let entries = self.logs.get_all();
                        
                        if entries.is_empty() {
                            ui.label(egui::RichText::new("No hay eventos registrados").color(egui::Color32::from_rgb(100, 100, 120)));
                        }
                        
                        for entry in entries {
                            let color = match entry.level {
                                crate::models::LogLevel::Info => egui::Color32::from_rgb(180, 180, 200),
                                crate::models::LogLevel::Warning => egui::Color32::from_rgb(220, 180, 80),
                                crate::models::LogLevel::Error => egui::Color32::from_rgb(255, 100, 100),
                                crate::models::LogLevel::Action => egui::Color32::from_rgb(120, 220, 120),
                            };
                            
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&entry.timestamp).size(11.0).color(egui::Color32::from_rgb(100, 100, 120)));
                                ui.label(egui::RichText::new(format!("[{}]", entry.module)).size(11.0).strong().color(egui::Color32::from_rgb(140, 140, 180)));
                                ui.label(egui::RichText::new(&entry.message).size(12.0).color(color));
                            });

                            // Write to file log if enabled
                            if self.config.file_logging_enabled {
                                let level_str = match entry.level {
                                    crate::models::LogLevel::Error => "error",
                                    crate::models::LogLevel::Warning => "warn",
                                    crate::models::LogLevel::Action => "info",
                                    _ => "info",
                                };
                                crate::file_logger::file_log(level_str, &entry.module, &entry.message);
                            }
                        }
                    });
            });

        // ── MAIN CONTENT ────────────────────────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            // Connection bar
            ui.horizontal(|ui| {
                if ui.button("🔄").on_hover_text("Refrescar puertos").clicked() {
                    self.refresh_ports();
                }

                let ports = self.available_ports.clone();
                egui::ComboBox::from_id_salt("port_select")
                    .selected_text(if self.selected_port.is_empty() { "Seleccionar puerto..." } else { &self.selected_port })
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for port in &ports {
                            ui.selectable_value(&mut self.selected_port, port.clone(), port);
                        }
                    });

                if self.hw.is_connected() {
                    if ui.button(egui::RichText::new("Desconectar").color(egui::Color32::from_rgb(220, 80, 80))).clicked() {
                        self.disconnect();
                    }
                } else if ui.button(egui::RichText::new("Conectar").color(egui::Color32::from_rgb(80, 220, 120))).clicked() {
                    self.connect();
                }

                // F3: Profile selection
                ui.separator();
                ui.label("Perfil:");
                
                egui::ComboBox::from_id_salt("profile_select")
                    .selected_text(if self.config.active_profile.is_empty() { 
                        "Default" 
                    } else { 
                        &self.config.active_profile 
                    })
                    .width(120.0)
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(self.config.active_profile.is_empty(), "Default").clicked() {
                            self.config.active_profile.clear();
                            self.config.save();
                        }
                        
                        for profile in &self.available_profiles.clone() {
                            if ui.selectable_label(&self.config.active_profile == profile, profile).clicked() {
                                match config::AppConfig::load_profile(profile) {
                                    Ok(mut loaded) => {
                                        loaded.active_profile = profile.clone();
                                        self.config = loaded;
                                        self.config.save();
                                        self.apply_hotkeys();
                                        self.logs.log(models::LogLevel::Action, "Config", &format!("Perfil '{}' cargado", profile));
                                    }
                                    Err(e) => {
                                        self.logs.log(models::LogLevel::Error, "Config", &format!("Error cargando perfil: {}", e));
                                    }
                                }
                            }
                        }
                    });
                
                if ui.button("💾").on_hover_text("Guardar perfil actual").clicked() {
                    self.show_save_profile_dialog = true;
                }

                // Overwrite active profile with current config
                let overwrite_enabled = !self.config.active_profile.is_empty();
                if ui.add_enabled(overwrite_enabled, egui::Button::new("↺"))
                    .on_hover_text("Sobreescribir perfil activo con la configuración actual")
                    .clicked()
                {
                    let profile_name = self.config.active_profile.clone();
                    match self.config.save_as_profile(&profile_name) {
                        Ok(()) => {
                            self.logs.log(models::LogLevel::Action, "Config", &format!("Perfil '{}' sobreescrito", profile_name));
                            self.set_status(&format!("✓ Perfil '{}' actualizado", profile_name));
                        }
                        Err(e) => {
                            self.logs.log(models::LogLevel::Error, "Config", &format!("Error sobreescribiendo perfil: {}", e));
                        }
                    }
                }

                let delete_enabled = !self.config.active_profile.is_empty();
                if ui.add_enabled(delete_enabled, egui::Button::new("🗑")).on_hover_text("Eliminar perfil actual").clicked() {
                    let profile_to_delete = self.config.active_profile.clone();
                    match config::AppConfig::delete_profile(&profile_to_delete) {
                        Ok(()) => {
                            self.available_profiles = config::AppConfig::scan_profiles();
                            self.config.active_profile.clear();
                            self.config.save();
                            self.logs.log(models::LogLevel::Action, "Config", &format!("Perfil '{}' eliminado", profile_to_delete));
                        }
                        Err(e) => {
                            self.logs.log(models::LogLevel::Error, "Config", &format!("Error eliminando: {}", e));
                        }
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let age = self.status_timestamp.elapsed().as_secs();
                    let alpha = if age < 5 { 255 } else { 120 };
                    ui.label(
                        egui::RichText::new(&self.status_message)
                            .size(11.0)
                            .color(egui::Color32::from_rgba_premultiplied(180, 180, 200, alpha as u8)),
                    );
                });
            });

            ui.separator();
            ui.add_space(6.0);

            match self.active_tab {
                Tab::Pulse     => self.render_pulse_tab(ui),
                Tab::KeepAlive => self.render_keepalive_tab(ui),
                Tab::Monitor   => self.render_monitor_tab(ui),
                Tab::Panic     => self.render_panic_tab(ui),
                Tab::Sequence  => self.render_sequence_tab(ui),
                _ => {}
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.config.save();
        self.hw.disconnect();
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Toggle/Info Cards
// ═══════════════════════════════════════════════════════════════════════════════
/// Clickable toggle card — returns true if clicked
fn render_toggle_card(ui: &mut egui::Ui, label: &str, active: bool, color: egui::Color32) -> bool {
    let bg = if active {
        egui::Color32::from_rgba_premultiplied(color.r() / 3, color.g() / 3, color.b() / 3, 180)
    } else {
        egui::Color32::from_rgb(25, 25, 35)
    };

    let toggle_text = if active { "ON" } else { "OFF" };
    let toggle_color = if active {
        egui::Color32::from_rgb(80, 220, 120)
    } else {
        egui::Color32::from_rgb(120, 80, 80)
    };

    let resp = egui::Frame::new()
        .fill(bg)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let dot = if active { "●" } else { "○" };
                let dot_color = if active { color } else { egui::Color32::from_rgb(80, 80, 100) };
                ui.label(egui::RichText::new(dot).size(14.0).color(dot_color));
                ui.label(egui::RichText::new(label).size(12.0).color(egui::Color32::from_rgb(200, 200, 220)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(toggle_text).size(10.0).strong().color(toggle_color));
                });
            });
        });

    resp.response.interact(egui::Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

/// Info-only card (no click action)
fn render_info_card(ui: &mut egui::Ui, label: &str, active: bool, color: egui::Color32) {
    let bg = if active {
        egui::Color32::from_rgba_premultiplied(color.r() / 3, color.g() / 3, color.b() / 3, 180)
    } else {
        egui::Color32::from_rgb(25, 25, 35)
    };
    egui::Frame::new()
        .fill(bg)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let dot = if active { "●" } else { "○" };
                let dot_color = if active { color } else { egui::Color32::from_rgb(80, 80, 100) };
                ui.label(egui::RichText::new(dot).size(14.0).color(dot_color));
                ui.label(egui::RichText::new(label).size(12.0).color(egui::Color32::from_rgb(200, 200, 220)));
            });
        });
}

// ═══════════════════════════════════════════════════════════════════════════════
// Entry Point
// ═══════════════════════════════════════════════════════════════════════════════
fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1050.0, 700.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("SysUtils"),
        ..Default::default()
    };

    eframe::run_native("SysUtils", options, Box::new(|cc| Ok(Box::new(SysUtilsApp::new(cc)))))
}
