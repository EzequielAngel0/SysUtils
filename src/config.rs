// ═══════════════════════════════════════════════════════════════════════════════
// config.rs — Persistent configuration (saved as JSON on disk)
// ═══════════════════════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// All persistent application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // Connection
    pub last_port: String,

    // Pulse (Clicker)
    pub pulse_min_delay: u32,
    pub pulse_max_delay: u32,
    pub pulse_target: String,       // "L", "R", "M" for mouse
    pub pulse_mode: String,         // "PULSE" or "HOLD"
    pub pulse_hotkey: String,       // e.g. "F6" or "Ctrl+F6"
    pub pulse_enabled: bool,
    pub pulse_input_type: String,   // "mouse" or "keyboard"
    pub pulse_key: String,          // keyboard key, e.g. "e", "f", "space"

    // KeepAlive (Anti-AFK)
    pub keepalive_keys: String,
    pub keepalive_hold_ms: u32,
    pub keepalive_interval_min: f32,
    pub keepalive_interval_max: f32,
    pub keepalive_enabled: bool,
    pub keepalive_hotkey: String,

    // Diagnostics (Vision)
    pub monitor_enabled: bool,
    pub monitor_target_id: usize,
    pub monitor_is_window: bool,
    pub monitor_duration_ms: u32,
    pub monitor_mode: String,           // "FULLSCREEN", "PIXEL", "REGION"
    pub monitor_pixel_x: u32,
    pub monitor_pixel_y: u32,
    pub monitor_region_x: u32,
    pub monitor_region_y: u32,
    pub monitor_region_w: u32,
    pub monitor_region_h: u32,
    pub monitor_tolerance: u8,
    pub monitor_sample_step: usize,
    pub monitor_click_action: String,   // "Left", "Right", "Middle"
    pub monitor_hotkey: String,
    // Condition system (point 5)
    pub monitor_condition: String,      // "change", "color_appear", "color_disappear"
    pub monitor_target_color_r: u8,
    pub monitor_target_color_g: u8,
    pub monitor_target_color_b: u8,
    pub monitor_color_tolerance: u8,
    // Action type for monitor
    pub monitor_action_type: String,    // "mouse_click" or "key_press"
    pub monitor_action_key: String,     // keyboard key for action

    // Panic Switch
    pub panic_threshold: f64,
    pub panic_enabled: bool,
    pub panic_check_interval_ms: u64,
    pub panic_hotkey: String,

    // Sequence
    pub sequence_hotkey_record: String,
    pub sequence_hotkey_play: String,

    // System
    pub file_logging_enabled: bool,

    // UI
    pub window_width: f32,
    pub window_height: f32,

    // Profiles (F3)
    #[serde(default)]
    pub active_profile: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            last_port: String::new(),
            pulse_min_delay: 50,
            pulse_max_delay: 100,
            pulse_target: "L".into(),
            pulse_mode: "PULSE".into(),
            pulse_hotkey: "F6".into(),
            pulse_enabled: false,
            pulse_input_type: "mouse".into(),
            pulse_key: "e".into(),
            keepalive_keys: "w".into(),
            keepalive_hold_ms: 200,
            keepalive_interval_min: 4.0,
            keepalive_interval_max: 7.0,
            keepalive_enabled: false,
            keepalive_hotkey: "F7".into(),
            monitor_enabled: false,
            monitor_target_id: 0,
            monitor_is_window: false,
            monitor_duration_ms: 500,
            monitor_mode: "FULLSCREEN".into(),
            monitor_pixel_x: 0,
            monitor_pixel_y: 0,
            monitor_region_x: 0,
            monitor_region_y: 0,
            monitor_region_w: 100,
            monitor_region_h: 100,
            monitor_tolerance: 50,
            monitor_sample_step: 3,
            monitor_click_action: "Left".into(),
            monitor_hotkey: "F5".into(),
            monitor_condition: "change".into(),
            monitor_target_color_r: 255,
            monitor_target_color_g: 0,
            monitor_target_color_b: 0,
            monitor_color_tolerance: 30,
            monitor_action_type: "mouse_click".into(),
            monitor_action_key: "e".into(),
            panic_threshold: 100.0,
            panic_enabled: false,
            panic_check_interval_ms: 1000,
            panic_hotkey: "Escape".into(),
            sequence_hotkey_record: "F8".into(),
            sequence_hotkey_play: "F9".into(),
            file_logging_enabled: false,
            window_width: 1050.0,
            window_height: 700.0,
            active_profile: String::new(),
        }
    }
}

impl AppConfig {
    fn config_path() -> PathBuf {
        let mut path = std::env::current_exe().unwrap_or_default();
        path.pop();
        path.push("sysutils_config.json");
        path
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    /// Export config to a user-chosen path
    pub fn export_to(&self, path: &std::path::Path) -> Result<(), String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialize error: {}", e))
            .and_then(|json| {
                std::fs::write(path, json).map_err(|e| format!("Write error: {}", e))
            })
    }

    /// Import config from a file path
    pub fn import_from(path: &std::path::Path) -> Result<Self, String> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("Read error: {}", e))?;
        serde_json::from_str(&contents)
            .map_err(|e| format!("Parse error: {}", e))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Profile Management (F3)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get the path for a named profile
    pub fn profile_path(name: &str) -> PathBuf {
        let mut path = std::env::current_exe().unwrap_or_default();
        path.pop();
        path.push(format!("sysutils_profile_{}.json", name));
        path
    }

    /// Save current config as a named profile
    pub fn save_as_profile(&self, name: &str) -> Result<(), String> {
        let path = Self::profile_path(name);
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialize error: {}", e))
            .and_then(|json| {
                std::fs::write(path, json).map_err(|e| format!("Write error: {}", e))
            })
    }

    /// Load a profile from file
    pub fn load_profile(name: &str) -> Result<Self, String> {
        let path = Self::profile_path(name);
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("Read error: {}", e))?;
        serde_json::from_str(&contents)
            .map_err(|e| format!("Parse error: {}", e))
    }

    /// Delete a profile file
    pub fn delete_profile(name: &str) -> Result<(), String> {
        let path = Self::profile_path(name);
        std::fs::remove_file(path)
            .map_err(|e| format!("Delete error: {}", e))
    }

    /// Scan directory for available profiles
    pub fn scan_profiles() -> Vec<String> {
        let mut profiles = Vec::new();
        let exe_dir = std::env::current_exe().unwrap_or_default();
        let dir = exe_dir.parent().unwrap_or(std::path::Path::new("."));
        
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("sysutils_profile_") && name.ends_with(".json") {
                        let profile_name = name
                            .strip_prefix("sysutils_profile_").unwrap()
                            .strip_suffix(".json").unwrap();
                        profiles.push(profile_name.to_string());
                    }
                }
            }
        }
        profiles.sort();
        profiles
    }
}
