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
    
    // F2: Preview thread state
    pub monitor_preview_pixels: Arc<Mutex<Option<Vec<u8>>>>,
    pub monitor_preview_width: Arc<Mutex<usize>>,
    pub monitor_preview_height: Arc<Mutex<usize>>,
    pub monitor_preview_stop: Arc<Mutex<bool>>,
    pub monitor_preview_generation: Arc<Mutex<u64>>,
    pub monitor_preview_last_gen: u64,

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
    pub sequence_loop_counter: Arc<Mutex<u32>>,

    pub hotkeys: Arc<crate::hotkey_engine::HotkeyEngine>,
    pub log_auto_scroll: bool,
    pub assigning_hotkey_for: Option<String>,
    pub hotkey_assign_start: Option<Instant>,

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

    // F3: Profiles
    pub available_profiles: Vec<String>,
    pub profile_name_input: String,
    pub show_save_profile_dialog: bool,

    // Notifications
    pub notification_service: crate::notifications::NotificationService,

    // Stealth
    pub stealth_mode_applied: bool,
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

        // F3: Scan available profiles
        let available_profiles = AppConfig::scan_profiles();

        // Notifications — must be created before config is moved into Self
        let notification_service = crate::notifications::NotificationService::new(&config);

        let mut app_instance = Self {
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
            
            // F2: Initialize preview thread state
            monitor_preview_pixels: Arc::new(Mutex::new(None)),
            monitor_preview_width: Arc::new(Mutex::new(0)),
            monitor_preview_height: Arc::new(Mutex::new(0)),
            monitor_preview_stop: Arc::new(Mutex::new(false)),
            monitor_preview_generation: Arc::new(Mutex::new(0)),
            monitor_preview_last_gen: 0,

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
            sequence_loop_counter: Arc::new(Mutex::new(0)),

            config,
            hotkeys,
            log_auto_scroll: true,
            assigning_hotkey_for: None,
            hotkey_assign_start: None,

            sys_monitor,
            last_sys_refresh: now,
            last_anticheat_scan: now,

            last_auto_save: now,
            config_dirty: false,

            file_log_guard,

            // F3: Initialize profile fields
            available_profiles,
            profile_name_input: String::new(),
            show_save_profile_dialog: false,

            notification_service,
            stealth_mode_applied: false,
        };

        app_instance.apply_hotkeys();

        // Log anti-cheat warnings
        let warnings = app_instance.sys_monitor.anticheat_warnings();
        for w in &warnings {
            app_instance.logs.log(crate::models::LogLevel::Warning, "AntiCheat", &format!("⚠ Detectado: {}", w));
        }

        app_instance
    }

    pub fn apply_hotkeys(&mut self) {
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
        self.notification_service.update_config(&self.config);
    }

    pub fn mark_dirty(&mut self) {
        self.config_dirty = true;
    }

    /// Enter hotkey assignment mode for the given target.
    /// Records the start time so the first click that triggered this is ignored.
    pub fn start_assigning_hotkey(&mut self, target: &str) {
        self.assigning_hotkey_for = Some(target.to_string());
        self.hotkey_assign_start = Some(std::time::Instant::now());
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
