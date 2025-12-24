//! Dashboard view - main recordings library and controls

use dioxus::prelude::*;
use std::path::PathBuf;

/// Recording entry in the library
#[derive(Clone, Debug, PartialEq)]
pub struct RecordingEntry {
    pub filename: String,
    pub path: PathBuf,
    pub events_path: Option<PathBuf>,
    pub size_bytes: u64,
    pub event_count: Option<usize>,
}

/// Get the recordings directory
fn get_recordings_dir() -> PathBuf {
    dirs::video_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default())
        .join("DemoRecorder")
}

/// Scan for recordings in the output folder
fn scan_recordings() -> Vec<RecordingEntry> {
    let dir = get_recordings_dir();
    let mut entries = Vec::new();

    if let Ok(read_dir) = std::fs::read_dir(&dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "mp4").unwrap_or(false) {
                let filename = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

                // Check for matching events file
                // Try both patterns: .events.json (replacing .mp4) and .mp4.events.json
                let events_path_a = path.with_extension("events.json");
                let events_path_b = PathBuf::from(format!("{}.events.json", path.display()));

                let (events_path, events_exists) = if events_path_a.exists() {
                    (events_path_a, true)
                } else if events_path_b.exists() {
                    (events_path_b, true)
                } else {
                    println!(
                        "No events file found. Checked: {:?} and {:?}",
                        events_path_a, events_path_b
                    );
                    (events_path_a, false)
                };

                // Count events if file exists
                let event_count = if events_exists {
                    std::fs::read_to_string(&events_path)
                        .ok()
                        .and_then(|s| serde_json::from_str::<Vec<serde_json::Value>>(&s).ok())
                        .map(|v| v.len())
                } else {
                    None
                };

                entries.push(RecordingEntry {
                    filename,
                    path,
                    events_path: if events_exists {
                        Some(events_path)
                    } else {
                        None
                    },
                    size_bytes,
                    event_count,
                });
            }
        }
    }

    // Sort by modification time (newest first)
    entries.sort_by(|a, b| {
        let a_time = std::fs::metadata(&a.path).and_then(|m| m.modified()).ok();
        let b_time = std::fs::metadata(&b.path).and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    entries
}

/// Format file size for display
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Dashboard component
#[component]
pub fn Dashboard() -> Element {
    let mut recordings = use_signal(scan_recordings);
    let mut refresh_trigger = use_signal(|| 0);

    // Refresh recordings list
    let refresh = move |_| {
        recordings.set(scan_recordings());
        refresh_trigger += 1;
    };

    let dir = get_recordings_dir();

    rsx! {
        div { class: "min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900 text-white",
            div { class: "max-w-4xl mx-auto p-8 pt-6",
                // Header - cleaner, no duplicate hotkey tip
                div { class: "mb-8",
                    h1 { class: "text-2xl font-bold flex items-center gap-3",
                        span { class: "text-3xl", "🎬" }
                        span { "DemoRecorder" }
                    }
                }

                // Recordings folder info - more subtle
                div { class: "mb-6 p-4 bg-gray-800/30 rounded-xl border border-gray-700/50",
                    div { class: "flex items-center justify-between",
                        div { class: "flex items-center gap-3 min-w-0",
                            span { class: "text-xl", "📁" }
                            div { class: "min-w-0",
                                div { class: "text-xs text-gray-500 uppercase tracking-wide", "Recordings Folder" }
                                div { class: "text-sm font-mono text-gray-300 truncate", "{dir.display()}" }
                            }
                        }
                        button {
                            class: "px-3 py-1.5 bg-gray-700/50 hover:bg-gray-600 rounded-lg text-sm transition flex items-center gap-2",
                            onclick: refresh,
                            span { "🔄" }
                            span { "Refresh" }
                        }
                    }
                }

                // Recordings list
                div { class: "space-y-3",
                    if recordings().is_empty() {
                        div { class: "text-center py-12 text-gray-500",
                            div { class: "text-4xl mb-4", "📹" }
                            div { "No recordings yet" }
                            div { class: "text-sm", "Press Ctrl+Shift+F9 to start recording" }
                        }
                    } else {
                        for entry in recordings() {
                            RecordingCard { entry: entry.clone() }
                        }
                    }
                }

                // Stats
                if !recordings().is_empty() {
                    div { class: "mt-8 pt-6 border-t border-gray-700 text-center text-sm text-gray-500",
                        "{recordings().len()} recording(s)"
                    }
                }
            }
        }
    }
}

/// Individual recording card
#[component]
fn RecordingCard(entry: RecordingEntry) -> Element {
    let mut processing = use_signal(|| false);
    let mut status_msg = use_signal(|| String::new());

    // Open file in default player
    let open_file = {
        let path = entry.path.clone();
        move |_| {
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "start", "", &path.to_string_lossy()])
                    .spawn();
            }
        }
    };

    // Apply zoom effects
    let apply_zoom = {
        let entry = entry.clone();
        move |_| {
            let events_path = match &entry.events_path {
                Some(p) => p.clone(),
                None => {
                    status_msg.set("No events file found".to_string());
                    return;
                }
            };

            processing.set(true);
            status_msg.set("Loading events...".to_string());

            // Load events from JSON file
            let events = match crate::zoom::load_events(&events_path) {
                Ok(e) => e,
                Err(err) => {
                    status_msg.set(format!("Failed to load events: {}", err));
                    processing.set(false);
                    return;
                }
            };

            if events.is_empty() {
                status_msg.set("No events to process".to_string());
                processing.set(false);
                return;
            }

            // Create output path
            let input_path = entry.path.clone();
            let output_path = input_path.with_file_name(format!(
                "{}_zoomed.mp4",
                input_path.file_stem().unwrap_or_default().to_string_lossy()
            ));

            status_msg.set(format!("Processing {} events...", events.len()));

            // Create config and run ffmpeg
            let config = crate::zoom::PostProcessConfig {
                input_path: input_path.to_string_lossy().to_string(),
                output_path: output_path.to_string_lossy().to_string(),
                width: 1920,
                height: 1080,
                fps: 30,
                zoom_level: 1.5,
                zoom_duration: 0.3,
                hold_duration: 2.0,
            };

            match crate::zoom::apply_zoom_effects(&events, &config) {
                Ok(_) => {
                    status_msg.set(format!(
                        "✓ Saved to {}",
                        output_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                    ));
                }
                Err(err) => {
                    status_msg.set(format!("FFmpeg error: {}", err));
                }
            }
            processing.set(false);
        }
    };

    // Delete recording
    let delete_recording = {
        let path = entry.path.clone();
        let events_path = entry.events_path.clone();
        move |_| {
            let _ = std::fs::remove_file(&path);
            if let Some(ref ep) = events_path {
                let _ = std::fs::remove_file(ep);
            }
        }
    };

    rsx! {
        div {
            class: "p-4 bg-gray-800/40 rounded-xl border border-gray-700/50 hover:border-gray-600/70 hover:bg-gray-800/60 transition-all",
            div { class: "flex items-center justify-between",
                // File info
                div { class: "flex items-center gap-3 flex-1 min-w-0",
                    // Video icon
                    div { class: "text-2xl", "🎬" }
                    div { class: "min-w-0",
                        div { class: "font-medium truncate text-gray-100", "{entry.filename}" }
                        div { class: "text-xs text-gray-500 flex items-center gap-2 mt-0.5",
                            span { "{format_size(entry.size_bytes)}" }
                            if let Some(count) = entry.event_count {
                                span { class: "text-blue-400 bg-blue-500/10 px-1.5 py-0.5 rounded", "🎯 {count} events" }
                            } else {
                                span { class: "text-gray-600", "No events" }
                            }
                        }
                    }
                }

                // Actions
                div { class: "flex items-center gap-2 ml-4",
                    button {
                        class: "px-3 py-1.5 bg-blue-600 hover:bg-blue-500 rounded-lg text-sm font-medium transition-all shadow-lg shadow-blue-600/20",
                        onclick: open_file,
                        "▶ Play"
                    }
                    if entry.events_path.is_some() {
                        button {
                            class: if processing() {
                                "px-3 py-1.5 bg-purple-800 rounded-lg text-sm font-medium cursor-wait opacity-70"
                            } else {
                                "px-3 py-1.5 bg-purple-600 hover:bg-purple-500 rounded-lg text-sm font-medium transition-all shadow-lg shadow-purple-600/20"
                            },
                            disabled: processing(),
                            onclick: apply_zoom,
                            if processing() { "Processing..." } else { "🔍 Zoom" }
                        }
                    }
                    button {
                        class: "px-2.5 py-1.5 bg-gray-700/50 hover:bg-red-600 rounded-lg text-sm transition-all",
                        onclick: delete_recording,
                        "🗑️"
                    }
                }
            }

            // Status message
            if !status_msg().is_empty() {
                div { class: "mt-3 text-sm text-amber-400 bg-amber-500/10 px-3 py-1.5 rounded-lg", "{status_msg}" }
            }
        }
    }
}
