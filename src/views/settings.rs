//! Settings view component

use crate::config::{AudioMode, CaptureTarget, Config, OutputFormat};
use dioxus::prelude::*;

/// Settings page component
#[component]
pub fn Settings() -> Element {
    let mut config = use_signal(Config::load);
    let mut save_status = use_signal(|| String::new());

    // Save handler
    let save_config = move |_| match config().save() {
        Ok(_) => save_status.set("Settings saved!".to_string()),
        Err(e) => save_status.set(format!("Error: {}", e)),
    };

    rsx! {
        div { class: "min-h-screen bg-gray-900 text-white p-8",
            div { class: "max-w-2xl mx-auto",
                h1 { class: "text-3xl font-bold mb-8", "⚙️ Settings" }

                // Capture Target
                div { class: "mb-6",
                    label { class: "block text-sm font-medium mb-2", "Capture Source" }
                    select {
                        class: "w-full bg-gray-800 border border-gray-700 rounded-lg p-3",
                        value: format!("{:?}", config().capture_target),
                        onchange: move |e| {
                            let mut c = config();
                            c.capture_target = match e.value().as_str() {
                                "ForegroundWindow" => CaptureTarget::ForegroundWindow,
                                _ => CaptureTarget::PrimaryMonitor,
                            };
                            config.set(c);
                        },
                        option { value: "PrimaryMonitor", "Primary Monitor (Full Screen)" }
                        option { value: "ForegroundWindow", "Foreground Window" }
                    }
                }

                // Output Format
                div { class: "mb-6",
                    label { class: "block text-sm font-medium mb-2", "Output Format" }
                    select {
                        class: "w-full bg-gray-800 border border-gray-700 rounded-lg p-3",
                        value: format!("{:?}", config().output_format),
                        onchange: move |e| {
                            let mut c = config();
                            c.output_format = match e.value().as_str() {
                                "WebM" => OutputFormat::WebM,
                                "Gif" => OutputFormat::Gif,
                                _ => OutputFormat::Mp4,
                            };
                            config.set(c);
                        },
                        option { value: "Mp4", "MP4 (H.264)" }
                        option { value: "WebM", "WebM" }
                        option { value: "Gif", "GIF" }
                    }
                }

                // Audio Mode
                div { class: "mb-6",
                    label { class: "block text-sm font-medium mb-2", "Audio" }
                    select {
                        class: "w-full bg-gray-800 border border-gray-700 rounded-lg p-3",
                        value: format!("{:?}", config().audio_mode),
                        onchange: move |e| {
                            let mut c = config();
                            c.audio_mode = match e.value().as_str() {
                                "System" => AudioMode::System,
                                "Microphone" => AudioMode::Microphone,
                                "Both" => AudioMode::Both,
                                _ => AudioMode::None,
                            };
                            config.set(c);
                        },
                        option { value: "None", "No Audio" }
                        option { value: "System", "System Audio" }
                        option { value: "Microphone", "Microphone" }
                        option { value: "Both", "System + Mic" }
                    }
                }

                // FPS
                div { class: "mb-6",
                    label { class: "block text-sm font-medium mb-2", "Frame Rate" }
                    select {
                        class: "w-full bg-gray-800 border border-gray-700 rounded-lg p-3",
                        value: config().fps.to_string(),
                        onchange: move |e| {
                            let mut c = config();
                            c.fps = e.value().parse().unwrap_or(30);
                            config.set(c);
                        },
                        option { value: "24", "24 FPS" }
                        option { value: "30", "30 FPS" }
                        option { value: "60", "60 FPS" }
                    }
                }

                // Hotkey display
                div { class: "mb-6",
                    label { class: "block text-sm font-medium mb-2", "Hotkey" }
                    div { class: "bg-gray-800 border border-gray-700 rounded-lg p-3 font-mono",
                        "{config().hotkey}"
                    }
                }

                // Output folder
                div { class: "mb-8",
                    label { class: "block text-sm font-medium mb-2", "Output Folder" }
                    div { class: "bg-gray-800 border border-gray-700 rounded-lg p-3 text-sm text-gray-400 truncate",
                        "{config().output_folder.display()}"
                    }
                }

                // Save button
                div { class: "flex items-center gap-4",
                    button {
                        class: "px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg font-medium transition",
                        onclick: save_config,
                        "Save Settings"
                    }
                    span { class: "text-green-400", "{save_status}" }
                }
            }
        }
    }
}
