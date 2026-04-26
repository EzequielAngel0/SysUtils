use eframe::egui;
use std::sync::{Arc, Mutex};

// ═══════════════════════════════════════════════════════════════════════════════
// Shared log buffer
// ═══════════════════════════════════════════════════════════════════════════════
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LogLevel {
    Info,
    Action,
    Warning,
    Error,
}

#[allow(dead_code)]
impl LogLevel {
    pub fn color(&self) -> egui::Color32 {
        match self {
            LogLevel::Info    => egui::Color32::from_rgb(140, 140, 160),
            LogLevel::Action  => egui::Color32::from_rgb(100, 200, 140),
            LogLevel::Warning => egui::Color32::from_rgb(240, 200, 60),
            LogLevel::Error   => egui::Color32::from_rgb(240, 80, 80),
        }
    }
    pub fn icon(&self) -> &str {
        match self {
            LogLevel::Info    => "ℹ",
            LogLevel::Action  => "▶",
            LogLevel::Warning => "⚠",
            LogLevel::Error   => "✗",
        }
    }
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub module: String,
    pub message: String,
}

#[derive(Clone)]
pub struct LogBuffer {
    pub entries: Arc<Mutex<Vec<LogEntry>>>,
    max_entries: usize,
}

impl LogBuffer {
    pub fn new(max: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::with_capacity(max))),
            max_entries: max,
        }
    }

    pub fn log(&self, level: LogLevel, module: &str, msg: &str) {
        let now = chrono::Local::now().format("%H:%M:%S").to_string();
        let entry = LogEntry {
            timestamp: now,
            level,
            module: module.to_string(),
            message: msg.to_string(),
        };
        let mut entries = self.entries.lock().unwrap();
        if entries.len() >= self.max_entries {
            entries.remove(0);
        }
        entries.push(entry);
    }

    pub fn get_all(&self) -> Vec<LogEntry> {
        self.entries.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Sequence (Macro) event types
// ═══════════════════════════════════════════════════════════════════════════════
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum MacroEvent {
    KeyDown(String),
    KeyUp(String),
    MouseDown(String),
    MouseUp(String),
    MouseMove(i32, i32),
    Delay(u64), // ms
}

impl MacroEvent {
    pub fn display(&self) -> String {
        match self {
            MacroEvent::KeyDown(k)    => format!("⌨ KEY_DOWN: {}", k),
            MacroEvent::KeyUp(k)      => format!("⌨ KEY_UP: {}", k),
            MacroEvent::MouseDown(b)  => format!("🖱 CLK_DOWN: {}", b),
            MacroEvent::MouseUp(b)    => format!("🖱 CLK_UP: {}", b),
            MacroEvent::MouseMove(x,y)=> format!("↗ MOVE: ({}, {})", x, y),
            MacroEvent::Delay(ms)     => format!("⏱ WAIT: {}ms", ms),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tabs
// ═══════════════════════════════════════════════════════════════════════════════
#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Pulse,
    KeepAlive,
    Monitor,
    Panic,
    Sequence,
    #[allow(dead_code)]
    Logs,
}
