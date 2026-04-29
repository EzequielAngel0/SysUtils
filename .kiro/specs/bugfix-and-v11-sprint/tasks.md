# Implementation Tasks

## Task 1: Fix LogBuffer O(n) rotation (B3)

- [x] 1.1 Change `LogBuffer.entries` from `Arc<Mutex<Vec<LogEntry>>>` to `Arc<Mutex<VecDeque<LogEntry>>>` in `src/models.rs`
- [x] 1.2 Update `LogBuffer::new()` to use `VecDeque::with_capacity(max)`
- [x] 1.3 Replace `entries.remove(0)` with `entries.pop_front()` in `LogBuffer::log()`
- [x] 1.4 Update `LogBuffer::get_all()` to return `Vec<LogEntry>` by collecting from VecDeque iterator
- [x] 1.5 Verify `LogBuffer::clear()` works with VecDeque
- [x] 1.6 Add `use std::collections::VecDeque;` import at top of `models.rs`

## Task 2: Implement MouseMove delta calculation (B1)

- [x] 2.1 In `play_sequence()` in `src/logic/sequence.rs`, add variables `let mut last_x = 0i32; let mut last_y = 0i32; let mut first_move = true;` before the event loop
- [x] 2.2 Replace the `MacroEvent::MouseMove(_, _) => { // skip }` arm with delta calculation logic
- [x] 2.3 For first MouseMove, initialize `last_x` and `last_y` to the event coordinates and send `MOUSE_MOVE_REL:0:0`
- [x] 2.4 For subsequent MouseMove events, calculate `dx = x - last_x` and `dy = y - last_y`
- [x] 2.5 Send `MOUSE_MOVE_REL:dx:dy` command to ESP32 via `hw.send()`
- [x] 2.6 Update `last_x` and `last_y` after each MouseMove
- [x] 2.7 Reset `first_move` flag at the start of each loop iteration

## Task 3: Fix color_disappear sliding reference (B2)

- [x] 3.1 In `start_monitor()` in `src/logic/monitor.rs`, locate the `"color_disappear"` condition arm
- [x] 3.2 For `"REGION"` and `"FULLSCREEN"` modes, add logic to check if target color is currently present
- [x] 3.3 If color IS present, update `baseline_img = current_img.clone()` and log "Referencia actualizada"
- [x] 3.4 If color is NOT present, check if it WAS present in `baseline_img` and trigger if true
- [x] 3.5 Keep `"PIXEL"` mode logic unchanged (fixed reference)
- [x] 3.6 Test with a color that appears and disappears to verify sliding reference works

## Task 4: Add hotkey validation and visual feedback (B4)

- [x] 4.1 Add `is_valid_key(key_str: &str) -> bool` static method to `HotkeyEngine` in `src/hotkey_engine.rs`
- [x] 4.2 Implement validation logic checking against known key names (F1-F12, alphanumeric, MouseLeft/Right/Middle, modifiers)
- [x] 4.3 In `src/ui/pulse_tab.rs`, wrap hotkey button in `egui::Frame` with red background if invalid
- [x] 4.4 Add `.on_hover_text("Hotkey inválida — usa formato: F6, Ctrl+F6, MouseLeft")` tooltip when invalid
- [x] 4.5 Repeat steps 4.3-4.4 for `keepalive_tab.rs`, `monitor_tab.rs`, `panic_tab.rs`, `sequence_tab.rs` (both record and play hotkeys)
- [x] 4.6 In each tab, call `HotkeyEngine::is_valid_key(&self.config.{module}_hotkey)` to determine validity

## Task 5: Add hotkey clear button (F1)

- [x] 5.1 In `src/ui/pulse_tab.rs`, add `ui.add_enabled(!self.config.pulse_hotkey.is_empty(), egui::Button::new("✕"))` after hotkey button
- [x] 5.2 On click, call `self.config.pulse_hotkey.clear()`, `self.apply_hotkeys()`, and `self.mark_dirty()`
- [x] 5.3 Add `.on_hover_text("Quitar hotkey")` tooltip to the clear button
- [x] 5.4 Repeat steps 5.1-5.3 for `keepalive_tab.rs` (keepalive_hotkey)
- [x] 5.5 Repeat steps 5.1-5.3 for `monitor_tab.rs` (monitor_hotkey)
- [x] 5.6 Repeat steps 5.1-5.3 for `panic_tab.rs` (panic_hotkey)
- [x] 5.7 Repeat steps 5.1-5.3 for `sequence_tab.rs` (sequence_hotkey_record and sequence_hotkey_play)

## Task 6: Implement real-time monitor preview (F2)

