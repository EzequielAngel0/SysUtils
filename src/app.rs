use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::config::AppConfig;
use crate::hw_link::HwLink;
use crate::models::{LogBuffer, MacroEvent, Tab};
use crate::system_info::SystemMonitor;

pub struct SysUtilsApp {
    pub config: AppConfig,
    pub hw: Arc<HwLink>,
    pub logs: LogBuffer,

    pub active_tab: Tab,
    pub status_message: String,
    pub status_timestamp: Instant,

    // Hardware
    pub available_ports: Vec<String>,
    pub selected_port: String,

    // KeepAlive
    pub keepalive_active: bool,
    pub keepalive_stop: Arc<Mutex<bool>>,

    // Pulse
    pub pulse_active: bool,

    // Monitor
    pub monitor_active: bool,
    pub monitor_stop: Arc<Mutex<bool>>,
    pub monitor_status: Arc<Mutex<String>>,
    pub monitor_has_reference: Arc<Mutex<bool>>,
    pub monitor_reference_pixels: Arc<Mutex<Option<Vec<u8>>>>,
    pub monitor_ref_width: Arc<Mutex<usize>>,
    pub monitor_ref_height: Arc<Mutex<usize>>,
    pub monitor_target_id: usize,
    pub monitor_is_window: bool,
    pub monitor_targets: Vec<crate::screen_capture::TargetInfo>,
    pub monitor_preview_texture: Option<eframe::egui::TextureHandle>,
    pub monitor_pixel_color_str: String,
    pub monitor_drag_start: Option<eframe::egui::Pos2>,
    pub monitor_zoom: f32,

    // Panic
    pub panic_active: bool,
    pub panic_stop: Arc<Mutex<bool>>,
    pub panic_status: Arc<Mutex<String>>,
    pub panic_has_reference: Arc<Mutex<bool>>,
    pub panic_reference_pixels: Arc<Mutex<Option<Vec<u8>>>>,

    // Sequence
    pub sequence_recording: bool,
    pub sequence_recording_stop: Arc<Mutex<bool>>,
    pub sequence_playing: bool,
    pub sequence_play_stop: Arc<Mutex<bool>>,
    pub sequence_events: Arc<Mutex<Vec<MacroEvent>>>,
    pub sequence_loops: String,
    pub sequence_last_event_time: Arc<Mutex<Option<Instant>>>,

    pub hotkeys: Arc<crate::hotkey_engine::HotkeyEngine>,
    pub log_auto_scroll: bool,
    pub assigning_hotkey_for: Option<String>,

    // System monitoring (CPU/RAM/Anti-cheat)
    pub sys_monitor: SystemMonitor,
    pub last_sys_refresh: Instant,
    pub last_anticheat_scan: Instant,

    // Auto-save
    pub last_auto_save: Instant,
    pub config_dirty: bool,

    // File logging
    #[allow(dead_code)]
    pub file_log_guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}

