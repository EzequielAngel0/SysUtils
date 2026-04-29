// ═══════════════════════════════════════════════════════════════════════════════
// notifications.rs — Sistema de notificaciones del sistema operativo
// ═══════════════════════════════════════════════════════════════════════════════

use crate::config::AppConfig;

// ─── Eventos ─────────────────────────────────────────────────────────────────

pub enum NotificationEvent {
    PanicTriggered { diff: f64 },
    Esp32Disconnected { port: String },
    ModuleToggled { module: &'static str, active: bool },
}

// ─── Servicio ─────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct NotificationService {
    enabled: bool,
    notify_on_panic: bool,
    notify_on_disconnect: bool,
    notify_on_module_toggle: bool,
}

impl NotificationService {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            enabled: config.notifications_enabled,
            notify_on_panic: config.notify_on_panic,
            notify_on_disconnect: config.notify_on_disconnect,
            notify_on_module_toggle: config.notify_on_module_toggle,
        }
    }

    pub fn update_config(&mut self, config: &AppConfig) {
        self.enabled = config.notifications_enabled;
        self.notify_on_panic = config.notify_on_panic;
        self.notify_on_disconnect = config.notify_on_disconnect;
        self.notify_on_module_toggle = config.notify_on_module_toggle;
    }

    pub fn notify(&self, event: NotificationEvent) {
        if !self.enabled {
            return;
        }

        let (summary, body, should_send) = match &event {
            NotificationEvent::PanicTriggered { diff } => (
                "⚠ SysUtils — Panic Switch".to_string(),
                format!("Diferencia detectada: {:.2}", diff),
                self.notify_on_panic,
            ),
            NotificationEvent::Esp32Disconnected { port } => (
                "🔌 SysUtils — Conexión perdida".to_string(),
                format!("Puerto desconectado: {}", port),
                self.notify_on_disconnect,
            ),
            NotificationEvent::ModuleToggled { module, active } => (
                "⚙ SysUtils".to_string(),
                format!(
                    "Módulo {} {}",
                    module,
                    if *active { "activado" } else { "desactivado" }
                ),
                self.notify_on_module_toggle,
            ),
        };

        if !should_send {
            return;
        }

        let result = notify_rust::Notification::new()
            .summary(&summary)
            .body(&body)
            .show();

        if let Err(e) = result {
            eprintln!("[notifications] Error al mostrar notificación: {}", e);
        }
    }
}
