// ═══════════════════════════════════════════════════════════════════════════════
// system_info.rs — CPU/RAM monitoring + Anti-cheat process detection
// ═══════════════════════════════════════════════════════════════════════════════

use sysinfo::System;
use std::sync::{Arc, Mutex};

/// Known anti-cheat process names (lowercase)
const ANTICHEAT_PROCESSES: &[(&str, &str)] = &[
    ("easyanticheat", "EasyAntiCheat (EAC)"),
    ("easyanticheat_eos", "EasyAntiCheat EOS"),
    ("beclient", "BattlEye Client"),
    ("beservice", "BattlEye Service"),
    ("vgc", "Riot Vanguard"),
    ("vgtray", "Riot Vanguard Tray"),
    ("faceitclient", "FACEIT Anti-Cheat"),
    ("faceitservice", "FACEIT Service"),
    ("equ8", "EQU8 Anti-Cheat"),
    ("xigncode", "XIGNCODE3"),
    ("gameguard", "GameGuard (nProtect)"),
    ("uncheater", "UnCheater"),
    ("mhyprot", "miHoYo Protect"),
    ("atvi-pillbox", "Ricochet (Activision)"),
];

pub struct SystemInfo {
    sys: System,
    pub cpu_usage: f32,
    pub ram_usage_mb: f64,
    pub anticheat_warnings: Vec<String>,
}

impl SystemInfo {
    pub fn new() -> Self {
        Self {
            sys: System::new_all(),
            cpu_usage: 0.0,
            ram_usage_mb: 0.0,
            anticheat_warnings: Vec::new(),
        }
    }

    /// Refresh CPU and RAM usage for the current process
    pub fn refresh_usage(&mut self) {
        self.sys.refresh_all();

        let pid = sysinfo::get_current_pid().ok();
        if let Some(pid) = pid {
            if let Some(process) = self.sys.process(pid) {
                self.cpu_usage = process.cpu_usage();
                self.ram_usage_mb = process.memory() as f64 / 1024.0 / 1024.0;
            }
        }
    }

    /// Scan for running anti-cheat processes
    pub fn scan_anticheat(&mut self) {
        self.sys.refresh_all();
        self.anticheat_warnings.clear();

        for (_, process) in self.sys.processes() {
            let name = process.name().to_string_lossy().to_lowercase();
            for &(pattern, display_name) in ANTICHEAT_PROCESSES {
                if name.contains(pattern) {
                    let warning = display_name.to_string();
                    if !self.anticheat_warnings.contains(&warning) {
                        self.anticheat_warnings.push(warning);
                    }
                }
            }
        }
    }
}

/// Thread-safe wrapper for use from SysUtilsApp
pub struct SystemMonitor {
    pub info: Arc<Mutex<SystemInfo>>,
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self {
            info: Arc::new(Mutex::new(SystemInfo::new())),
        }
    }

    pub fn cpu_usage(&self) -> f32 {
        self.info.lock().unwrap().cpu_usage
    }

    pub fn ram_usage_mb(&self) -> f64 {
        self.info.lock().unwrap().ram_usage_mb
    }

    pub fn anticheat_warnings(&self) -> Vec<String> {
        self.info.lock().unwrap().anticheat_warnings.clone()
    }

    pub fn refresh(&self) {
        let mut info = self.info.lock().unwrap();
        info.refresh_usage();
    }

    pub fn scan_anticheat(&self) {
        let mut info = self.info.lock().unwrap();
        info.scan_anticheat();
    }
}
