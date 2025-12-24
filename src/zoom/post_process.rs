//! Post-processing for zoom effects using video-rs
//!
//! Uses video-rs for frame-by-frame processing with zoom/pan effects

use super::event_log::{EventLog, RecordedEvent};
use super::render_engine::{RenderEngine, RenderUniforms};
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

/// Represents a zoom keyframe
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
#[tracing::instrument(skip(log, config))]
pub fn generate_keyframes(log: &EventLog, config: &PostProcessConfig) -> Vec<ZoomKeyframe> {
    tracing::info!("Generating keyframes from {} events", log.events.len());
    let mut keyframes = Vec::new();

    // Normalize coordinates using the resolution the events were recorded in
    let screen_width = log.metadata.width as f32;
    let screen_height = log.metadata.height as f32;

    for event in &log.events {
        if let RecordedEvent::Click { x, y, timestamp_ms } = event {
            let start_time = *timestamp_ms as f32 / 1000.0;
            let end_time = start_time + config.hold_duration + config.zoom_duration * 2.0;

            // Normalize coordinates to 0-1 range based on RECORDING dimensions
            let center_x = *x as f32 / screen_width;
            let center_y = *y as f32 / screen_height;

            keyframes.push(ZoomKeyframe {
                start_time,
                end_time,
                center_x: center_x.clamp(0.0, 1.0),
                center_y: center_y.clamp(0.0, 1.0),
                zoom: config.zoom_level,
            });
        }
    }

    // Merge/chain overlapping keyframes
    let initial_count = keyframes.len();
    merge_overlapping_keyframes(&mut keyframes);
    tracing::info!(
        "Keyframe generation complete: {} -> {} keyframes",
        initial_count,
        keyframes.len()
    );

    keyframes
}

/// Adjust keyframes so they don't overlap, creating a sequential path
fn merge_overlapping_keyframes(keyframes: &mut Vec<ZoomKeyframe>) {
    if keyframes.len() < 2 {
        return;
    }

    // Sort by start time
    keyframes.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());

    // Sequential Pathing: If a new click happens while another is active,
    // the previous one should cut short to allow the new one to take over.
    for i in 0..keyframes.len() - 1 {
        if keyframes[i].end_time > keyframes[i + 1].start_time {
            // Cut previous short at the exact moment the next one starts
            keyframes[i].end_time = keyframes[i + 1].start_time;
        }
    }
}

/// Finds the nearest cursor position at a given time from the event log
fn get_cursor_pos_at(time_secs: f32, log: &EventLog) -> (f32, f32) {
    let mut nearest_pos = (0.5, 0.5);
    let mut min_diff = f32::MAX;

    let screen_width = log.metadata.width as f32;
    let screen_height = log.metadata.height as f32;

    for event in &log.events {
        let (x, y, timestamp_ms) = match event {
            RecordedEvent::Click { x, y, timestamp_ms } => (*x, *y, *timestamp_ms),
            RecordedEvent::CursorMove { x, y, timestamp_ms } => (*x, *y, *timestamp_ms),
        };

        let event_time = timestamp_ms as f32 / 1000.0;
        let diff = (time_secs - event_time).abs();

        if diff < min_diff {
            min_diff = diff;
            nearest_pos = (x as f32 / screen_width, y as f32 / screen_height);
        }

        // Cursors are sorted by time, so we can stop early if we pass the target time
        if event_time > time_secs + 0.5 {
            break;
        }
    }

    nearest_pos
}

