//! Post-processing for zoom effects using ffmpeg
//!
//! Generates ffmpeg commands to apply zoom/pan effects based on recorded events

use super::event_log::RecordedEvent;
use std::path::Path;

/// Configuration for post-processing
#[derive(Clone, Debug)]
pub struct PostProcessConfig {
    /// Input video path
    pub input_path: String,
    /// Output video path  
    pub output_path: String,
    /// Video width
    pub width: u32,
    /// Video height
    pub height: u32,
    /// Frame rate
    pub fps: u32,
    /// Zoom level to apply on clicks (e.g., 1.5 = 150%)
    pub zoom_level: f32,
    /// Duration of zoom in/out animation in seconds
    pub zoom_duration: f32,
    /// How long to hold the zoom before zooming out (seconds)
    pub hold_duration: f32,
}

impl Default for PostProcessConfig {
    fn default() -> Self {
        Self {
            input_path: String::new(),
            output_path: String::new(),
            width: 1920,
            height: 1080,
            fps: 30,
            zoom_level: 1.5,
            zoom_duration: 0.3,
            hold_duration: 2.0,
        }
    }
}

/// Represents a zoom keyframe for ffmpeg
#[derive(Clone, Debug)]
pub struct ZoomKeyframe {
    /// Start time in seconds
    pub start_time: f32,
    /// End time in seconds  
    pub end_time: f32,
    /// Center X position (0-1 normalized)
    pub center_x: f32,
    /// Center Y position (0-1 normalized)
    pub center_y: f32,
    /// Zoom level
    pub zoom: f32,
}

/// Generate zoom keyframes from recorded events
pub fn generate_keyframes(
    events: &[RecordedEvent],
    config: &PostProcessConfig,
) -> Vec<ZoomKeyframe> {
    let mut keyframes = Vec::new();

    for event in events {
        if let RecordedEvent::Click { x, y, timestamp_ms } = event {
            let start_time = *timestamp_ms as f32 / 1000.0;
            let end_time = start_time + config.hold_duration + config.zoom_duration * 2.0;

            // Normalize coordinates to 0-1 range
            let center_x = *x as f32 / config.width as f32;
            let center_y = *y as f32 / config.height as f32;

            keyframes.push(ZoomKeyframe {
                start_time,
                end_time,
                center_x: center_x.clamp(0.0, 1.0),
                center_y: center_y.clamp(0.0, 1.0),
                zoom: config.zoom_level,
            });
        }
    }

    // Merge overlapping keyframes
    merge_overlapping_keyframes(&mut keyframes);

    keyframes
}

/// Merge overlapping zoom keyframes
fn merge_overlapping_keyframes(keyframes: &mut Vec<ZoomKeyframe>) {
    if keyframes.len() < 2 {
        return;
    }

    keyframes.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());

    let mut i = 0;
    while i < keyframes.len() - 1 {
        if keyframes[i].end_time > keyframes[i + 1].start_time {
            // Extend the first keyframe to cover both
            keyframes[i].end_time = keyframes[i + 1].end_time;
            // Update center to the newer click
            keyframes[i].center_x = keyframes[i + 1].center_x;
            keyframes[i].center_y = keyframes[i + 1].center_y;
            keyframes.remove(i + 1);
        } else {
            i += 1;
        }
    }
}

/// Generate ffmpeg filter complex for zoompan effect
pub fn generate_ffmpeg_filter(keyframes: &[ZoomKeyframe], config: &PostProcessConfig) -> String {
    if keyframes.is_empty() {
        return String::new();
    }

    // Build the zoom expression
    // This creates a conditional zoom based on time
    let mut zoom_expr = String::from("'");
    let mut x_expr = String::from("'");
    let mut y_expr = String::from("'");

    for (i, kf) in keyframes.iter().enumerate() {
        let start_frame = (kf.start_time * config.fps as f32) as u32;
        let zoom_in_end = start_frame + (config.zoom_duration * config.fps as f32) as u32;
        let hold_end = zoom_in_end + (config.hold_duration * config.fps as f32) as u32;
        let end_frame = (kf.end_time * config.fps as f32) as u32;

        // Zoom in, hold, zoom out pattern
        if i > 0 {
            zoom_expr.push_str("+");
            x_expr.push_str("+");
            y_expr.push_str("+");
        }

        // Zoom expression: lerp from 1 to zoom_level and back
        zoom_expr.push_str(&format!(
            "if(between(n\\,{}\\,{})\\,1+{}*(n-{})/{}\\,\
             if(between(n\\,{}\\,{})\\,{}\\,\
             if(between(n\\,{}\\,{})\\,{}-{}*(n-{})/{}\\,0)))",
            start_frame,
            zoom_in_end,
            kf.zoom - 1.0,
            start_frame,
            zoom_in_end - start_frame,
            zoom_in_end,
            hold_end,
            kf.zoom,
            hold_end,
            end_frame,
            kf.zoom,
            kf.zoom - 1.0,
            hold_end,
            end_frame - hold_end
        ));

        // X position: center on click point
        let x_offset = kf.center_x * config.width as f32;
        x_expr.push_str(&format!(
            "if(between(n\\,{}\\,{})\\,{}\\,0)",
            start_frame, end_frame, x_offset
        ));

        // Y position: center on click point
        let y_offset = kf.center_y * config.height as f32;
        y_expr.push_str(&format!(
            "if(between(n\\,{}\\,{})\\,{}\\,0)",
            start_frame, end_frame, y_offset
        ));
    }

    zoom_expr.push('\'');
    x_expr.push('\'');
    y_expr.push('\'');

    format!(
        "zoompan=z={}:x={}:y={}:d=1:s={}x{}:fps={}",
        zoom_expr, x_expr, y_expr, config.width, config.height, config.fps
    )
}

/// Generate complete ffmpeg command
pub fn generate_ffmpeg_command(events: &[RecordedEvent], config: &PostProcessConfig) -> String {
    let keyframes = generate_keyframes(events, config);

    if keyframes.is_empty() {
        // No zoom effects, just copy
        return format!(
            "ffmpeg -i \"{}\" -c copy \"{}\"",
            config.input_path, config.output_path
        );
    }

    let filter = generate_ffmpeg_filter(&keyframes, config);

    format!(
        "ffmpeg -i \"{}\" -vf \"{}\" -c:v libx264 -c:a copy \"{}\"",
        config.input_path, filter, config.output_path
    )
}

/// Apply post-processing using ffmpeg
pub fn apply_zoom_effects(
    events: &[RecordedEvent],
    config: &PostProcessConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let command = generate_ffmpeg_command(events, config);

    println!("Running ffmpeg post-processing...");
    println!("Command: {}", command);

    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(["/C", &command])
            .status()?;
    }

    #[cfg(not(windows))]
    {
        std::process::Command::new("sh")
            .args(["-c", &command])
            .status()?;
    }

    Ok(())
}
