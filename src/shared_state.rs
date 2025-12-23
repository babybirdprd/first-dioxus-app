//! Shared state for cross-thread communication

use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag set by hotkey thread when toggle is requested
static HOTKEY_TOGGLE_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Check if a hotkey toggle was requested and clear the flag
pub fn take_hotkey_toggle() -> bool {
    HOTKEY_TOGGLE_REQUESTED.swap(false, Ordering::SeqCst)
}

/// Request a hotkey toggle (called from hotkey listener thread)
pub fn request_hotkey_toggle() {
    HOTKEY_TOGGLE_REQUESTED.store(true, Ordering::SeqCst);
    println!("Hotkey toggle requested!");
}
