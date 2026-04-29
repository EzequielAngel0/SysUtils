# Design Document

## High-Level Design

Este sprint implementa 4 correcciones de bugs y 5 features de alta prioridad para Phantom Native. El diseño mantiene la arquitectura existente basada en traits, `Arc<Mutex<T>>` para concurrencia, y el patrón dirty flag + auto-save.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      SysUtilsApp                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ LogBuffer    │  │ AppConfig    │  │ HotkeyEngine │     │
│  │ (VecDeque)   │  │ + profiles   │  │              │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Sequence Module                                      │  │
│  │  - play_sequence() with MouseMove delta calc        │  │
│  │  - loop_counter: Arc<Mutex<u32>>                    │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Monitor Module                                       │  │
│  │  - sliding reference for color_disappear            │  │
│  │  - preview_thread (500ms capture loop)              │  │
│  │  - preview_pixels: Arc<Mutex<Option<Vec<u8>>>>      │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ UI Tabs (pulse, keepalive, monitor, panic, sequence)│  │
│  │  - hotkey clear button (✕)                          │  │
│  │  - hotkey validation + visual feedback              │  │
│  │  - color picker widget                              │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## Component Design

### Component 1: LogBuffer with VecDeque (B3)

**Location:** `src/models.rs`

**Changes:**
```rust
use std::collections::VecDeque;

pub struct LogBuffer {
    pub entries: Arc<Mutex<VecDeque<LogEntry>>>,  // Changed from Vec
    max_entries: usize,
}

impl LogBuffer {
    pub fn new(max: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::with_capacity(max))),
            max_entries: max,
        }
    }

    pub fn log(&self, level: LogLevel, module: &str, msg: &str) {
        // ... create entry ...
        let mut entries = self.entries.lock().unwrap();
        if entries.len() >= self.max_entries {
            entries.pop_front();  // O(1) instead of remove(0)
        }
        entries.push_back(entry);
    }

    pub fn get_all(&self) -> Vec<LogEntry> {
        self.entries.lock().unwrap().iter().cloned().collect()
    }

    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }
}
```

**Rationale:** `VecDeque::pop_front()` is O(1) vs `Vec::remove(0)` which is O(n). With 500 entries and high log frequency, this eliminates UI stutters.

---

### Component 2: MouseMove Delta Calculation (B1)

**Location:** `src/logic/sequence.rs`

**Changes to `play_sequence()`:**
```rust
std::thread::spawn(move || {
    let actual_loops = if loops == 0 { u32::MAX } else { loops };
    
    for lap in 0..actual_loops {
        if *stop_flag.lock().unwrap() { break; }
        
        let mut last_x = 0i32;
        let mut last_y = 0i32;
        let mut first_move = true;
        
        for evt in &events {
            if *stop_flag.lock().unwrap() { break; }
            
            match evt {
                MacroEvent::MouseMove(x, y) => {
                    if first_move {
                        last_x = *x;
                        last_y = *y;
                        first_move = false;
                        // Send (0, 0) for first move
                        let _ = hw.send("MOUSE_MOVE_REL:0:0");
                    } else {
                        let dx = x - last_x;
                        let dy = y - last_y;
                        let _ = hw.send(&format!("MOUSE_MOVE_REL:{}:{}", dx, dy));
                        last_x = *x;
                        last_y = *y;
                    }
                }
                // ... other events unchanged ...
            }
        }
    }
});
```

**Rationale:** ESP32 expects relative mouse movement. We calculate deltas between consecutive absolute positions recorded during capture.

---

### Component 3: Sliding Reference for color_disappear (B2)

**Location:** `src/logic/monitor.rs` in `start_monitor()`

**Changes:**
```rust
// Inside the monitor thread loop
loop {
    if *stop_flag.lock().unwrap() { break; }
    std::thread::sleep(Duration::from_millis(50));
    
    if let Some(current_img) = capturer.capture_frame() {
        let triggered = match condition.as_str() {
            "color_disappear" => {
                match monitor_mode.as_str() {
                    "REGION" | "FULLSCREEN" => {
                        // Check if color is currently present
                        let is_present = has_color_in_region(&current_img, ...);
                        
                        if is_present {
                            // Update sliding reference
                            baseline_img = current_img.clone();
                            logs.log(LogLevel::Info, "Monitor", "Referencia actualizada (color presente)");
                            false  // Don't trigger yet
                        } else {
                            // Check if it WAS present in the last reference
                            let was_present = has_color_in_region(&baseline_img, ...);
                            was_present  // Trigger if it disappeared
                        }
                    }
                    "PIXEL" => {
                        // Keep existing fixed reference logic
                        // ...
                    }
                    _ => false
                }
            }
            // ... other conditions unchanged ...
        };
        
        if triggered {
            execute_action(&hw, &logs);
        }
    }
}
```

