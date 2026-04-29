// ═══════════════════════════════════════════════════════════════════════════════
// stealth.rs — Stealth mode: fake window title and process name
// ═══════════════════════════════════════════════════════════════════════════════

/// Configuration for stealth mode behaviour.
pub struct StealthConfig {
    pub enabled: bool,
    pub fake_window_title: String,
    pub fake_process_name: String,
}

/// Stealth mode operations.
pub struct StealthMode;

impl StealthMode {
    /// Update the egui viewport title.
    pub fn apply_window_title(ctx: &egui::Context, title: &str) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.to_string()));
    }

    /// Rename the current process (Linux only via `prctl PR_SET_NAME`).
    #[cfg(target_os = "linux")]
    pub fn apply_process_name(name: &str) {
        use std::ffi::CString;
        let truncated = if name.len() > 15 { &name[..15] } else { name };
        if let Ok(cname) = CString::new(truncated) {
            let ret = unsafe {
                libc::prctl(
                    libc::PR_SET_NAME,
                    cname.as_ptr() as libc::c_ulong,
                    0,
                    0,
                    0,
                )
            };
            if ret != 0 {
                eprintln!("[stealth] prctl PR_SET_NAME falló (ret={})", ret);
            }
        }
    }

    /// No-op on Windows and other platforms.
    /// Renaming the process on Windows requires techniques outside the scope of this sprint.
    #[cfg(not(target_os = "linux"))]
    pub fn apply_process_name(_name: &str) {
        // No-op en Windows: renombrar el proceso requiere técnicas fuera del scope de este sprint
    }

    /// Apply (or restore) stealth mode based on `config`.
    ///
    /// - If `config.enabled` is `false`: restores the default "SysUtils" title and returns.
    /// - If `config.enabled` is `true`: applies the fake window title and process name.
    pub fn apply(config: &StealthConfig, ctx: &egui::Context) {
        if !config.enabled {
            Self::apply_window_title(ctx, "SysUtils");
            return;
        }

        Self::apply_window_title(ctx, &config.fake_window_title);
        Self::apply_process_name(&config.fake_process_name);
    }
}
