---
description: Continue DemoRecorder development - Phase 4 AI Zoom and beyond
---

# DemoRecorder Continuation Workflow

## Project Overview
Cross-platform screen recorder with smart zoom effects, built with Dioxus.

## Completed Features
- **Phase 1**: Dioxus app scaffold, windows-capture recording, global hotkey (Ctrl+Shift+F9)
- **Phase 2**: Settings UI, capture source selection (monitor/window), recording indicator
- **Phase 3**: Post-processing zoom system (event logging, ffmpeg zoompan filter generator)
- **UI Cleanup**: Dashboard with recordings library, auto-timestamp filenames, events.json saving

## Current State
- Recording works with timestamped files: `recording_{timestamp}.mp4` + `.events.json`
- Dashboard shows recordings library with Play, Zoom (placeholder), Delete buttons
- Settings page has: Output Format, Capture Source, Audio, FPS, Hotkey, Output Folder

## Key Files
- `src/main.rs` - Main app, hotkey polling, event logging integration
- `src/capture/recorder.rs` - Screen capture with RecorderConfig
- `src/views/dashboard.rs` - Recordings library UI
- `src/views/settings.rs` - Configuration UI
- `src/zoom/event_log.rs` - Mouse event tracking with timestamps
- `src/zoom/post_process.rs` - FFmpeg zoompan command generator
- `src/zoom/camera.rs` - Camera interpolation math (for future use)

## Remaining Work

### Quick Fixes (Before Next Session)
1. **Remove duplicate hotkey tip** - Shows "Press Ctrl+Shift+F9" twice (Dashboard header + overlay)
2. **Style scrollbar** - Default browser scrollbar is ugly
3. **UI Polish** - Better visual hierarchy, consistent spacing

### UI: Wire Apply Zoom Button
1. In `dashboard.rs`, update `apply_zoom` handler to:
   - Load events from `.events.json`
   - Call `zoom::apply_zoom_effects()` with PostProcessConfig
   - Show progress/status in UI

### Phase 4: AI Zoom (Research Complete)
**Approach 1: Smart Click-Based Zoom (Easiest)**
- Improve zoom trigger: Only zoom when 2+ clicks in 3 seconds (Cursorful pattern)
- Ignore dead zones (close buttons, scrollbars)
- Detect rapid clicks as single interaction

**Approach 2: Frame Diff Detection**
- Use `rendiff` or `diffimg-rs` crates
- Capture frames → grayscale → diff → threshold → find changed regions
- Trigger zoom on high-activity regions

**Approach 3: Cursor Velocity Heuristics**
- Zoom when cursor "settles" (velocity drops below threshold)
- Already tracking CursorMove events with timestamps

**Approach 4: Region of Interest (Advanced)**
- OpenCV edge detection via `opencv` crate
- Template matching for common UI patterns

### Future Enhancements
- macOS support (crabgrab integration)
- Audio recording (system audio, microphone)
- Custom hotkey configuration
- Video preview in dashboard
- Zoom effect preview before applying

## Commands
// turbo
```bash
# Run the app
dx serve --platform desktop
```

// turbo
```bash
# Build release
dx build --release --platform desktop
```

// turbo
```bash
# Check for errors
cargo check
```

## Notes
- Zoom effects are post-processing only (not live during recording)
- Events are saved as JSON with click positions and timestamps
- FFmpeg zoompan filter is used for smooth zoom effects