**Rationale:** For REGION and FULLSCREEN modes, we need a "sliding window" reference that updates when the target color IS present, so we can detect when it disappears. PIXEL mode keeps the fixed reference for simplicity.

---

### Component 4: Hotkey Validation & Visual Feedback (B4)

**Location:** All UI tab files (`src/ui/*.rs`)

**New helper function in `src/hotkey_engine.rs`:**
```rust
impl HotkeyEngine {
    pub fn is_valid_key(key_str: &str) -> bool {
        if key_str.is_empty() { return true; }  // Empty is valid
        
        // Parse modifiers
        let parts: Vec<&str> = key_str.split('+').collect();
        let key_part = parts.last().unwrap_or(&"");
        
        // Check if key_part is a valid key name
        matches!(key_part, 
            "F1" | "F2" | "F3" | "F4" | "F5" | "F6" | "F7" | "F8" | "F9" | "F10" | "F11" | "F12" |
            "Escape" | "Space" | "Return" | "Tab" | "Backspace" |
            "MouseLeft" | "MouseRight" | "MouseMiddle" |
            // ... add all valid keys
        ) || (key_part.len() == 1 && key_part.chars().next().unwrap().is_alphanumeric())
    }
}
```

**UI changes (example for `pulse_tab.rs`):**
```rust
ui.horizontal(|ui| {
    ui.label("Hotkey:");
    
    let is_valid = HotkeyEngine::is_valid_key(&self.config.pulse_hotkey);
    let hotkey_text = if self.config.pulse_hotkey.is_empty() {
        "Ninguna".to_string()
    } else {
        self.config.pulse_hotkey.clone()
    };
    
    let mut frame = egui::Frame::none();
    if !is_valid {
        frame = frame.fill(egui::Color32::from_rgba_premultiplied(200, 60, 60, 40));
    }
    
    frame.show(ui, |ui| {
        let btn = ui.add_sized([120.0, 20.0], egui::Button::new(
            egui::RichText::new(&hotkey_text).color(egui::Color32::WHITE)
        ));
        
        if !is_valid {
            btn.on_hover_text("Hotkey inválida — usa formato: F6, Ctrl+F6, MouseLeft");
        }
        
        if btn.clicked() {
            self.assigning_hotkey_for = Some("pulse".to_string());
        }
    });
});
```

**Rationale:** Immediate visual feedback prevents users from saving invalid configurations.

---

### Component 5: Hotkey Clear Button (F1)

**Location:** All UI tab files

**UI pattern (example for `pulse_tab.rs`):**
```rust
ui.horizontal(|ui| {
    ui.label("Hotkey:");
    
    let hotkey_text = if self.config.pulse_hotkey.is_empty() {
        "Ninguna".to_string()
    } else {
        self.config.pulse_hotkey.clone()
    };
    
    if ui.add_sized([120.0, 20.0], egui::Button::new(
        egui::RichText::new(&hotkey_text).color(egui::Color32::WHITE)
    )).clicked() {
        self.assigning_hotkey_for = Some("pulse".to_string());
    }
    
    // Clear button
    let clear_enabled = !self.config.pulse_hotkey.is_empty();
    let clear_btn = ui.add_enabled(clear_enabled, egui::Button::new("✕"));
    if clear_btn.on_hover_text("Quitar hotkey").clicked() {
        self.config.pulse_hotkey.clear();
        self.apply_hotkeys();
        self.mark_dirty();
    }
});
```

**Rationale:** One-click hotkey removal. The button is disabled when the hotkey is already empty to provide clear state feedback.

---

### Component 6: Real-time Monitor Preview (F2)

**Location:** `src/app.rs` (state), `src/logic/monitor.rs` (thread), `src/ui/monitor_tab.rs` (rendering)

**New fields in `SysUtilsApp`:**
```rust
pub struct SysUtilsApp {
    // ... existing fields ...
    
    // Preview thread state
    pub monitor_preview_pixels: Arc<Mutex<Option<Vec<u8>>>>,
    pub monitor_preview_width: Arc<Mutex<usize>>,
    pub monitor_preview_height: Arc<Mutex<usize>>,
    pub monitor_preview_stop: Arc<Mutex<bool>>,
    pub monitor_preview_generation: Arc<Mutex<u64>>,  // Increment on each update
}
```

