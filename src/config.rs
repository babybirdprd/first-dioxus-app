//! Application configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Output format for recordings
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum OutputFormat {
    #[default]
    Mp4,
    WebM,
    Gif,
}

/// Audio recording mode
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum AudioMode {
    #[default]
    None,
    System,
    Microphone,
    Both,
}

/// Zoom behavior mode
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum ZoomMode {
    #[default]
    None,
    FollowCursor,
    ClickToZoom,
    SmartAI,
}

/// What to capture
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum CaptureTarget {
    #[default]
    PrimaryMonitor,
    ForegroundWindow,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub hotkey: String,
    pub output_format: OutputFormat,
    pub audio_mode: AudioMode,
    pub zoom_mode: ZoomMode,
    pub zoom_level: f32,
    pub output_folder: PathBuf,
    pub fps: u32,
    pub show_countdown: bool,
    pub countdown_seconds: u32,
    pub capture_target: CaptureTarget,
}

impl Default for Config {
    fn default() -> Self {
        let output_folder = dirs::video_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default())
            .join("DemoRecorder");

        Self {
            hotkey: "Ctrl+Shift+F9".to_string(),
            output_format: OutputFormat::default(),
            audio_mode: AudioMode::default(),
            zoom_mode: ZoomMode::None,
            zoom_level: 1.5,
            output_folder,
            fps: 30,
            show_countdown: true,
            countdown_seconds: 3,
            capture_target: CaptureTarget::default(),
        }
    }
}

impl Config {
    /// Load config from file or create default
    pub fn load() -> Self {
        let config_path = Self::config_path();

        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(contents) => match serde_json::from_str(&contents) {
                    Ok(config) => return config,
                    Err(e) => eprintln!("Failed to parse config: {e}"),
                },
                Err(e) => eprintln!("Failed to read config: {e}"),
            }
        }

        Self::default()
    }

    /// Save config to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path();

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, contents)?;

        Ok(())
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default())
            .join("DemoRecorder")
            .join("config.json")
    }
}
