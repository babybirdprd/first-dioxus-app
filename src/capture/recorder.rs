//! Screen recording using windows-capture crate
//!
//! Uses the Windows Graphics Capture API for high-performance screen capture
//! with built-in video encoding.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[cfg(windows)]
use windows_capture::{
    capture::{Context, GraphicsCaptureApiHandler},
    encoder::{
        AudioSettingsBuilder, ContainerSettingsBuilder, VideoEncoder, VideoSettingsBuilder,
        VideoSettingsSubType,
    },
    frame::Frame,
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor,
    settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    },
};

/// Global recording state
static IS_RECORDING: AtomicBool = AtomicBool::new(false);

/// Check if currently recording
pub fn is_recording() -> bool {
    IS_RECORDING.load(Ordering::SeqCst)
}

/// Set recording state
pub fn set_recording(state: bool) {
    IS_RECORDING.store(state, Ordering::SeqCst);
}

/// Recorder configuration
#[derive(Clone, Debug)]
pub struct RecorderConfig {
    pub output_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

impl Default for RecorderConfig {
    fn default() -> Self {
        let output_dir = dirs::video_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default())
            .join("DemoRecorder");

        Self {
            output_path: output_dir.join("recording.mp4"),
            width: 1920,
            height: 1080,
            fps: 30,
        }
    }
}

#[cfg(windows)]
pub mod windows_impl {
    use super::*;

    /// Capture handler that encodes frames to video
    pub struct CaptureHandler {
        encoder: Option<VideoEncoder>,
        start: Instant,
        max_duration_secs: u64,
    }

    impl GraphicsCaptureApiHandler for CaptureHandler {
        type Flags = RecorderConfig;
        type Error = Box<dyn std::error::Error + Send + Sync>;

        fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
            let config = ctx.flags;

            // Create output directory
            if let Some(parent) = config.output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let encoder = VideoEncoder::new(
                VideoSettingsBuilder::new(config.width, config.height)
                    .sub_type(VideoSettingsSubType::H264) // H.264 for WMP compatibility
                    .frame_rate(config.fps),
                AudioSettingsBuilder::default().disabled(true),
                ContainerSettingsBuilder::default(),
                &config.output_path,
            )?;

            println!("Recording to: {:?}", config.output_path);

            Ok(Self {
                encoder: Some(encoder),
                start: Instant::now(),
                max_duration_secs: 300, // 5 minutes max
            })
        }

        fn on_frame_arrived(
            &mut self,
            frame: &mut Frame,
            capture_control: InternalCaptureControl,
        ) -> Result<(), Self::Error> {
            // Check if we should stop
            if !is_recording() || self.start.elapsed().as_secs() >= self.max_duration_secs {
                if let Some(encoder) = self.encoder.take() {
                    encoder.finish()?;
                    println!("\nRecording saved!");
                }
                capture_control.stop();
                set_recording(false);
                return Ok(());
            }

            // Send frame to encoder
            if let Some(ref mut encoder) = self.encoder {
                encoder.send_frame(frame)?;
            }

            // Print progress every second
            let elapsed = self.start.elapsed().as_secs();
            if elapsed > 0 && self.start.elapsed().subsec_millis() < 50 {
                print!("\rRecording: {}s", elapsed);
                std::io::Write::flush(&mut std::io::stdout())?;
            }

            Ok(())
        }

        fn on_closed(&mut self) -> Result<(), Self::Error> {
            println!("\nCapture session ended");
            if let Some(encoder) = self.encoder.take() {
                encoder.finish()?;
            }
            set_recording(false);
            Ok(())
        }
    }

    /// Start recording the primary monitor
    pub fn start_recording(
        config: RecorderConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        set_recording(true);

        let primary_monitor = Monitor::primary()?;

        let settings = Settings::new(
            primary_monitor,
            CursorCaptureSettings::Default,
            DrawBorderSettings::WithoutBorder,
            SecondaryWindowSettings::Default,
            MinimumUpdateIntervalSettings::Default,
            DirtyRegionSettings::Default,
            ColorFormat::Rgba8,
            config,
        );

        // Start capture in a new thread
        std::thread::spawn(move || {
            if let Err(e) = CaptureHandler::start(settings) {
                eprintln!("Capture error: {e}");
                set_recording(false);
            }
        });

        Ok(())
    }

    /// Stop the current recording
    pub fn stop_recording() {
        set_recording(false);
    }
}

#[cfg(windows)]
pub use windows_impl::{start_recording, stop_recording};

// macOS stub
#[cfg(target_os = "macos")]
pub fn start_recording(
    _config: RecorderConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("macOS recording not yet implemented");
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn stop_recording() {
    println!("Stopping macOS recording");
}