**Preview thread in `capture_monitor_reference()`:**
```rust
fn capture_monitor_reference(&mut self) {
    // ... existing capture logic ...
    
    // Start preview thread
    *self.monitor_preview_stop.lock().unwrap() = false;
    
    let preview_pixels = self.monitor_preview_pixels.clone();
    let preview_w = self.monitor_preview_width.clone();
    let preview_h = self.monitor_preview_height.clone();
    let preview_stop = self.monitor_preview_stop.clone();
    let preview_gen = self.monitor_preview_generation.clone();
    let target_id = self.monitor_target_id;
    let is_window = self.monitor_is_window;
    let logs = self.logs.clone();
    
    std::thread::spawn(move || {
        let mut capturer = ScreenCapture::new();
        capturer.target_id = target_id;
        capturer.is_window = is_window;
        
        loop {
            if *preview_stop.lock().unwrap() { break; }
            
            if let Some(frame) = capturer.capture_frame() {
                *preview_w.lock().unwrap() = frame.width() as usize;
                *preview_h.lock().unwrap() = frame.height() as usize;
                *preview_pixels.lock().unwrap() = Some(frame.into_raw());
                *preview_gen.lock().unwrap() += 1;
            } else {
                logs.log(LogLevel::Warning, "Monitor", "Preview capture failed");
            }
            
            std::thread::sleep(Duration::from_millis(500));
        }
    });
}
```

**UI rendering in `monitor_tab.rs`:**
```rust
// Track last generation we rendered
if self.monitor_preview_last_gen != *self.monitor_preview_generation.lock().unwrap() {
    // New frame available
    let pixels = self.monitor_preview_pixels.lock().unwrap();
    let w = *self.monitor_preview_width.lock().unwrap();
    let h = *self.monitor_preview_height.lock().unwrap();
    
    if let Some(ref rgba) = *pixels {
        let image = egui::ColorImage::from_rgba_unmultiplied([w, h], rgba);
        self.monitor_preview_texture = Some(ui.ctx().load_texture(
            "monitor_preview",
            image,
            egui::TextureOptions::LINEAR,
        ));
        self.monitor_preview_last_gen = *self.monitor_preview_generation.lock().unwrap();
    }
}
```

**Rationale:** Separate preview thread from monitoring thread allows real-time feedback without interfering with detection logic. Generation counter prevents redundant texture updates.

---

### Component 7: Configuration Profiles (F3)

**Location:** `src/config.rs` (profile management), `src/app.rs` (state), `src/main.rs` (UI)

**New field in `AppConfig`:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // ... existing fields ...
    
    #[serde(default)]
    pub active_profile: String,  // Name of currently active profile
}
```

**New methods in `AppConfig`:**
```rust
impl AppConfig {
    pub fn profile_path(name: &str) -> PathBuf {
        let mut path = std::env::current_exe().unwrap_or_default();
        path.pop();
        path.push(format!("sysutils_profile_{}.json", name));
        path
    }
    
    pub fn save_as_profile(&self, name: &str) -> Result<(), String> {
        let path = Self::profile_path(name);
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialize error: {}", e))
            .and_then(|json| {
                std::fs::write(path, json).map_err(|e| format!("Write error: {}", e))
            })
    }
    