- [x] 6.1 Add new fields to `SysUtilsApp` in `src/app.rs`: `monitor_preview_pixels: Arc<Mutex<Option<Vec<u8>>>>`, `monitor_preview_width/height: Arc<Mutex<usize>>`, `monitor_preview_stop: Arc<Mutex<bool>>`, `monitor_preview_generation: Arc<Mutex<u64>>`, `monitor_preview_last_gen: u64`
- [x] 6.2 Initialize all new fields in `SysUtilsApp::new()`
- [x] 6.3 In `capture_monitor_reference()` in `src/logic/monitor.rs`, start a new preview thread after capturing the reference
- [x] 6.4 Preview thread: loop with 500ms sleep, capture frame, update preview_pixels/width/height, increment generation counter
- [x] 6.5 Preview thread: check `preview_stop` flag and break loop if true
- [x] 6.6 In `stop_monitor()`, set `*self.monitor_preview_stop.lock().unwrap() = true`
- [x] 6.7 In `render_monitor_tab()` in `src/ui/monitor_tab.rs`, check if `monitor_preview_generation` changed
- [x] 6.8 If generation changed, read `preview_pixels` and update `monitor_preview_texture` with new frame
- [x] 6.9 Update `monitor_preview_last_gen` after texture update to avoid redundant updates

## Task 7: Implement configuration profiles (F3)

- [x] 7.1 Add `active_profile: String` field to `AppConfig` in `src/config.rs` with `#[serde(default)]`
- [x] 7.2 Add `profile_path(name: &str) -> PathBuf` static method returning `sysutils_profile_{name}.json` path
- [x] 7.3 Add `save_as_profile(&self, name: &str) -> Result<(), String>` method to save config as named profile
- [x] 7.4 Add `load_profile(name: &str) -> Result<Self, String>` static method to load profile from file
- [x] 7.5 Add `delete_profile(name: &str) -> Result<(), String>` static method to delete profile file
- [x] 7.6 Add `scan_profiles() -> Vec<String>` static method to scan directory for `sysutils_profile_*.json` files
- [x] 7.7 Add `available_profiles: Vec<String>`, `profile_name_input: String`, `show_save_profile_dialog: bool` fields to `SysUtilsApp`
- [x] 7.8 In `SysUtilsApp::new()`, call `AppConfig::scan_profiles()` and store in `available_profiles`
- [x] 7.9 In `src/main.rs` connection bar, add `ComboBox` for profile selection with "Default" + scanned profiles
- [x] 7.10 On profile selection, call `AppConfig::load_profile()`, update `self.config`, call `apply_hotkeys()`, log action
- [x] 7.11 Add "💾" button to show save profile dialog (set `show_save_profile_dialog = true`)
- [x] 7.12 Add "🗑" button to delete current profile (call `AppConfig::delete_profile()`, rescan, clear active_profile)
- [x] 7.13 Implement save profile dialog window with text input for profile name and Save/Cancel buttons
- [x] 7.14 On Save in dialog, call `config.save_as_profile(name)`, rescan profiles, update active_profile

## Task 8: Add color picker widget (F4)

- [x] 8.1 In `src/ui/monitor_tab.rs`, locate the RGB input section for `monitor_target_color_r/g/b`
- [x] 8.2 Create temporary `let mut color = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];` array
- [x] 8.3 Replace three `DragValue` widgets with `egui::color_picker::color_edit_button_rgb(ui, &mut color)`
- [x] 8.4 On `.changed()`, convert back to u8: `self.config.monitor_target_color_r = (color[0] * 255.0) as u8` (same for g, b)
- [x] 8.5 Call `self.mark_dirty()` when color changes
- [x] 8.6 Keep the existing color preview swatch after the picker
- [x] 8.7 Keep the tolerance `DragValue` unchanged

## Task 9: Add loop counter display (F5)

- [x] 9.1 Add `sequence_loop_counter: Arc<Mutex<u32>>` field to `SysUtilsApp` in `src/app.rs`
- [x] 9.2 Initialize `sequence_loop_counter: Arc::new(Mutex::new(0))` in `SysUtilsApp::new()`
- [x] 9.3 In `play_sequence()` in `src/logic/sequence.rs`, clone `loop_counter` from self before spawning thread
- [x] 9.4 At start of thread, reset counter: `*loop_counter.lock().unwrap() = 0;`
- [x] 9.5 At start of each loop iteration, update counter: `*loop_counter.lock().unwrap() = lap + 1;`
- [x] 9.6 In `render_sequence_tab()` in `src/ui/sequence_tab.rs`, check if `self.sequence_playing` is true
- [x] 9.7 If playing, read `*self.sequence_loop_counter.lock().unwrap()` and parse `self.sequence_loops`
- [x] 9.8 Display `"Loop: {current} / {total}"` label with green color (or "∞" if loops is 0)
- [x] 9.9 Position the label below the play/stop buttons or in the status area

