//! Global hotkey management for the screen recorder
//!
//! Default hotkey: Ctrl+Shift+F9

use crate::shared_state;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};

/// Manages global hotkeys for the application
pub struct HotkeyManager {
    _manager: GlobalHotKeyManager,
    toggle_recording_id: u32,
}

impl HotkeyManager {
    /// Create a new hotkey manager and register default hotkeys
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let manager = GlobalHotKeyManager::new()?;

        // Ctrl+Shift+F9 for toggle recording
        let toggle_hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::F9);
        let toggle_recording_id = toggle_hotkey.id();

        manager.register(toggle_hotkey)?;

        println!("Registered hotkey: Ctrl+Shift+F9 for toggle recording");

        Ok(Self {
            _manager: manager,
            toggle_recording_id,
        })
    }

    /// Get the toggle recording hotkey ID
    pub fn toggle_id(&self) -> u32 {
        self.toggle_recording_id
    }
}

/// Start the hotkey listener in a background thread
pub fn start_hotkey_listener(toggle_id: u32) {
    std::thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();
        println!("Hotkey listener started");

        loop {
            if let Ok(event) = receiver.recv() {
                // Only trigger on key PRESS, not release
                if event.id == toggle_id && event.state == HotKeyState::Pressed {
                    shared_state::request_hotkey_toggle();
                }
            }
        }
    });
}