/// Calculate camera state at a given time using "Magnetic Camera" interpolation
fn calculate_camera_at_time(
    time_secs: f32,
    keyframes: &[ZoomKeyframe],
    log: &EventLog,
    config: &PostProcessConfig,
) -> (f32, f32, f32) {
    // Returns (zoom_level, center_x, center_y)

    // 1. Find indices of previous, current, and next keyframes
    let mut current_idx = None;
    for (i, kf) in keyframes.iter().enumerate() {
        if time_secs >= kf.start_time && time_secs <= kf.end_time {
            current_idx = Some(i);
            break;
        }
    }

    if let Some(idx) = current_idx {
        let kf = &keyframes[idx];
        let zoom_in_end = kf.start_time + config.zoom_duration;

        // Ensure hold/zoom-out doesn't exceed the keyframe duration (which might be cut short by next click)
        let hold_end = (zoom_in_end + config.hold_duration).min(kf.end_time);

        // Zoom Interpolation
        let zoom = if time_secs < zoom_in_end {
            let t = (time_secs - kf.start_time) / config.zoom_duration;
            let eased_t = ease_in_out(t);
            // Zoom in from 1.0 to Target
            1.0 + (kf.zoom - 1.0) * eased_t
        } else if time_secs < hold_end {
            kf.zoom
        } else {
            // Zoom out back to 1.0
            let t = ((time_secs - hold_end) / config.zoom_duration).clamp(0.0, 1.0);
            let eased_t = ease_in_out(t);
            kf.zoom - (kf.zoom - 1.0) * eased_t
        };

        // Center Interpolation (Panning)
        // If this is not the first keyframe, pan from the previous click center
        let (start_cx, start_cy) = if idx > 0 {
            (keyframes[idx - 1].center_x, keyframes[idx - 1].center_y)
        } else {
            (0.5, 0.5)
        };

        // The pan should start as soon as the keyframe begins
        let pan_t = ((time_secs - kf.start_time) / config.zoom_duration).clamp(0.0, 1.0);
        let eased_pan_t = ease_in_out(pan_t);

        let mut cx = start_cx + (kf.center_x - start_cx) * eased_pan_t;
        let mut cy = start_cy + (kf.center_y - start_cy) * eased_pan_t;

        // "Cursor Follow" - Magnetic drift towards the live cursor during the hold phase
        if time_secs > zoom_in_end && time_secs < hold_end {
            let (cur_x, cur_y) = get_cursor_pos_at(time_secs, log);

            // Apply a 30% magnetic pull towards the cursor
            // This makes the camera feel "alive" and follow the action
            let follow_intensity = 0.35;
            cx = cx + (cur_x - cx) * follow_intensity;
            cy = cy + (cur_y - cy) * follow_intensity;
        }

        tracing::debug!(
            time = %format!("{:.3}", time_secs),
            zoom = %format!("{:.2}", zoom),
            center = %format!("{:.3},{:.3}", cx, cy),
            "Camera State"
        );

        return (zoom, cx, cy);
    }

    // No zoom active - return to center slowly if we just finished a keyframe
    // TODO: Implement "return to center" panning
    (1.0, 0.5, 0.5)
}

