//! Settings view component

use crate::config::{AudioMode, Config, OutputFormat, ZoomMode};
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

                // Zoom Mode
                div { class: "mb-6",
                    label { class: "block text-sm font-medium mb-2", "Smart Zoom" }
                    select {
                        class: "w-full bg-gray-800 border border-gray-700 rounded-lg p-3",
                        value: format!("{:?}", config().zoom_mode),
                        onchange: move |e| {
                            let mut c = config();
                            c.zoom_mode = match e.value().as_str() {
                                "FollowCursor" => ZoomMode::FollowCursor,
                                "ClickToZoom" => ZoomMode::ClickToZoom,
                                "SmartAI" => ZoomMode::SmartAI,
                                _ => ZoomMode::None,
                            };
                            config.set(c);
                        },
                        option { value: "None", "Disabled" }
                        option { value: "FollowCursor", "Follow Cursor" }
                        option { value: "ClickToZoom", "Zoom on Click" }
                        option { value: "SmartAI", "Smart AI (Experimental)" }
                    }
                }

                // Zoom Level
                div { class: "mb-6",
                    label { class: "block text-sm font-medium mb-2",
                        "Zoom Level: {config().zoom_level:.1}x"
                    }
                    input {
                        r#type: "range",
                        class: "w-full",
                        min: "1.0",
                        max: "3.0",
                        step: "0.1",
                        value: config().zoom_level.to_string(),
                        oninput: move |e| {
                            let mut c = config();
                            c.zoom_level = e.value().parse().unwrap_or(1.5);
                            config.set(c);
                        }
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
