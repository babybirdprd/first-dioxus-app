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

/// Start the event logger
pub fn start_event_logging() {
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
    println!("Event logging started");
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

        // Sample cursor position every 100ms
        if state.last_sample_time.elapsed() >= Duration::from_millis(100) {
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

/// Save events to JSON file
pub fn save_events(
    events: &[RecordedEvent],
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(events)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load events from JSON file
pub fn load_events(path: &PathBuf) -> Result<Vec<RecordedEvent>, Box<dyn std::error::Error>> {
    let json = std::fs::read_to_string(path)?;
    let events: Vec<RecordedEvent> = serde_json::from_str(&json)?;
    Ok(events)
}

/// Event logger struct for manual control (optional)
pub struct EventLogger {
    events: Vec<RecordedEvent>,
    start_time: Instant,
    device_state: DeviceState,
    last_mouse_state: MouseState,
    sample_interval: Duration,
    last_sample_time: Instant,
}

impl EventLogger {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            start_time: Instant::now(),
            device_state: DeviceState::new(),
            last_mouse_state: MouseState {
                coords: (0, 0),
                button_pressed: vec![],
            },
            sample_interval: Duration::from_millis(100), // Sample cursor 10x/sec
            last_sample_time: Instant::now(),
        }
    }

    /// Start the logger (reset timestamp)
    pub fn start(&mut self) {
        self.events.clear();
        self.start_time = Instant::now();
        self.last_sample_time = Instant::now();
    }

    /// Update - call this every frame to record events
    pub fn update(&mut self) {
        let mouse = self.device_state.get_mouse();
        let (x, y) = mouse.coords;
        let timestamp_ms = self.start_time.elapsed().as_millis() as u64;

        // Detect new click
        let is_clicking = !mouse.button_pressed.is_empty();
        let was_clicking = !self.last_mouse_state.button_pressed.is_empty();

        if is_clicking && !was_clicking {
            self.events
                .push(RecordedEvent::Click { x, y, timestamp_ms });
        }

        // Sample cursor position periodically
        if self.last_sample_time.elapsed() >= self.sample_interval {
            self.events
                .push(RecordedEvent::CursorMove { x, y, timestamp_ms });
            self.last_sample_time = Instant::now();
        }

        self.last_mouse_state = mouse;
    }

    /// Get all recorded events
    pub fn get_events(&self) -> &[RecordedEvent] {
        &self.events
    }

    /// Save events to JSON file
    pub fn save(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        save_events(&self.events, path)
    }

    /// Load events from JSON file
    pub fn load(path: &PathBuf) -> Result<Vec<RecordedEvent>, Box<dyn std::error::Error>> {
        load_events(path)
    }
}

impl Default for EventLogger {
    fn default() -> Self {
        Self::new()
    }
}