    pub fn load_profile(name: &str) -> Result<Self, String> {
        let path = Self::profile_path(name);
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("Read error: {}", e))?;
        serde_json::from_str(&contents)
            .map_err(|e| format!("Parse error: {}", e))
    }
    
    pub fn delete_profile(name: &str) -> Result<(), String> {
        let path = Self::profile_path(name);
        std::fs::remove_file(path)
            .map_err(|e| format!("Delete error: {}", e))
    }
    
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
```

**New fields in `SysUtilsApp`:**
```rust
pub struct SysUtilsApp {
    // ... existing fields ...
    pub available_profiles: Vec<String>,
    pub profile_name_input: String,  // For creating new profiles
}
```

**UI in `main.rs` (connection bar area):**
```rust
ui.horizontal(|ui| {
    // ... existing port selection ...
    
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
            
            for profile in &self.available_profiles {
                if ui.selectable_label(&self.config.active_profile == profile, profile).clicked() {
                    match AppConfig::load_profile(profile) {
                        Ok(loaded) => {
                            self.config = loaded;
                            self.config.active_profile = profile.clone();
                            self.config.save();
                            self.apply_hotkeys();
                            self.logs.log(LogLevel::Action, "Config", &format!("Perfil '{}' cargado", profile));
                        }
                        Err(e) => {
                            self.logs.log(LogLevel::Error, "Config", &format!("Error cargando perfil: {}", e));
                        }
                    }
                }
            }
        });
    
    if ui.button("💾").on_hover_text("Guardar perfil actual").clicked() {
        // Show dialog for profile name
        self.show_save_profile_dialog = true;
    }
    
    if ui.button("🗑").on_hover_text("Eliminar perfil actual").clicked() {
        if !self.config.active_profile.is_empty() {
            match AppConfig::delete_profile(&self.config.active_profile) {
                Ok(()) => {
                    self.available_profiles = AppConfig::scan_profiles();
                    self.config.active_profile.clear();
                    self.logs.log(LogLevel::Action, "Config", "Perfil eliminado");
                }
                Err(e) => {
                    self.logs.log(LogLevel::Error, "Config", &format!("Error eliminando: {}", e));
                }
            }
        }
    }
});
```

**Rationale:** Profiles are stored as separate JSON files. The active profile name is persisted in the main config. Scanning happens at startup and after save/delete operations.

---

### Component 8: Color Picker Widget (F4)

**Location:** `src/ui/monitor_tab.rs`

**Changes:**
```rust
// Replace the three DragValue widgets for R, G, B with:
if self.config.monitor_condition == "color_appear" || self.config.monitor_condition == "color_disappear" {
    ui.horizontal(|ui| {
        ui.label("Color objetivo:");
        
        // Convert u8 to f32 for egui color picker
        let mut color = [
            self.config.monitor_target_color_r as f32 / 255.0,
            self.config.monitor_target_color_g as f32 / 255.0,
            self.config.monitor_target_color_b as f32 / 255.0,
        ];
        
        if egui::color_picker::color_edit_button_rgb(ui, &mut color).changed() {
            self.config.monitor_target_color_r = (color[0] * 255.0) as u8;
            self.config.monitor_target_color_g = (color[1] * 255.0) as u8;
            self.config.monitor_target_color_b = (color[2] * 255.0) as u8;
            self.mark_dirty();
        }
        
        // Preview swatch (keep existing)
        let preview_color = egui::Color32::from_rgb(
            self.config.monitor_target_color_r,
            self.config.monitor_target_color_g,
            self.config.monitor_target_color_b,
        );
        let (resp, painter) = ui.allocate_painter(egui::vec2(24.0, 16.0), egui::Sense::hover());
        painter.rect_filled(resp.rect, egui::CornerRadius::same(3), preview_color);
    });
    
    // ... tolerance field unchanged ...
}
```

**Rationale:** `egui::color_picker::color_edit_button_rgb` is built-in and requires no additional dependencies. It provides a standard color picker UI with visual feedback.

---

### Component 9: Loop Counter Display (F5)

**Location:** `src/app.rs` (state), `src/logic/sequence.rs` (update), `src/ui/sequence_tab.rs` (display)

**New field in `SysUtilsApp`:**
```rust
pub struct SysUtilsApp {
    // ... existing fields ...
    pub sequence_loop_counter: Arc<Mutex<u32>>,
}
```

**Initialization in `SysUtilsApp::new()`:**
```rust
sequence_loop_counter: Arc::new(Mutex::new(0)),
```

**Update in `play_sequence()`:**
```rust
std::thread::spawn(move || {
    let actual_loops = if loops == 0 { u32::MAX } else { loops };
    
    // Reset counter
    *loop_counter.lock().unwrap() = 0;
    
    for lap in 0..actual_loops {
        if *stop_flag.lock().unwrap() { break; }
        
        // Increment counter at start of each loop
        *loop_counter.lock().unwrap() = lap + 1;
        
        logs.log(LogLevel::Info, "Sequence", &format!("Loop {}/{}", lap + 1, if loops == 0 { "∞".to_string() } else { loops.to_string() }));
        
        // ... execute events ...
    }
    
    logs.log(LogLevel::Action, "Sequence", "Reproducción finalizada");
});
```

**Display in `sequence_tab.rs`:**
```rust
// After the play/stop buttons
if self.sequence_playing {
    let current_loop = *self.sequence_loop_counter.lock().unwrap();
    let total_loops: u32 = self.sequence_loops.parse().unwrap_or(1);
    let total_str = if total_loops == 0 { "∞".to_string() } else { total_loops.to_string() };
    
    ui.label(
        egui::RichText::new(format!("Loop: {} / {}", current_loop, total_str))
            .size(13.0)
            .color(egui::Color32::from_rgb(100, 220, 140))
    );
}
```

**Rationale:** Simple counter shared between playback thread and UI. Updated at the start of each loop iteration for accurate real-time feedback.

---

## Data Models

### Modified: LogBuffer
```rust
pub struct LogBuffer {
    pub entries: Arc<Mutex<VecDeque<LogEntry>>>,  // Changed from Vec
    max_entries: usize,
}
```

### Modified: AppConfig
```rust
pub struct AppConfig {
    // ... all existing fields ...
    
