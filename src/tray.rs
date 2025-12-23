//! System tray icon for DemoRecorder
//! Provides background access to recording controls

use std::sync::atomic::{AtomicBool, Ordering};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIcon, TrayIconBuilder,
};

static SHOULD_QUIT: AtomicBool = AtomicBool::new(false);

/// Check if app should quit
pub fn should_quit() -> bool {
    SHOULD_QUIT.load(Ordering::SeqCst)
}

/// Menu item IDs
pub struct TrayMenuIds {
    pub record: MenuItem,
    pub settings: MenuItem,
    pub quit: MenuItem,
}

/// Create the system tray icon with menu
pub fn create_tray() -> Result<(TrayIcon, TrayMenuIds), Box<dyn std::error::Error>> {
    // Create menu items
    let record = MenuItem::new("Start Recording", true, None);
    let settings = MenuItem::new("Settings", true, None);
    let quit = MenuItem::new("Quit", true, None);

    // Build menu
    let menu = Menu::new();
    menu.append(&record)?;
    menu.append(&settings)?;
    menu.append(&quit)?;

    // Create tray icon (using a simple colored icon)
    let icon = create_icon()?;

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("DemoRecorder - Ctrl+Shift+F9")
        .with_icon(icon)
        .build()?;

    Ok((
        tray,
        TrayMenuIds {
            record,
            settings,
            quit,
        },
    ))
}

/// Handle tray menu events
pub fn handle_menu_event(
    event: MenuEvent,
    menu_ids: &TrayMenuIds,
    on_toggle_recording: impl Fn(),
    on_settings: impl Fn(),
) {
    if event.id == menu_ids.record.id() {
        on_toggle_recording();
    } else if event.id == menu_ids.settings.id() {
        on_settings();
    } else if event.id == menu_ids.quit.id() {
        SHOULD_QUIT.store(true, Ordering::SeqCst);
    }
}

/// Update record menu item text based on recording state
pub fn update_record_menu(menu_ids: &TrayMenuIds, is_recording: bool) {
    let text = if is_recording {
        "Stop Recording"
    } else {
        "Start Recording"
    };
    menu_ids.record.set_text(text);
}

/// Create a simple colored icon (red circle for recording indicator)
fn create_icon() -> Result<tray_icon::Icon, Box<dyn std::error::Error>> {
    // Create a 32x32 RGBA icon
    let size = 32;
    let mut rgba = vec![0u8; size * size * 4];

    // Draw a red circle
    let center = size as f32 / 2.0;
    let radius = size as f32 / 2.0 - 2.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();

            let idx = (y * size + x) * 4;
            if dist <= radius {
                // Red circle
                rgba[idx] = 220; // R
                rgba[idx + 1] = 50; // G
                rgba[idx + 2] = 50; // B
                rgba[idx + 3] = 255; // A
            } else if dist <= radius + 1.0 {
                // Anti-aliased edge
                let alpha = ((radius + 1.0 - dist) * 255.0) as u8;
                rgba[idx] = 220;
                rgba[idx + 1] = 50;
                rgba[idx + 2] = 50;
                rgba[idx + 3] = alpha;
            }
        }
    }

    let icon = tray_icon::Icon::from_rgba(rgba, size as u32, size as u32)?;
    Ok(icon)
}
