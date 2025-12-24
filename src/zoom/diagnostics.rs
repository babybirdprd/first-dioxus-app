use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize tracing with both console and file output
pub fn init_diagnostics() -> WorkerGuard {
    let log_dir = dirs::video_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default())
        .join("DemoRecorder")
        .join("logs");

    if !log_dir.exists() {
        let _ = std::fs::create_dir_all(&log_dir);
    }

    tracing::info!("Diagnostics initialized. Logs: {:?}", log_dir);

    // file_appender takes ownership of log_dir, so we do it after the info! call
    let file_appender = tracing_appender::rolling::never(log_dir, "zoom_diagnosis.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with(fmt::layer().with_writer(std::io::stdout)) // Console output
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false)) // File output
        .init();

    guard
}