/// Smooth ease in/out curve
fn ease_in_out(t: f32) -> f32 {
    // Cubic easing for smoother transitions
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

// apply_zoom_to_frame removed in favor of RenderEngine

/// Apply post-processing using video-rs
#[tracing::instrument(skip(log, config))]
pub fn apply_zoom_effects(
    log: &EventLog,
    config: &PostProcessConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Applying zoom effects to: {}", config.input_path);
    use video_rs::decode::Decoder;
    use video_rs::encode::{Encoder, Settings};

    println!("Running video-rs post-processing...");

    // Initialize video-rs
    video_rs::init()?;

    // Open input video FIRST to get actual dimensions
    let source = Path::new(&config.input_path);
    let mut decoder = Decoder::new(source)?;

    // Get video properties
    let (width, height) = decoder.size();
    let frame_rate = decoder.frame_rate();
    println!("Input: {}x{} @ {:.2} fps", width, height, frame_rate);

    // Create a config copy with actual video dimensions for keyframe generation
    let mut actual_config = config.clone();
    actual_config.width = width as u32;
    actual_config.height = height as u32;
    actual_config.fps = frame_rate as u32;

    // NOW generate keyframes with actual dimensions
    let keyframes = generate_keyframes(log, &actual_config);
    println!(
        "Generated {} keyframes from {} events",
        keyframes.len(),
        log.events.len()
    );

    if keyframes.is_empty() {
        println!("No keyframes to apply, copying file...");
        std::fs::copy(&config.input_path, &config.output_path)?;
        return Ok(());
    }

    // Create encoder for output
    let destination = Path::new(&config.output_path);
    let settings = Settings::preset_h264_yuv420p(width as usize, height as usize, false);
    let mut encoder = Encoder::new(destination, settings)?;

    let mut processed = 0;

    println!("Starting frame processing...");

    // Initialize GPU Render Engine
    let mut render_engine = pollster::block_on(RenderEngine::new(width as u32, height as u32))?;

    let mut current_zoom = 1.0;
    let mut current_cx = 0.5;
    let mut current_cy = 0.5;

    // Process each frame
    for frame_result in decoder.decode_iter() {
        let (time, frame) = match frame_result {
            Ok(f) => f,
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("exhausted") {
                    println!("End of video stream at frame {}", processed);
                    break;
                }
                println!("Error decoding frame {}: {}", processed, e);
                continue;
            }
        };
        let time_secs = time.as_secs_f64() as f32;

        let prev_zoom = current_zoom;
        let prev_cx = current_cx;
        let prev_cy = current_cy;

        // Calculate camera state at this time
        let (zoom, cx, cy) = calculate_camera_at_time(time_secs, &keyframes, log, config);
        current_zoom = zoom;
        current_cx = cx;
        current_cy = cy;

        // OPTIMIZATION: If no zoom needed AND we wasn't zooming before, pass through original frame directly
        if (zoom - 1.0).abs() < 0.001 && (prev_zoom - 1.0).abs() < 0.001 {
            if let Err(e) = encoder.encode(&frame, time) {
                println!("Error encoding passthrough frame {}: {}", processed, e);
            }
            processed += 1;
            if processed % 100 == 0 {
                print!("\rProcessed {} frames (passthrough)...", processed);
                std::io::Write::flush(&mut std::io::stdout())?;
            }
            continue;
        }

        // GPU PATH: Frame needs zoom/pan/blur processing
        // Prepare uniforms
        let uniforms = RenderUniforms {
            zoom: current_zoom,
            center_x: current_cx,
            center_y: current_cy,
            aspect: width as f32 / height as f32,
            blur_samples: 5.0, // 5 samples for decent quality
            prev_center_x: prev_cx,
            prev_center_y: prev_cy,
            prev_zoom: prev_zoom,
            width: width as f32,
            height: height as f32,
        };

        // Convert ndarray RGB to RGBA for WGPU
        let frame_rgb = frame.as_slice().ok_or("Frame not contiguous")?;
        let mut frame_rgba = Vec::with_capacity((width * height * 4) as usize);
        for chunk in frame_rgb.chunks_exact(3) {
            frame_rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
        }

        // Process with GPU
        let processed_rgba = render_engine.process_frame(&frame_rgba, &uniforms)?;

        // Convert back to RGB for video-rs encoding
        let mut processed_rgb = Vec::with_capacity((width * height * 3) as usize);
        for chunk in processed_rgba.chunks_exact(4) {
            processed_rgb.extend_from_slice(&[chunk[0], chunk[1], chunk[2]]);
        }

        let zoomed_frame =
            video_rs::Frame::from_shape_vec((height as usize, width as usize, 3), processed_rgb)?;

        // Encode frame
        if let Err(e) = encoder.encode(&zoomed_frame, time) {
            println!("Error encoding frame {}: {}", processed, e);
        }

        processed += 1;
        if processed % 30 == 0 {
            print!("\rProcessed {} frames (GPU)...", processed);
            std::io::Write::flush(&mut std::io::stdout())?;
        }
    }

    // Finish encoding
    encoder.finish()?;

    println!(
        "\nVideo-rs processing complete! {} frames processed.",
        processed
    );
    Ok(())
}

// Keep the old generate_ffmpeg functions for reference but unused
#[allow(dead_code)]
pub fn generate_ffmpeg_command(log: &EventLog, config: &PostProcessConfig) -> String {
    let keyframes = generate_keyframes(log, config);

    if keyframes.is_empty() {
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

#[allow(dead_code)]
fn generate_ffmpeg_filter(_keyframes: &[ZoomKeyframe], _config: &PostProcessConfig) -> String {
    // Deprecated - kept for reference
    String::new()
}
