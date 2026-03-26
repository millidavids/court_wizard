use std::fmt::Write as _;
use std::io::Write;
use std::panic;
use std::path::PathBuf;

/// Installs a custom panic hook that writes crash details to a file.
///
/// The crash log is written to the platform data directory
/// (`%APPDATA%/court_wizard/crash.log` on Windows,
/// `~/.local/share/court_wizard/crash.log` on Linux).
///
/// Each crash overwrites the previous log so only the most recent is kept.
/// Must be called before `App::new()` so it catches panics during setup too.
pub fn install() {
    // Resolve the crash log path once at startup so we don't do env/fs lookups
    // inside the panic hook (process state may be compromised).
    let crash_log_path = crate::config::storage::save_dir()
        .ok()
        .map(|dir| dir.join("crash.log"));

    panic::set_hook(Box::new(move |info| {
        let crash_report = build_crash_report(info);

        // Print to stderr so it's visible in the terminal
        eprintln!("{crash_report}");

        // Write to file — best-effort since we're already crashing
        if let Some(path) = &crash_log_path {
            match write_crash_log(path, &crash_report) {
                Ok(()) => eprintln!("Crash log written to: {}", path.display()),
                Err(e) => eprintln!("Failed to write crash log to {}: {e}", path.display()),
            }
        }
    }));
}

fn write_crash_log(path: &PathBuf, report: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut file = std::fs::File::create(path)?;
    file.write_all(report.as_bytes())
}

fn build_crash_report(info: &panic::PanicHookInfo<'_>) -> String {
    let mut report = String::new();

    // Header
    let _ = writeln!(report, "=== COURT WIZARD CRASH REPORT ===");
    let _ = writeln!(report);
    let _ = writeln!(report, "Version: v{}", env!("CARGO_PKG_VERSION"));

    // Timestamp
    if let Ok(elapsed) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        let _ = writeln!(report, "Timestamp: {} (unix)", elapsed.as_secs());
    }

    // OS info
    let _ = writeln!(report, "OS: {}", std::env::consts::OS);
    let _ = writeln!(report, "Arch: {}", std::env::consts::ARCH);
    let _ = writeln!(report);

    // Panic message
    let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
        *s
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.as_str()
    } else {
        "Unknown panic payload"
    };
    let _ = writeln!(report, "Panic: {message}");

    // Location
    if let Some(location) = info.location() {
        let _ = writeln!(
            report,
            "Location: {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }

    // Backtrace
    let _ = writeln!(report, "\n--- Backtrace ---");
    let backtrace = std::backtrace::Backtrace::force_capture();
    let _ = write!(report, "{backtrace}");
    let _ = writeln!(report, "\n=== END CRASH REPORT ===");

    report
}
