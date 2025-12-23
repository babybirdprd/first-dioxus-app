use dioxus::prelude::*;

mod capture;
mod components;
mod config;
mod hotkey;
mod views;

use capture::{is_recording, start_recording, stop_recording, RecorderConfig};
use components::Hero;
use config::Config;
use hotkey::HotkeyManager;
use views::{Blog, Home, Navbar};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
        #[route("/")]
        Home {},
        #[route("/blog/:id")]
        Blog { id: i32 },
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

    // Initialize hotkey manager
    let _hotkey_manager = match HotkeyManager::new() {
        Ok(hm) => {
            println!("Hotkeys registered successfully");
            Some(hm)
        }
        Err(e) => {
            eprintln!("Failed to register hotkeys: {e}");
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
    let mut status_message = use_signal(|| "Ready to record (Ctrl+Shift+F9)".to_string());

    // Toggle recording function
    let toggle_recording = move |_| {
        let currently_recording = is_rec();
        if currently_recording {
            // Stop recording
            stop_recording();
            is_rec.set(false);
            status_message.set("Recording saved!".to_string());
            println!("Recording stopped");
        } else {
            // Start recording
            let config = RecorderConfig::default();
            match start_recording(config) {
                Ok(_) => {
                    is_rec.set(true);
                    status_message.set("Recording...".to_string());
                    println!("Recording started");
                }
                Err(e) => {
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

        // Recording controls overlay
        div { class: "fixed top-4 right-4 z-50 bg-gray-900/90 rounded-lg p-4 shadow-xl border border-gray-700",
            div { class: "flex items-center gap-4",
                // Recording indicator
                if is_rec() {
                    div { class: "w-3 h-3 bg-red-500 rounded-full animate-pulse" }
                } else {
                    div { class: "w-3 h-3 bg-gray-500 rounded-full" }
                }

                // Status text
                span { class: "text-white text-sm", "{status_message}" }

                // Toggle button
                button {
                    class: if is_rec() {
                        "px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg font-medium transition"
                    } else {
                        "px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg font-medium transition"
                    },
                    onclick: toggle_recording,
                    if is_rec() { "Stop" } else { "Record" }
                }
            }
        }

        Router::<Route> {}
    }
}
