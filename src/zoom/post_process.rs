//! Post-processing for zoom effects using video-rs
//!
//! Uses video-rs for frame-by-frame processing with zoom/pan effects

use super::event_log::RecordedEvent;
use image::{imageops, ImageBuffer, Rgb};
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

/// Calculate zoom level at a given time
fn calculate_zoom_at_time(
    time_secs: f32,
    keyframes: &[ZoomKeyframe],
    config: &PostProcessConfig,
) -> (f32, f32, f32) {
    // Returns (zoom_level, center_x, center_y)
    for kf in keyframes {
        if time_secs >= kf.start_time && time_secs <= kf.end_time {
            let zoom_in_end = kf.start_time + config.zoom_duration;
            let hold_end = zoom_in_end + config.hold_duration;

            let zoom = if time_secs < zoom_in_end {
                // Zooming in - smooth ease
                let t = (time_secs - kf.start_time) / config.zoom_duration;
                let eased_t = ease_in_out(t);
                1.0 + (kf.zoom - 1.0) * eased_t
            } else if time_secs < hold_end {
                // Holding zoom
                kf.zoom
            } else {
                // Zooming out - smooth ease
                let t = (time_secs - hold_end) / config.zoom_duration;
                let eased_t = ease_in_out(t);
                kf.zoom - (kf.zoom - 1.0) * eased_t
            };

            return (zoom, kf.center_x, kf.center_y);
        }
    }

    // No zoom active
    (1.0, 0.5, 0.5)
}

/// Smooth ease in/out curve
fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    // Smooth step function
    t * t * (3.0 - 2.0 * t)
}

/// Apply zoom effect to a frame
fn apply_zoom_to_frame(
    frame: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    zoom: f32,
    center_x: f32,
    center_y: f32,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    if (zoom - 1.0).abs() < 0.01 {
        // No zoom, return as-is
        return frame.clone();
    }

    let (width, height) = frame.dimensions();

    // Calculate crop rectangle (inverse zoom = crop)
    let crop_width = (width as f32 / zoom) as u32;
    let crop_height = (height as f32 / zoom) as u32;

    // Center the crop on the click point
    let center_px_x = (center_x * width as f32) as i32;
    let center_px_y = (center_y * height as f32) as i32;

    let crop_x = (center_px_x - crop_width as i32 / 2).clamp(0, (width - crop_width) as i32) as u32;
    let crop_y =
        (center_px_y - crop_height as i32 / 2).clamp(0, (height - crop_height) as i32) as u32;

    // Crop and scale back to original size
    let cropped = imageops::crop_imm(frame, crop_x, crop_y, crop_width, crop_height).to_image();
    imageops::resize(&cropped, width, height, imageops::FilterType::Lanczos3)
}

/// Apply post-processing using video-rs
pub fn apply_zoom_effects(
    events: &[RecordedEvent],
    config: &PostProcessConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    use video_rs::decode::Decoder;
    use video_rs::encode::{Encoder, Settings};
    use video_rs::time::Time;

    println!("Running video-rs post-processing...");

    let keyframes = generate_keyframes(events, config);
    println!(
        "Generated {} keyframes from {} events",
        keyframes.len(),
        events.len()
    );

    if keyframes.is_empty() {
        println!("No keyframes to apply, copying file...");
        std::fs::copy(&config.input_path, &config.output_path)?;
        return Ok(());
    }

    // Initialize video-rs
    video_rs::init()?;

    // Open input video
    let source = Path::new(&config.input_path);
    let mut decoder = Decoder::new(source)?;

    // Get video properties
    let (width, height) = decoder.size();
    let frame_rate = decoder.frame_rate();
    println!("Input: {}x{} @ {:.2} fps", width, height, frame_rate);

    // Create encoder for output
    let destination = Path::new(&config.output_path);
    let settings = Settings::preset_h264_yuv420p(width as usize, height as usize, false);
    let mut encoder = Encoder::new(destination, settings)?;

    let _frame_duration = Time::from_secs(1.0 / frame_rate);
    let mut frame_idx: usize = 0;
    let mut processed = 0;

    // Process each frame
    for frame_result in decoder.decode_iter() {
        let (time, frame) = frame_result?;
        let time_secs = time.as_secs() as f32;

        // Calculate zoom at this time
        let (zoom, cx, cy) = calculate_zoom_at_time(time_secs, &keyframes, config);

        // Convert frame to image - video-rs frame is ndarray
        // Get the raw data from the ndarray frame
        let frame_slice = frame.as_slice().ok_or("Frame not contiguous")?;

        // video-rs gives us RGB data, convert to image
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::from_raw(width as u32, height as u32, frame_slice.to_vec())
                .ok_or("Failed to create image buffer")?;

        // Apply zoom
        let zoomed = apply_zoom_to_frame(&img, zoom, cx, cy);

        // Convert back to ndarray for encoding
        let zoomed_data: Vec<u8> = zoomed.into_raw();
        let zoomed_frame =
            video_rs::Frame::from_shape_vec((height as usize, width as usize, 3), zoomed_data)?;

        // Encode frame
        let position = Time::from_nth_of_a_second(frame_idx);
        encoder.encode(&zoomed_frame, position)?;

        frame_idx += 1;
        processed += 1;

        // Progress every 30 frames
        if processed % 30 == 0 {
            print!("\rProcessed {} frames...", processed);
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
pub fn generate_ffmpeg_command(events: &[RecordedEvent], config: &PostProcessConfig) -> String {
    let keyframes = generate_keyframes(events, config);

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
