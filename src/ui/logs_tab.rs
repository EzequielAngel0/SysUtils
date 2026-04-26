use crate::{app::SysUtilsApp, models::{LogLevel, LogEntry}};
use eframe::egui;

#[allow(dead_code)]
impl SysUtilsApp {
    pub(crate) fn render_logs_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("📋 Application Logs").size(18.0).color(egui::Color32::from_rgb(180, 180, 200)));
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Filtro:");
            if ui.selectable_label(self.log_filter == LogLevel::Info, "Todos").clicked() {
                self.log_filter = LogLevel::Info;
            }
            if ui.selectable_label(self.log_filter == LogLevel::Action, "Acciones").clicked() {
                self.log_filter = LogLevel::Action;
            }
            if ui.selectable_label(self.log_filter == LogLevel::Warning, "Warnings").clicked() {
                self.log_filter = LogLevel::Warning;
            }
            if ui.selectable_label(self.log_filter == LogLevel::Error, "Errores").clicked() {
                self.log_filter = LogLevel::Error;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑 Limpiar").clicked() {
                    self.logs.clear();
                }
                ui.checkbox(&mut self.log_auto_scroll, "Auto-scroll");
            });
        });

        ui.separator();
        ui.add_space(2.0);

        let entries = self.logs.get_all();
        let min_level = match self.log_filter {
            LogLevel::Info    => 0,
            LogLevel::Action  => 1,
            LogLevel::Warning => 2,
            LogLevel::Error   => 3,
        };

        let filtered: Vec<&LogEntry> = entries.iter().filter(|e| {
            let level = match e.level {
                LogLevel::Info    => 0,
                LogLevel::Action  => 1,
                LogLevel::Warning => 2,
                LogLevel::Error   => 3,
            };
            level >= min_level
        }).collect();

        let scroll = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(self.log_auto_scroll);

        scroll.show(ui, |ui| {
            if filtered.is_empty() {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("Sin logs").size(13.0).color(egui::Color32::from_rgb(80, 80, 100)));
                });
            } else {
                for entry in &filtered {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&entry.timestamp)
                                .size(10.0).monospace().color(egui::Color32::from_rgb(80, 80, 100))
                        );
                        ui.label(
                            egui::RichText::new(entry.level.icon())
                                .size(11.0).color(entry.level.color())
                        );
                        ui.label(
                            egui::RichText::new(format!("[{}]", entry.module))
                                .size(10.0).strong().color(egui::Color32::from_rgb(140, 120, 180))
                        );
                        ui.label(
                            egui::RichText::new(&entry.message)
                                .size(11.0).color(entry.level.color())
                        );
                    });
                }
            }
        });
    }
}
