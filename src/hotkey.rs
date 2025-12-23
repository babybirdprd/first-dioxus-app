//! Global hotkey management for the screen recorder
//!
//! Default hotkey: Ctrl+Shift+F9

use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
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

    /// Check if the given event matches toggle recording
    pub fn is_toggle_recording(&self, event: &GlobalHotKeyEvent) -> bool {
        event.id == self.toggle_recording_id
    }
}