    #[serde(default)]
    pub active_profile: String,
}
```

### Modified: SysUtilsApp
```rust
pub struct SysUtilsApp {
    // ... all existing fields ...
    
    // F2: Preview thread
    pub monitor_preview_pixels: Arc<Mutex<Option<Vec<u8>>>>,
    pub monitor_preview_width: Arc<Mutex<usize>>,
    pub monitor_preview_height: Arc<Mutex<usize>>,
    pub monitor_preview_stop: Arc<Mutex<bool>>,
    pub monitor_preview_generation: Arc<Mutex<u64>>,
    pub monitor_preview_last_gen: u64,
    
    // F3: Profiles
    pub available_profiles: Vec<String>,
    pub profile_name_input: String,
    pub show_save_profile_dialog: bool,
    
    // F5: Loop counter
    pub sequence_loop_counter: Arc<Mutex<u32>>,
}
```

---

## Error Handling

### LogBuffer Operations
- **Scenario:** Lock poisoning on `entries` mutex
- **Handling:** Use `.unwrap()` as lock poisoning indicates unrecoverable panic in another thread

### Profile Operations
- **Scenario:** Profile file doesn't exist or is corrupted
- **Handling:** Return `Result<T, String>` with descriptive error message, log to LogBuffer with `Error` level, continue with current config

### Preview Thread
- **Scenario:** Frame capture fails
- **Handling:** Log warning, skip frame, continue loop. Don't crash the thread.

### Hotkey Validation
- **Scenario:** Invalid hotkey string
- **Handling:** Visual feedback (red background), tooltip with format hint, prevent registration in `apply_hotkeys()`

### MouseMove Delta
- **Scenario:** No MouseMove events in sequence
- **Handling:** No-op, existing behavior for other events unchanged

---

## Testing Strategy

### Unit Tests
- `LogBuffer::log()` with VecDeque rotation
- `AppConfig::scan_profiles()` with mock filesystem
- `HotkeyEngine::is_valid_key()` with various inputs
- Delta calculation for MouseMove with edge cases (first move, single move, no moves)

### Integration Tests
- Profile save → load → verify config matches
- Preview thread start → capture → stop → verify no leaks
- Sequence playback with MouseMove → verify ESP32 receives MOUSE_MOVE_REL commands

### Manual Testing
- Hotkey clear button in all tabs
- Color picker updates config correctly
- Loop counter updates in real-time during playback
- Preview updates every ~500ms
- Sliding reference for color_disappear in REGION mode

---

## Performance Considerations

### LogBuffer
- **Before:** O(n) rotation with `Vec::remove(0)` → ~500 operations per rotation
- **After:** O(1) rotation with `VecDeque::pop_front()` → constant time
- **Impact:** Eliminates UI stutters during high-frequency logging

### Preview Thread
- **Overhead:** One additional capture every 500ms
- **Mitigation:** Independent thread, doesn't block UI or monitoring logic
- **Memory:** One extra frame buffer (~1920x1080x4 = 8MB worst case)

### Profile Scanning
- **Overhead:** Filesystem scan on startup
- **Mitigation:** Cached in `available_profiles`, only re-scanned after save/delete

---

## Security Considerations

### Profile Files
- **Risk:** Malicious JSON could crash the app
- **Mitigation:** Use `serde_json::from_str()` which validates structure, catch errors and log

### Hotkey Validation
- **Risk:** Invalid hotkey strings could cause panics in HotkeyEngine
- **Mitigation:** Validate before registration, visual feedback prevents invalid input

---

## Deployment Notes

### Breaking Changes
- None. All changes are backward-compatible.
- Existing `sysutils_config.json` files will load correctly (new fields have `#[serde(default)]`)

### Migration Path
- Users with existing configs: No action required
- LogBuffer change is internal, no data migration needed
- Profiles are opt-in feature, default behavior unchanged

---

## Future Enhancements

### Potential Improvements
- Profile import/export (share profiles between machines)
- Preview thread FPS control (user-configurable interval)
- Hotkey conflict detection (warn if two modules use same key)
- MouseMove interpolation (smooth movement between recorded positions)
- Sliding reference configurable window size (update every N frames instead of every frame)

---

*Design Document v1.0 — Bugfix & v1.1 Sprint*
