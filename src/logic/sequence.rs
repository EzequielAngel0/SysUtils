use crate::app::SysUtilsApp;
use crate::logic::hardware::HardwareLogic;
use crate::models::{LogLevel, MacroEvent};
use std::time::{Duration, Instant};

pub trait SequenceLogic {
    fn start_recording(&mut self);
    fn stop_recording(&mut self);
    fn play_sequence(&mut self);
    fn stop_playback(&mut self);
    fn save_sequence(&self);
    fn load_sequence(&mut self);
}

fn extract_key_char(key_debug: &str) -> String {
    let s = key_debug.trim();
    // Handle rdev Key debug format: "KeyA", "KeyB", etc.
    if s.starts_with("Key") && s.len() >= 4 {
        return s[3..].to_lowercase();
    }
    if s.starts_with("Num") {
        return s[3..].to_string();
    }
    match s {
        "Space" => " ".into(),
        "Return" => "enter".into(),
        "Tab" => "tab".into(),
        "Escape" => "esc".into(),
        "ShiftLeft" | "ShiftRight" => "shift".into(),
        "ControlLeft" | "ControlRight" => "ctrl".into(),
        "Alt" | "AltGr" => "alt".into(),
        "Backspace" => "backspace".into(),
        "CapsLock" => "capslock".into(),
        "UpArrow" => "up".into(),
        "DownArrow" => "down".into(),
        "LeftArrow" => "left".into(),
        "RightArrow" => "right".into(),
        _ => s.to_lowercase(),
    }
}

fn extract_mouse_btn(btn_debug: &str) -> &str {
    if btn_debug.contains("Right") { "R" }
    else if btn_debug.contains("Middle") { "M" }
    else { "L" }
}

impl SequenceLogic for SysUtilsApp {
    fn start_recording(&mut self) {
        self.sequence_recording = true;
        *self.sequence_recording_stop.lock().unwrap() = false;
        self.sequence_events.lock().unwrap().clear();

        let events = self.sequence_events.clone();
        let stop_flag = self.sequence_recording_stop.clone();
        let logs = self.logs.clone();
        let last_time = self.sequence_last_event_time.clone();

        *last_time.lock().unwrap() = None;
        logs.log(LogLevel::Action, "Sequence", "Grabación iniciada — presiona teclas/ratón para grabar");

        let raw_rx = self.hotkeys.raw_rx.clone();

        std::thread::spawn(move || {
            use rdev::EventType;

            let events_clone = events.clone();
            let stop_clone = stop_flag.clone();
            let last_time_clone = last_time.clone();
            // Track last mouse position for distance filtering
            let last_mouse_pos = std::sync::Arc::new(std::sync::Mutex::new((0i32, 0i32)));
            let lmp = last_mouse_pos.clone();

            while let Ok(event) = raw_rx.recv() {
                if *stop_clone.lock().unwrap() { break; }

                let mut evts = events_clone.lock().unwrap();
                let mut lt = last_time_clone.lock().unwrap();

                match event.event_type {
                    EventType::KeyPress(key) => {
                        // Record delay since last event
                        if let Some(prev) = *lt {
                            let elapsed = prev.elapsed().as_millis() as u64;
                            if elapsed > 5 {
                                evts.push(MacroEvent::Delay(elapsed));
                            }
                        }
                        *lt = Some(Instant::now());
                        evts.push(MacroEvent::KeyDown(format!("{:?}", key)));
                    }
                    EventType::KeyRelease(key) => {
                        if let Some(prev) = *lt {
                            let elapsed = prev.elapsed().as_millis() as u64;
                            if elapsed > 5 {
                                evts.push(MacroEvent::Delay(elapsed));
                            }
                        }
                        *lt = Some(Instant::now());
                        evts.push(MacroEvent::KeyUp(format!("{:?}", key)));
                    }
                    EventType::ButtonPress(btn) => {
                        if let Some(prev) = *lt {
                            let elapsed = prev.elapsed().as_millis() as u64;
                            if elapsed > 5 {
                                evts.push(MacroEvent::Delay(elapsed));
                            }
                        }
                        *lt = Some(Instant::now());
                        evts.push(MacroEvent::MouseDown(format!("{:?}", btn)));
                    }
                    EventType::ButtonRelease(btn) => {
                        if let Some(prev) = *lt {
                            let elapsed = prev.elapsed().as_millis() as u64;
                            if elapsed > 5 {
                                evts.push(MacroEvent::Delay(elapsed));
                            }
                        }
                        *lt = Some(Instant::now());
                        evts.push(MacroEvent::MouseUp(format!("{:?}", btn)));
                    }
                    EventType::MouseMove { x, y } => {
                        let (ix, iy) = (x as i32, y as i32);
                        let mut last_pos = lmp.lock().unwrap();

                        // Only record moves with >10px distance
                        let dx = (ix - last_pos.0).abs();
                        let dy = (iy - last_pos.1).abs();
                        if dx < 10 && dy < 10 {
                            continue; // Skip small movements
                        }
                        *last_pos = (ix, iy);
                        drop(last_pos);

                        // Coalesce: remove previous MouseMove AND its Delay
                        if evts.len() >= 2 {
                            if matches!(evts.last(), Some(MacroEvent::MouseMove(_, _))) {
                                evts.pop(); // Remove old MouseMove
                                if matches!(evts.last(), Some(MacroEvent::Delay(_))) {
                                    evts.pop(); // Remove preceding Delay too
                                }
                            }
                        } else if evts.len() == 1 {
                            if matches!(evts.last(), Some(MacroEvent::MouseMove(_, _))) {
                                evts.pop();
                            }
                        }

                        // Record delay for the new position
                        if let Some(prev) = *lt {
                            let elapsed = prev.elapsed().as_millis() as u64;
                            if elapsed > 5 {
                                evts.push(MacroEvent::Delay(elapsed));
                            }
                        }
                        *lt = Some(Instant::now());
                        evts.push(MacroEvent::MouseMove(ix, iy));
                    }
                    _ => {}
                }
            }
        });
    }