impl SysUtilsApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = AppConfig::load();
        let logs = LogBuffer::new(500);

        let hotkeys = Arc::new(crate::hotkey_engine::HotkeyEngine::new());
        hotkeys.start();
        logs.log(crate::models::LogLevel::Info, "Sistema", "SysUtils iniciado correctamente");

        let ports = HwLink::available_ports();
        let selected = if !config.last_port.is_empty() {
            config.last_port.clone()
        } else {
            ports.first().cloned().unwrap_or_default()
        };

        if !ports.is_empty() {
            logs.log(crate::models::LogLevel::Info, "Serial", &format!("{} puerto(s) detectado(s)", ports.len()));
        }

        // Initialize file logging if enabled
        let file_log_guard = if config.file_logging_enabled {
            let guard = crate::file_logger::init_file_logging();
            if guard.is_some() {
                logs.log(crate::models::LogLevel::Info, "Logger", "Logging a archivo activado");
            }
            guard
        } else {
            None
        };

        let sys_monitor = SystemMonitor::new();
        // Initial scans
        sys_monitor.refresh();
        sys_monitor.scan_anticheat();

        let now = Instant::now();

        let app_instance = Self {
            hw: Arc::new(HwLink::new()),
            logs,
            active_tab: Tab::Pulse,
            status_message: "Iniciando...".into(),
            status_timestamp: now,

            available_ports: ports,
            selected_port: selected,

            keepalive_active: false,
            keepalive_stop: Arc::new(Mutex::new(false)),

            pulse_active: false,

            monitor_active: false,
            monitor_stop: Arc::new(Mutex::new(false)),
            monitor_status: Arc::new(Mutex::new("Inactivo".into())),
            monitor_has_reference: Arc::new(Mutex::new(false)),
            monitor_reference_pixels: Arc::new(Mutex::new(None)),
            monitor_ref_width: Arc::new(Mutex::new(0)),
            monitor_ref_height: Arc::new(Mutex::new(0)),
            monitor_target_id: config.monitor_target_id,
            monitor_is_window: config.monitor_is_window,
            monitor_targets: Vec::new(),
            monitor_preview_texture: None,
            monitor_drag_start: None,
            monitor_pixel_color_str: String::new(),
            monitor_zoom: 1.0,

            panic_active: false,
            panic_stop: Arc::new(Mutex::new(false)),
            panic_status: Arc::new(Mutex::new("Inactivo".into())),
            panic_has_reference: Arc::new(Mutex::new(false)),
            panic_reference_pixels: Arc::new(Mutex::new(None)),

            sequence_recording: false,
            sequence_recording_stop: Arc::new(Mutex::new(false)),
            sequence_playing: false,
            sequence_play_stop: Arc::new(Mutex::new(false)),
            sequence_events: Arc::new(Mutex::new(Vec::new())),
            sequence_loops: "1".into(),
            sequence_last_event_time: Arc::new(Mutex::new(None)),

            config,
            hotkeys,
            log_auto_scroll: true,
            assigning_hotkey_for: None,

            sys_monitor,
            last_sys_refresh: now,
            last_anticheat_scan: now,

            last_auto_save: now,
            config_dirty: false,

            file_log_guard,
        };

        app_instance.apply_hotkeys();

        // Log anti-cheat warnings
        let warnings = app_instance.sys_monitor.anticheat_warnings();
        for w in &warnings {
            app_instance.logs.log(crate::models::LogLevel::Warning, "AntiCheat", &format!("⚠ Detectado: {}", w));
        }

        app_instance
    }

    pub fn apply_hotkeys(&self) {
        self.hotkeys.clear_bindings();
        if !self.config.pulse_hotkey.is_empty() {
            self.hotkeys.register("pulse_toggle", &self.config.pulse_hotkey);
        }
        if !self.config.keepalive_hotkey.is_empty() {
            self.hotkeys.register("keepalive_toggle", &self.config.keepalive_hotkey);
        }
        if !self.config.monitor_hotkey.is_empty() {
            self.hotkeys.register("monitor_toggle", &self.config.monitor_hotkey);
        }
        if !self.config.panic_hotkey.is_empty() {
            self.hotkeys.register("panic_toggle", &self.config.panic_hotkey);
        }
        if !self.config.sequence_hotkey_record.is_empty() {
            self.hotkeys.register("seq_record", &self.config.sequence_hotkey_record);
        }
        if !self.config.sequence_hotkey_play.is_empty() {
            self.hotkeys.register("seq_play", &self.config.sequence_hotkey_play);
        }
    }

    pub fn mark_dirty(&mut self) {
        self.config_dirty = true;
    }

    /// Called from update() — auto-saves every 5 seconds if config is dirty
    pub fn auto_save_tick(&mut self) {
        if self.config_dirty && self.last_auto_save.elapsed().as_secs() >= 5 {
            self.config.save();
            self.config_dirty = false;
            self.last_auto_save = Instant::now();
        }
    }

    /// Called from update() — refresh system info every 2 seconds
    pub fn sys_info_tick(&mut self) {
        if self.last_sys_refresh.elapsed().as_secs() >= 2 {
            self.sys_monitor.refresh();
            self.last_sys_refresh = Instant::now();
        }
        // Anti-cheat scan every 30 seconds
        if self.last_anticheat_scan.elapsed().as_secs() >= 30 {
            self.sys_monitor.scan_anticheat();
            self.last_anticheat_scan = Instant::now();
        }
    }
}