## Task 10: Testing and validation

- [x] 10.1 Test LogBuffer with high-frequency logging (500+ entries) and verify no UI stutters
- [x] 10.2 Record a sequence with MouseMove events and verify playback sends MOUSE_MOVE_REL commands
- [x] 10.3 Test color_disappear in REGION mode with a color that appears and disappears
- [x] 10.4 Test hotkey validation with invalid inputs (e.g., "InvalidKey123") and verify red background + tooltip
- [x] 10.5 Test hotkey clear button in all tabs and verify hotkey is removed and not registered
- [x] 10.6 Test preview thread updates every ~500ms while on Monitor tab
- [x] 10.7 Create, load, and delete profiles and verify config changes correctly
- [x] 10.8 Test color picker updates RGB values correctly
- [x] 10.9 Test loop counter displays correctly during sequence playback with various loop counts (1, 5, 0/infinite)
- [x] 10.10 Run full regression test: all modules (Pulse, KeepAlive, Monitor, Panic, Sequence) work as before

## Task 11: Rebranding a SysUtils

- [x] 11.1 Renombrar título de ventana a "SysUtils" en `src/main.rs`
- [x] 11.2 Actualizar heading del sidebar a "⚙ SysUtils"
- [x] 11.3 Actualizar `Cargo.toml` descripción y versión a 1.1.0
- [x] 11.4 Reescribir `Manager.ps1` con rutas corregidas y nombre actualizado
- [x] 11.5 Renombrar bat launcher → `SysUtils.bat`
- [x] 11.6 Actualizar `manager.sh` con nombre y rutas correctas
- [x] 11.7 Actualizar `KIRO.md`, `CLAUDE.md`, `README.md` y `roadmap.md`

## Task 12: Fix bug de selección de hotkeys (clic izquierdo capturado)

- [x] 12.1 Agregar campo `hotkey_assign_start: Option<Instant>` a `SysUtilsApp`
- [x] 12.2 Crear método `start_assigning_hotkey(target)` que setea ambos campos con timestamp
- [x] 12.3 En el loop de captura, ignorar eventos durante los primeros 200ms (cooldown)
- [x] 12.4 Drenar el canal `raw_rx` durante el cooldown para evitar acumulación
- [x] 12.5 Limpiar `hotkey_assign_start` al completar o cancelar la asignación
- [x] 12.6 Reemplazar todos los `assigning_hotkey_for = Some(...)` por `start_assigning_hotkey()` en los 6 tabs

## Task 13: Scripts de instalación para Linux

- [x] 13.1 Crear `manager.sh` equivalente al `Manager.ps1` para bash
- [x] 13.2 Instalar en `~/.local/share/sysutils-manager/` sin necesitar sudo
- [x] 13.3 Crear symlink en `~/.local/bin/sysutils-manager`
- [x] 13.4 Generar entrada `.desktop` para el lanzador de aplicaciones
- [x] 13.5 Implementar opción de desinstalación completa

## Task 14: Sobreescribir perfiles con configuración actual

- [x] 14.1 Agregar botón "↺" en la barra de conexión entre 💾 y 🗑
- [x] 14.2 Botón habilitado solo cuando hay un perfil activo (no en "Default")
- [x] 14.3 Al hacer clic, sobreescribir `sysutils_profile_{nombre}.json` con config actual
- [x] 14.4 Mostrar mensaje en log y barra de estado al completar

## Task 15: Configuración en tiempo real + bloqueo durante ejecución

- [x] 15.1 En Pulse: enviar config al ESP32 automáticamente al cambiar cualquier valor (si conectado)
- [x] 15.2 En Pulse: eliminar botón "Aplicar Config" redundante, restaurar como botón manual adicional
- [x] 15.3 Bloquear todos los controles de configuración con `add_enabled_ui(!running)` en todos los módulos
- [x] 15.4 Bloquear hotkeys también cuando el módulo está activo (incluidas en el bloque locked)
- [x] 15.5 Mostrar aviso `⚠ Detén el módulo para modificar la configuración.` cuando está corriendo
- [x] 15.6 Agregar botón "Aplicar Config" en todos los módulos (Pulse, KeepAlive, Monitor, Panic, Sequence)
- [x] 15.7 Extraer `send_pulse_config()` como método helper en `pulse_tab.rs`