    fn stop_recording(&mut self) {
        *self.sequence_recording_stop.lock().unwrap() = true;
        self.sequence_recording = false;
        let count = self.sequence_events.lock().unwrap().len();
        self.logs.log(LogLevel::Action, "Sequence", &format!("Grabación finalizada: {} eventos", count));
        self.set_status(&format!("Secuencia grabada: {} eventos", count));
    }

    fn play_sequence(&mut self) {
        let loops: u32 = self.sequence_loops.parse().unwrap_or(1);
        let events = self.sequence_events.lock().unwrap().clone();

        if events.is_empty() {
            self.set_status("⚠ No hay eventos grabados");
            return;
        }
        if !self.hw.is_connected() {
            self.set_status("⚠ Hardware no conectado");
            return;
        }

        self.sequence_playing = true;
        *self.sequence_play_stop.lock().unwrap() = false;

        let hw = self.hw.clone();
        let stop_flag = self.sequence_play_stop.clone();
        let logs = self.logs.clone();
        let loop_counter = self.sequence_loop_counter.clone();

        logs.log(LogLevel::Action, "Sequence", &format!("Reproduciendo {} eventos x{} veces", events.len(), loops));

        std::thread::spawn(move || {
            let actual_loops = if loops == 0 { u32::MAX } else { loops };
            *loop_counter.lock().unwrap() = 0;

            for lap in 0..actual_loops {
                if *stop_flag.lock().unwrap() { break; }
                *loop_counter.lock().unwrap() = lap + 1;
                logs.log(LogLevel::Info, "Sequence", &format!("Loop {}/{}", lap + 1, if loops == 0 { "∞".to_string() } else { loops.to_string() }));

                let mut last_x = 0i32;
                let mut last_y = 0i32;
                let mut first_move = true;

                for evt in &events {
                    if *stop_flag.lock().unwrap() { break; }

                    match evt {
                        MacroEvent::KeyDown(k) => {
                            let ch = extract_key_char(k);
                            let _ = hw.send(&format!("KEY_DOWN:{}", ch));
                        }
                        MacroEvent::KeyUp(k) => {
                            let ch = extract_key_char(k);
                            let _ = hw.send(&format!("KEY_UP:{}", ch));
                        }
                        MacroEvent::MouseDown(b) => {
                            let btn = extract_mouse_btn(b);
                            let _ = hw.send(&format!("CLK_DOWN:{}", btn));
                        }
                        MacroEvent::MouseUp(b) => {
                            let btn = extract_mouse_btn(b);
                            let _ = hw.send(&format!("CLK_UP:{}", btn));
                        }
                        MacroEvent::MouseMove(x, y) => {
                            if first_move {
                                last_x = *x;
                                last_y = *y;
                                first_move = false;
                                let _ = hw.send("MOUSE_MOVE_REL:0:0");
                            } else {
                                let dx = x - last_x;
                                let dy = y - last_y;
                                let _ = hw.send(&format!("MOUSE_MOVE_REL:{}:{}", dx, dy));
                                last_x = *x;
                                last_y = *y;
                            }
                        }
                        MacroEvent::Delay(ms) => {
                            std::thread::sleep(Duration::from_millis(*ms));
                        }
                    }
                }
            }
            logs.log(LogLevel::Action, "Sequence", "Reproducción finalizada");
        });
    }

    fn stop_playback(&mut self) {
        *self.sequence_play_stop.lock().unwrap() = true;
        self.sequence_playing = false;
        self.logs.log(LogLevel::Action, "Sequence", "Reproducción detenida");
    }

    fn save_sequence(&self) {
        let events = self.sequence_events.lock().unwrap().clone();
        let path = {
            let mut p = std::env::current_exe().unwrap_or_default();
            p.pop();
            p.push("sysutils_sequence.json");
            p
        };
        if let Ok(json) = serde_json::to_string_pretty(&events) {
            if let Ok(()) = std::fs::write(&path, json) {
                self.logs.log(LogLevel::Action, "Sequence", &format!("Guardado en {}", path.display()));
            }
        }
    }

    fn load_sequence(&mut self) {
        let path = {
            let mut p = std::env::current_exe().unwrap_or_default();
            p.pop();
            p.push("sysutils_sequence.json");
            p
        };
        if path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(events) = serde_json::from_str::<Vec<MacroEvent>>(&contents) {
                    let count = events.len();
                    *self.sequence_events.lock().unwrap() = events;
                    self.logs.log(LogLevel::Action, "Sequence", &format!("Cargado: {} eventos", count));
                    self.set_status(&format!("Secuencia cargada: {} eventos", count));
                }
            }
        } else {
            self.set_status("⚠ No se encontró archivo de secuencia");
        }
    }
}
