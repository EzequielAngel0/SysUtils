# CLAUDE.md — Phantom Native Development Guide

## Project Overview
Phantom Native is a hardware automation suite built in Rust with egui/eframe.
It communicates with an ESP32-S3 via serial to perform HID input emulation.

## Architecture
```
src/
├── main.rs          # Entry point + egui App impl + sidebar UI
├── app.rs           # PhantomApp struct (central state) + hotkey management + auto-save
├── config.rs        # AppConfig (persistent JSON config)
├── file_logger.rs   # Persistent file logging using tracing-appender
├── hotkey_engine.rs # Global OS-level hotkey listener (rdev) + Modifier support
├── hw_link.rs       # Serial communication with ESP32
├── screen_capture.rs# Screen/window capture (xcap + image)
├── system_info.rs   # CPU/RAM usage + Anti-cheat process scanning
├── models.rs        # Shared types: LogBuffer, MacroEvent, Tab, LogLevel
├── logic/
│   ├── hardware.rs  # Connect/disconnect/refresh ports
│   ├── pulse.rs     # Auto-clicker/Keyboard key toggle
│   ├── keepalive.rs # Anti-AFK periodic key press
│   ├── monitor.rs   # Screen change detection & Color checking
│   ├── panic.rs     # Emergency stop on screen change
│   └── sequence.rs  # Record/playback macro sequences + Noise filtering
└── ui/
    ├── pulse_tab.rs
    ├── keepalive_tab.rs
    ├── monitor_tab.rs
    ├── panic_tab.rs
    ├── sequence_tab.rs
    └── logs_tab.rs
```

## Key Patterns
- **Trait-per-module:** Each logic module is a trait implemented on PhantomApp
- **Arc<Mutex<T>>:** All shared state between UI thread and background threads
- **HwLink:** Thread-safe serial bridge, sends string commands like "CLK_DOWN:L"
- **Config persistence:** `phantom_config.json` next to the executable, with Dirty flagging & Auto-Save tick
- **Background Monitoreo:** CPU/RAM and Anti-Cheat scanning happens every 2 seconds on a background tick

## ESP32 Command Protocol
```
START / STOP           — Toggle pulse
DELAY:min:max          — Set random delay range
TARGET_BTN:L|R|M       — Set mouse button target
MODE:PULSE|HOLD        — Set click mode
CLK_DOWN:L|R|M         — Press mouse button
CLK_UP:L|R|M           — Release mouse button
KEY_DOWN:char          — Press keyboard key
KEY_UP:char            — Release keyboard key
PING                   — Connection check
```

## Build Commands
```powershell
cargo build --release          # Optimized build
cargo run --release            # Run in release mode
./run.ps1                      # Shortcut script
./build_release.ps1            # Full optimized build (target-cpu=native)
./build_portable.ps1           # Create portable ZIP
```

## Completed Implementation Sprint (Roadmap v1.1)
✓ **Auto-save config** — Saves automatically after 5s if modified
✓ **Monitor conditions** — color_appear / color_disappear / change modes added
✓ **Export/Import configs** — Built-in JSON load/save support
✓ **System Overview** — CPU/RAM and Anti-cheat warnings present in the Right Sidebar
✓ **Hotkey modifiers** — Engine updated to support Ctrl+, Alt+, Shift+ and alphanumeric combos
✓ **Release scripts** — `build_release.ps1` and `build_portable.ps1` prepared
✓ **File logging** — `phantom_logs/` folder handles daily persistent traces
✓ **Pulse keyboard keys** — Allow "e", "f", Space instead of only Mouse clicks
✓ **Sequence Fixes** — Reduced mouse-move spam (10px Euclidean filter) and enabled Keyboard interception natively.

## Upcoming Roadmap (Phase 2) 🚀
- Multi-Monitor Mode (Independent isolated regions checking for different colors simultaneously).
- AI/Computer Vision advanced overlays.
- Further Sequence Editor refinements (Drag-and-Drop events reordering).
