// ═══════════════════════════════════════════════════════════════════════════════
// hw_link.rs — Serial communication layer with the ESP32-S3
// Zero-overhead, thread-safe hardware bridge
// ═══════════════════════════════════════════════════════════════════════════════

use crossbeam_channel::{Receiver, Sender};
use serialport::{self, SerialPort};
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Represents the connection state to the ESP32
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connected(String), // port name
    #[allow(dead_code)]
    Error(String),
}

/// Thread-safe hardware link to the ESP32-S3
pub struct HwLink {
    port: Arc<Mutex<Option<Box<dyn SerialPort>>>>,
    state: Arc<Mutex<ConnectionState>>,
    reader_tx: Sender<String>,
    #[allow(dead_code)]
    pub reader_rx: Receiver<String>,
}

impl HwLink {
    pub fn new() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        Self {
            port: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(ConnectionState::Disconnected)),
            reader_tx: tx,
            reader_rx: rx,
        }
    }

    /// List all available COM ports
    pub fn available_ports() -> Vec<String> {
        serialport::available_ports()
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.port_name)
            .collect()
    }

    /// Connect to the ESP32 on the given port
    pub fn connect(&self, port_name: &str) -> Result<(), String> {
        // Disconnect if already connected
        self.disconnect();

        let port = serialport::new(port_name, 115_200)
            .timeout(Duration::from_millis(100))
            .open()
            .map_err(|e| format!("Failed to open {}: {}", port_name, e))?;

        // Clone port for the reader thread
        let reader_port = port
            .try_clone()
            .map_err(|e| format!("Failed to clone port: {}", e))?;

        *self.port.lock().unwrap() = Some(port);
        *self.state.lock().unwrap() = ConnectionState::Connected(port_name.to_string());

        // Spawn a background thread to read ACKs/responses from the ESP32
        let tx = self.reader_tx.clone();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(reader_port);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => break,              // Port closed
                    Ok(_) => {
                        let trimmed = line.trim().to_string();
                        if !trimmed.is_empty() {
                            let _ = tx.send(trimmed);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                    Err(_) => break,
                }
            }
        });

        // Send a PING to verify connection
        self.send("PING")?;

        Ok(())
    }

    /// Disconnect from the ESP32
    pub fn disconnect(&self) {
        let mut port_lock = self.port.lock().unwrap();
        if port_lock.is_some() {
            // Attempt to send STOP before disconnecting
            if let Some(ref mut p) = *port_lock {
                let _ = p.write_all(b"STOP\n");
                let _ = p.flush();
            }
        }
        *port_lock = None;
        *self.state.lock().unwrap() = ConnectionState::Disconnected;
    }

    /// Send a command string to the ESP32 (appends newline)
    pub fn send(&self, cmd: &str) -> Result<(), String> {
        let mut port_lock = self.port.lock().unwrap();
        match port_lock.as_mut() {
            Some(p) => {
                let data = format!("{}\n", cmd);
                p.write_all(data.as_bytes())
                    .map_err(|e| format!("Write failed: {}", e))?;
                p.flush().map_err(|e| format!("Flush failed: {}", e))?;
                Ok(())
            }
            None => Err("Not connected".into()),
        }
    }

    /// Get the current connection state
    pub fn state(&self) -> ConnectionState {
        self.state.lock().unwrap().clone()
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        matches!(self.state(), ConnectionState::Connected(_))
    }
}
