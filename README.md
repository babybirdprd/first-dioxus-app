# DemoRecorder

A high-performance screen recorder for developers and creators, featuring smart auto-zoom, cinematic motion blur, and professional GPU-accelerated post-processing. Inspired by Screen Studio, Motionik, and FocuSee.

## ✨ Features

- **🎯 Smart Auto-Zoom**: Automatically zooms in on mouse clicks and follows your cursor.
- **🎥 Magnetic Camera**: Cinematic easing and smooth panning between points of interest.
- **🌪️ Motion Blur**: High-quality cinematic motion blur for all transitions and cursor movements.
- **🛠️ GPU Engine**: WGPU-powered rendering pipeline for 4K exports at lightning speed.
- **🌈 Modern Aesthetics**: Customizable backgrounds, drop shadows, and rounded corners.

## 📊 Feature Matrix

| Category | Feature | DemoRecorder | Screen Studio | Motionik | FocuSee |
| :--- | :--- | :---: | :---: | :---: | :---: |
| **System** | Platform | Windows (Active) | macOS Only | Win/macOS | Win/macOS |
| | GPU Acceleration | ⏳ (In Progress) | ✅ | ✅ | ✅ |
| **Camera** | Auto-Zoom on Click | ✅ | ✅ | ✅ | ✅ |
| | Follow Cursor | ✅ | ✅ | ✅ | ✅ |
| | Magnetic Panning | ⏳ (In Progress) | ✅ | ✅ | ✅ |
| | Manual Keyframes | ❌ | ✅ | ✅ | ✅ |
| **Effects** | Cinematic Motion Blur | ⏳ (Planned) | ✅ | ✅ | ✅ |
| | Click Ripples/Spotlight| ❌ | ✅ | ✅ | ✅ |
| | Backgrounds/Shadows | ❌ | ✅ | ✅ | ✅ |
| **Audio/AI** | AI Subtitles/Captions | ❌ | ⏳ (Planned) | ✅ | ✅ |
| | Silence Removal | ❌ | ❌ | ❌ | ✅ |
| | Audio Enhancement | ❌ | ⏳ (Planned) | ✅ | ✅ |
| **Export** | 4K 60fps | ✅ | ✅ | ✅ | ✅ |
| | GIF Export | ❌ | ✅ | ✅ | ✅ |
| | Export Presets | ❌ | ✅ | ✅ | ✅ |

## 🚀 Roadmap

### Phase 3: GPU Engine (Current)
- [ ] **WGPU Pipeline**: Move all rendering to the GPU for real-time export performance.
- [ ] **Magnetic Camera**: Smooth interpolation and panning between click positions.
- [ ] **Advanced Filtering**: Bicubic and Lanczos resampling for pixel-perfect zooms.

### Phase 4: Cinematic Polish
- [ ] **Motion Blur**: Implementation of velocity-aware motion blur.
- [ ] **Beautify**: Customizable backgrounds (gradients/wallpapers), shadows, and corner rounding.
- [ ] **Cursor Effects**: Click ripples, spotlight effects, and cursor smoothing.

### Phase 5: AI & Smart Features
- [ ] **Smart Trim**: Automatically remove long silences or static sections.
- [ ] **AI Voice**: Microphone noise reduction and enhancement.
- [ ] **Captions**: Generate and burn-in subtitles automatically.

## 🛠️ Getting Started

1. **Serve the App**: `dx serve --platform desktop`
2. **Record**: Press `Ctrl+Shift+F9` to toggle recording.
3. **Capture**: Choose between "Primary Monitor" or "Foreground Window" in Settings.
4. **Process**: Go to Dashboard and click "🔍 Zoom" on any recording.

---
Built with [Dioxus](https://dioxus.rs) and [Rust](https://rust-lang.org).
