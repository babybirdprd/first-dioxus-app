//! System tray icon for DemoRecorder
//! Provides background access to recording controls

use tray_icon::{TrayIcon, TrayIconBuilder};

/// Create the system tray icon
pub fn create_tray() -> Result<TrayIcon, Box<dyn std::error::Error>> {
    // Create tray icon (using a simple colored icon)
    let icon = create_icon()?;

    let tray = TrayIconBuilder::new()
        .with_tooltip("DemoRecorder - Ctrl+Shift+F9")
        .with_icon(icon)
        .build()?;

    Ok(tray)
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
