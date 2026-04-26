// ═══════════════════════════════════════════════════════════════════════════════
// file_logger.rs — File-based logging with daily rotation using tracing
// ═══════════════════════════════════════════════════════════════════════════════

use std::path::PathBuf;
use tracing_appender::rolling;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

/// Initialize file logging to `sysutils_logs/` next to the executable.
/// Logs rotate daily. Returns a guard that must be kept alive.
pub fn init_file_logging() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let log_dir = log_directory();
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Failed to create log directory: {}", e);
        return None;
    }

    let file_appender = rolling::daily(&log_dir, "sysutils.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(false)
                .with_timer(fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string()))
        )
        .init();

    Some(guard)
}

/// Get the log directory path (next to the executable)
pub fn log_directory() -> PathBuf {
    let mut path = std::env::current_exe().unwrap_or_default();
    path.pop();
    path.push("sysutils_logs");
    path
}

/// Write a message to the file logger (if active)
pub fn file_log(level: &str, module: &str, message: &str) {
    match level {
        "error" => tracing::error!("[{}] {}", module, message),
        "warn"  => tracing::warn!("[{}] {}", module, message),
        "info"  => tracing::info!("[{}] {}", module, message),
        _       => tracing::debug!("[{}] {}", module, message),
    }
}
