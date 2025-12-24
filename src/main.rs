use dioxus::prelude::*;

mod capture;
mod components;
mod config;
mod hotkey;
mod shared_state;
mod tray;
mod views;
mod zoom;

use capture::{start_recording, stop_recording, RecorderConfig};
use config::Config;
use hotkey::HotkeyManager;
use views::{Dashboard, Navbar, Settings};
use zoom::{start_event_logging, stop_event_logging, update_event_logging};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
        #[route("/")]
        Dashboard {},
        #[route("/settings")]
        Settings {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    // Initialize config
    let config = Config::load();
    println!("DemoRecorder starting...");
    println!("Output folder: {:?}", config.output_folder);
    println!("Hotkey: {}", config.hotkey);

    // Create output directory if needed
    if let Err(e) = std::fs::create_dir_all(&config.output_folder) {
        eprintln!("Failed to create output folder: {e}");
    }

    // Initialize hotkey manager and start listener
    match HotkeyManager::new() {
        Ok(hm) => {
            println!("Hotkeys registered successfully");
            hotkey::start_hotkey_listener(hm.toggle_id());
            // Keep the manager alive by leaking it (it needs to stay alive for hotkeys to work)
            Box::leak(Box::new(hm));
        }
        Err(e) => {
            eprintln!("Failed to register hotkeys: {e}");
        }
    };

    // Initialize system tray
    let _tray = match tray::create_tray() {
        Ok((tray_icon, menu_ids)) => {
            println!("System tray initialized");
            Some((tray_icon, menu_ids))
        }
        Err(e) => {
            eprintln!("Failed to create tray icon: {e}");
            None
        }
    };

    // Launch Dioxus app
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Recording state
    let mut is_rec = use_signal(|| false);
    let mut status_message = use_signal(|| "Ready".to_string());
    let mut current_events_path = use_signal(|| None::<std::path::PathBuf>);
    let mut saved_at = use_signal(|| None::<std::time::Instant>);

    // Poll for hotkey toggle requests and update event logging
    use_future(move || async move {
        loop {
            // Check every 100ms for hotkey toggle and update event logging
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            // Update event logging if recording
            if is_rec() {
                update_event_logging();
            }

            // Auto-reset status message after 3 seconds
            let should_reset = match *saved_at.read() {
                Some(t) => t.elapsed() >= std::time::Duration::from_secs(3),
                None => false,
            };
            if should_reset {
                status_message.set("Ready".to_string());
                saved_at.set(None);
            }

            if shared_state::take_hotkey_toggle() {
                let currently_recording = is_rec();
                if currently_recording {
                    // Stop recording and save events
                    let events = stop_event_logging();
                    stop_recording();
                    is_rec.set(false);

                    // Save events to file
                    if let Some(ref path) = *current_events_path.read() {
                        let event_count = events.len();
                        let monitor = windows_capture::monitor::Monitor::primary().unwrap();
                        let event_log = zoom::EventLog {
                            metadata: zoom::RecordingMetadata {
                                width: monitor.width().unwrap_or(1920),
                                height: monitor.height().unwrap_or(1080),
                            },
                            events,
                        };
                        if let Err(e) = zoom::save_event_log(&event_log, path) {
                            eprintln!("Failed to save events: {e}");
                        } else {
                            println!("Events saved to: {:?}", path);
                        }

                        status_message.set(format!("✓ Saved {} events", event_count));
                        println!("Hotkey: Recording stopped, {} events captured", event_count);
                    }

                    // Set a timestamp for auto-reset (handled in the polling loop)
                    saved_at.set(Some(std::time::Instant::now()));
                } else {
                    // Start recording and event logging
                    let monitor = windows_capture::monitor::Monitor::primary().unwrap();
                    let width = monitor.width().unwrap_or(1920);
                    let height = monitor.height().unwrap_or(1080);

                    start_event_logging(width, height);
                    let mut config = RecorderConfig::default();
                    config.width = width;
                    config.height = height;

                    current_events_path.set(Some(config.events_path.clone()));

                    match start_recording(config) {
                        Ok(_) => {
                            is_rec.set(true);
                            status_message.set("Recording...".to_string());
                            println!("Hotkey: Recording started");
                        }
                        Err(e) => {
                            stop_event_logging(); // Clean up
                            current_events_path.set(None);
                            status_message.set(format!("Error: {}", e));
                            eprintln!("Failed to start recording: {e}");
                        }
                    }
                }
            }
        }
    });

    // Toggle recording function (for button click)
    let toggle_recording = move |_| {
        let currently_recording = is_rec();
        if currently_recording {
            // Stop recording and event logging
            let events = stop_event_logging();
            stop_recording();
            is_rec.set(false);

            // Save events to file (same as hotkey handler)
            if let Some(ref path) = *current_events_path.read() {
                let event_count = events.len();
                let monitor = windows_capture::monitor::Monitor::primary().unwrap();
                let event_log = zoom::EventLog {
                    metadata: zoom::RecordingMetadata {
                        width: monitor.width().unwrap_or(1920),
                        height: monitor.height().unwrap_or(1080),
                    },
                    events,
                };
                if let Err(e) = zoom::save_event_log(&event_log, path) {
                    eprintln!("Failed to save events: {e}");
                } else {
                    println!("Events saved to: {:?}", path);
                }

                status_message.set(format!("✓ Saved {} events", event_count));
                println!("Recording stopped, {} events captured", event_count);
            }
            saved_at.set(Some(std::time::Instant::now()));
        } else {
            // Start recording and event logging
            let monitor = windows_capture::monitor::Monitor::primary().unwrap();
            let width = monitor.width().unwrap_or(1920);
            let height = monitor.height().unwrap_or(1080);

            start_event_logging(width, height);
            let mut config = RecorderConfig::default();
            config.width = width;
            config.height = height;

            current_events_path.set(Some(config.events_path.clone()));

            match start_recording(config) {
                Ok(_) => {
                    is_rec.set(true);
                    status_message.set("Recording...".to_string());
                    println!("Recording started at {}x{}", width, height);
                }
                Err(e) => {
                    stop_event_logging(); // Clean up
                    current_events_path.set(None);
                    status_message.set(format!("Error: {}", e));
                    eprintln!("Failed to start recording: {e}");
                }
            }
        }
    };

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        // Recording controls overlay - compact and polished
        div {
            class: "fixed top-4 right-4 z-50 rounded-xl px-4 py-2.5 shadow-2xl border border-gray-600/50",
            style: "background: rgba(17, 24, 39, 0.95); backdrop-filter: blur(12px);",
            div { class: "flex items-center gap-3",
                // Recording indicator dot
                div {
                    class: if is_rec() {
                        "w-2.5 h-2.5 rounded-full bg-red-500 animate-pulse shadow-lg shadow-red-500/50"
                    } else {
                        "w-2.5 h-2.5 rounded-full bg-gray-500"
                    }
                }

                // Status text - more compact
                span {
                    class: if is_rec() { "text-red-400 text-sm font-medium" } else { "text-gray-400 text-sm" },
                    "{status_message}"
                }

                // Toggle button - sleeker
                button {
                    class: if is_rec() {
                        "px-4 py-1.5 bg-red-600 hover:bg-red-500 text-white rounded-lg text-sm font-medium transition-all shadow-lg shadow-red-600/30"
                    } else {
                        "px-4 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white rounded-lg text-sm font-medium transition-all shadow-lg shadow-emerald-600/30"
                    },
                    onclick: toggle_recording,
                    if is_rec() { "■ Stop" } else { "● Record" }
                }
            }
        }

        // Recording border indicator (red glowing border when recording)
        if is_rec() {
            div {
                class: "fixed inset-0 pointer-events-none z-40 border-4 border-red-500 animate-pulse",
                style: "box-shadow: inset 0 0 20px rgba(239, 68, 68, 0.5);"
            }
        }

        Router::<Route> {}
    }
}
