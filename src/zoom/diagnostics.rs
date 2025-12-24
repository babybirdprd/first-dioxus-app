use serde::{Deserialize, Serialize};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// A single frame of camera telemetry for diagnosis
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TelemetryFrame {
    pub frame_index: usize,
    pub time_secs: f32,
    pub zoom: f32,
    pub cx: f32,
    pub cy: f32,
    pub target_cx: f32,
    pub target_cy: f32,
    pub mouse_cx: f32,
    pub mouse_cy: f32,
    pub velocity_cx: f32,
    pub velocity_cy: f32,
}

/// A complete telemetry session for a video export
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TelemetrySession {
    pub input_path: String,
    pub output_path: String,
    pub frames: Vec<TelemetryFrame>,
}

/// Get the central log directory
pub fn get_log_dir() -> std::path::PathBuf {
    dirs::video_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default())
        .join("DemoRecorder")
        .join("logs")
}

/// Initialize tracing with both console and file output
pub fn init_diagnostics() -> WorkerGuard {
    let log_dir = get_log_dir();

    if !log_dir.exists() {
        let _ = std::fs::create_dir_all(&log_dir);
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let log_filename = format!("session_{}.log", timestamp);

    tracing::info!(
        "Diagnostics initialized. Logs: {:?}/{}",
        log_dir,
        log_filename
    );

    // file_appender takes ownership of log_dir, so we do it after the info! call
    let file_appender = tracing_appender::rolling::never(log_dir, log_filename);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
                .add_directive("demo_recorder=debug".parse().unwrap()) // Enable debug for our code
                .add_directive("wgpu=warn".parse().unwrap()) // Silence WGPU
                .add_directive("wgpu_hal=warn".parse().unwrap()) // Silence WGPU HAL
                .add_directive("video_rs=warn".parse().unwrap()), // Silence Video-RS
        )
        .with(fmt::layer().with_writer(std::io::stdout)) // Console
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false)) // File
        .init();

    guard
}

/// Save telemetry session to JSON
pub fn save_telemetry(
    session: &TelemetrySession,
    path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(session)?;
    std::fs::write(path, json)?;
    tracing::info!("Telemetry saved to: {:?}", path);

    // Perform immediate analysis
    analyze_motion_health(session);
    Ok(())
}

fn analyze_motion_health(session: &TelemetrySession) {
    let mut velocity_spikes = 0;
    let mut jitter_events = 0;

    // Thresholds: 0.015 units per frame is roughly 5% screen width per frame @ 60fps
    let velocity_threshold = 0.015;

    for i in 1..session.frames.len() {
        let f = &session.frames[i];
        let prev = &session.frames[i - 1];

        let vel_x = (f.cx - prev.cx).abs();
        let vel_y = (f.cy - prev.cy).abs();

        if vel_x > velocity_threshold || vel_y > velocity_threshold {
            velocity_spikes += 1;
        }

        // Jitter: check for sign changes in velocity (oscillation)
        if i > 2 {
            let prev_prev = &session.frames[i - 2];
            let v1_x = f.cx - prev.cx;
            let v2_x = prev.cx - prev_prev.cx;

            if v1_x * v2_x < -0.0001 {
                // Velocity flipped direction
                jitter_events += 1;
            }
        }
    }

    if velocity_spikes > 0 || jitter_events > 10 {
        tracing::warn!(
            "MOTION HEALTH WARNING: Detected {} velocity spikes and {} jitter events. Video may feel clunky.",
            velocity_spikes, jitter_events
        );
    } else {
        tracing::info!(
            "Motion health looks GOOD. {} frames analyzed.",
            session.frames.len()
        );
    }
}
