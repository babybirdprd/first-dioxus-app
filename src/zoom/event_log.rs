//! Event log for tracking mouse/keyboard events during recording
//!
//! Records clicks and cursor positions with timestamps for post-processing zoom

use device_query::{DeviceQuery, DeviceState, MouseState};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Global event log
static EVENTS_LOG: Mutex<Option<EventLoggerState>> = Mutex::new(None);

/// State for the event logger
struct EventLoggerState {
    events: Vec<RecordedEvent>,
    start_time: Instant,
    device_state: DeviceState,
    last_mouse_state: MouseState,
    last_sample_time: Instant,
}

/// A recorded event during capture
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecordedEvent {
    /// Mouse click at position
    Click { x: i32, y: i32, timestamp_ms: u64 },
    /// Cursor position sample
    CursorMove { x: i32, y: i32, timestamp_ms: u64 },
}

/// Metadata for the recording session
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordingMetadata {
    pub width: u32,
    pub height: u32,
}

/// Complete event log with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventLog {
    pub metadata: RecordingMetadata,
    pub events: Vec<RecordedEvent>,
}

/// Start the event logger
pub fn start_event_logging(width: u32, height: u32) {
    let mut log = EVENTS_LOG.lock().unwrap();
    *log = Some(EventLoggerState {
        events: Vec::new(),
        start_time: Instant::now(),
        device_state: DeviceState::new(),
        last_mouse_state: MouseState {
            coords: (0, 0),
            button_pressed: vec![],
        },
        last_sample_time: Instant::now(),
    });
    // We effectively store width/height in the state if needed, but for now we'll pass them to stop()
    println!("Event logging started for {}x{}", width, height);
}

/// Update the event logger (call periodically during recording)
pub fn update_event_logging() {
    let mut log = EVENTS_LOG.lock().unwrap();
    if let Some(ref mut state) = *log {
        let mouse = state.device_state.get_mouse();
        let (x, y) = mouse.coords;
        let timestamp_ms = state.start_time.elapsed().as_millis() as u64;

        // Detect new click
        let is_clicking = !mouse.button_pressed.is_empty();
        let was_clicking = !state.last_mouse_state.button_pressed.is_empty();

        if is_clicking && !was_clicking {
            state
                .events
                .push(RecordedEvent::Click { x, y, timestamp_ms });
        }

        // Sample cursor position every 16ms (60Hz) for smoother 4K tracking
        if state.last_sample_time.elapsed() >= Duration::from_millis(16) {
            state
                .events
                .push(RecordedEvent::CursorMove { x, y, timestamp_ms });
            state.last_sample_time = Instant::now();
        }

        state.last_mouse_state = mouse;
    }
}

/// Stop event logging and get events
pub fn stop_event_logging() -> Vec<RecordedEvent> {
    let mut log = EVENTS_LOG.lock().unwrap();
    if let Some(state) = log.take() {
        println!(
            "Event logging stopped. {} events recorded.",
            state.events.len()
        );
        state.events
    } else {
        Vec::new()
    }
}

/// Save event log to JSON file
pub fn save_event_log(log: &EventLog, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(log)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load event log from JSON file
pub fn load_event_log(path: &PathBuf) -> Result<EventLog, Box<dyn std::error::Error>> {
    let json = std::fs::read_to_string(path)?;
    // Try loading as new EventLog format first
    if let Ok(log) = serde_json::from_str::<EventLog>(&json) {
        return Ok(log);
    }
    // Fallback for old format (just Vec<RecordedEvent>)
    let events: Vec<RecordedEvent> = serde_json::from_str(&json)?;
    Ok(EventLog {
        metadata: RecordingMetadata {
            width: 1920,
            height: 1080,
        }, // Default for old recordings
        events,
    })
}
