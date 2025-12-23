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
                let events_path = path.with_extension("events.json");
                let events_exists = events_path.exists();

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
            div { class: "max-w-4xl mx-auto p-8",
                // Header
                div { class: "flex items-center justify-between mb-8",
                    h1 { class: "text-3xl font-bold", "🎬 DemoRecorder" }
                    div { class: "text-sm text-gray-400",
                        "Press Ctrl+Shift+F9 to record"
                    }
                }

                // Recordings folder info
                div { class: "mb-6 p-4 bg-gray-800/50 rounded-lg border border-gray-700",
                    div { class: "flex items-center justify-between",
                        div {
                            div { class: "text-sm text-gray-400", "📁 Recordings Folder" }
                            div { class: "text-sm font-mono truncate", "{dir.display()}" }
                        }
                        button {
                            class: "px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg text-sm transition",
                            onclick: refresh,
                            "🔄 Refresh"
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
            if entry.events_path.is_none() {
                status_msg.set("No events file found".to_string());
                return;
            }
            processing.set(true);
            status_msg.set("Processing...".to_string());

            // TODO: Actually run ffmpeg post-processing
            // For now just show it's possible
            status_msg.set("Zoom processing not yet implemented".to_string());
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
        div { class: "p-4 bg-gray-800/70 rounded-lg border border-gray-700 hover:border-gray-600 transition",
            div { class: "flex items-center justify-between",
                // File info
                div { class: "flex-1 min-w-0",
                    div { class: "font-medium truncate", "📹 {entry.filename}" }
                    div { class: "text-sm text-gray-400 flex items-center gap-3",
                        span { "{format_size(entry.size_bytes)}" }
                        if let Some(count) = entry.event_count {
                            span { class: "text-blue-400", "🎯 {count} events" }
                        }
                    }
                }

                // Actions
                div { class: "flex items-center gap-2 ml-4",
                    button {
                        class: "px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded text-sm transition",
                        onclick: open_file,
                        "▶ Play"
                    }
                    if entry.events_path.is_some() {
                        button {
                            class: "px-3 py-1.5 bg-purple-600 hover:bg-purple-700 rounded text-sm transition",
                            disabled: processing(),
                            onclick: apply_zoom,
                            "🔍 Zoom"
                        }
                    }
                    button {
                        class: "px-3 py-1.5 bg-red-600/50 hover:bg-red-600 rounded text-sm transition",
                        onclick: delete_recording,
                        "🗑️"
                    }
                }
            }

            // Status message
            if !status_msg().is_empty() {
                div { class: "mt-2 text-sm text-yellow-400", "{status_msg}" }
            }
        }
    }
}
