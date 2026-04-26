// -------------------------------------------------------------------------------
// hotkey_engine.rs - Global hotkey listener using raw OS hooks
// Supports direct matching of rdev event strings (Key or Mouse Button)
// -------------------------------------------------------------------------------

use crossbeam_channel::{Receiver, Sender};
use rdev::{listen, Event, EventType};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Events produced by the hotkey engine
#[derive(Debug, Clone)]
pub enum HotkeyEvent {
    Triggered(String),
    #[allow(dead_code)]
    Released(String),
}

/// A registered hotkey binding
#[derive(Debug, Clone)]
pub struct HotkeyBinding {
    pub id: String,
    pub trigger: String,
}

/// The engine that listens for global key events
pub struct HotkeyEngine {
    bindings: Arc<Mutex<Vec<HotkeyBinding>>>,
    pressed_keys: Arc<Mutex<HashSet<String>>>,
    pub event_rx: Receiver<HotkeyEvent>,
    event_tx: Sender<HotkeyEvent>,
    pub raw_rx: Receiver<rdev::Event>,
    raw_tx: Sender<rdev::Event>,
}

impl HotkeyEngine {
    pub fn new() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        let (raw_tx, raw_rx) = crossbeam_channel::unbounded();
        Self {
            bindings: Arc::new(Mutex::new(Vec::new())),
            pressed_keys: Arc::new(Mutex::new(HashSet::new())),
            event_rx: rx,
            event_tx: tx,
            raw_rx,
            raw_tx,
        }
    }

    /// Register a new hotkey binding
    pub fn register(&self, id: &str, trigger: &str) {
        let binding = HotkeyBinding {
            id: id.to_string(),
            trigger: trigger.to_string(),
        };
        self.bindings.lock().unwrap().push(binding);
    }

    /// Clear all bindings
    pub fn clear_bindings(&self) {
        self.bindings.lock().unwrap().clear();
    }

    /// Check if a trigger string matches the current pressed
    fn check_triggers(tx: &Sender<HotkeyEvent>, bindings: &Arc<Mutex<Vec<HotkeyBinding>>>, current_id: &str) {
        let bindings_lock = bindings.lock().unwrap();
        for binding in bindings_lock.iter() {
            if binding.trigger == current_id {
                let _ = tx.send(HotkeyEvent::Triggered(binding.id.clone()));
            }
        }
    }

    /// Start the listener in a background thread (call once)
    pub fn start(&self) {
        let bindings = self.bindings.clone();
        let pressed = self.pressed_keys.clone();
        let tx = self.event_tx.clone();
        let raw_tx = self.raw_tx.clone();

        std::thread::spawn(move || {
            let callback = move |event: Event| {
                let _ = raw_tx.send(event.clone());

                match event.event_type {
                    EventType::KeyPress(key) => {
                        let mut pressed_lock = pressed.lock().unwrap();
                        let key_id = format!("{:?}", key);

                        if pressed_lock.contains(&key_id) {
                            return;
                        }
                        pressed_lock.insert(key_id.clone());
                        drop(pressed_lock);

                        Self::check_triggers(&tx, &bindings, &key_id);
                    }
                    EventType::KeyRelease(key) => {
                        let mut pressed_lock = pressed.lock().unwrap();
                        pressed_lock.remove(&format!("{:?}", key));
                    }
                    EventType::ButtonPress(btn) => {
                        let mut pressed_lock = pressed.lock().unwrap();
                        let btn_id = format!("{:?}", btn);

                        if pressed_lock.contains(&btn_id) {
                            return;
                        }
                        pressed_lock.insert(btn_id.clone());
                        drop(pressed_lock);

                        Self::check_triggers(&tx, &bindings, &btn_id);
                    }
                    EventType::ButtonRelease(btn) => {
                        let mut pressed_lock = pressed.lock().unwrap();
                        pressed_lock.remove(&format!("{:?}", btn));
                    }
                    _ => {}
                }
            };

            if let Err(e) = listen(callback) {
                log::error!("Hotkey listener crashed: {:?}", e);
            }
        });
    }
}
