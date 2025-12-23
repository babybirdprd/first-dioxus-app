//! Smart camera system for auto-zoom and pan during recording
//!
//! Implements three zoom modes:
//! 1. FollowCursor - Smoothly follows mouse position
//! 2. ClickToZoom - Zooms in on click, zooms out after idle
//! 3. SmartAI - (Future) AI-powered region detection

use device_query::{DeviceQuery, DeviceState, MouseState};
use std::time::{Duration, Instant};

/// Represents a 2D point
#[derive(Clone, Copy, Debug, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Linear interpolation between two points
    pub fn lerp(&self, target: &Point, t: f32) -> Point {
        Point {
            x: self.x + (target.x - self.x) * t,
            y: self.y + (target.y - self.y) * t,
        }
    }

    /// Smooth interpolation with easing
    pub fn smooth_lerp(&self, target: &Point, t: f32) -> Point {
        // Ease-out cubic for smooth deceleration
        let eased_t = 1.0 - (1.0 - t).powi(3);
        self.lerp(target, eased_t)
    }
}

/// Camera state for virtual zoom/pan
#[derive(Clone, Debug)]
pub struct Camera {
    /// Current camera center position
    pub position: Point,
    /// Target position to move towards
    pub target: Point,
    /// Current zoom level (1.0 = no zoom, 2.0 = 2x zoom)
    pub zoom: f32,
    /// Target zoom level
    pub target_zoom: f32,
    /// Smoothing factor (0.0-1.0, higher = faster)
    pub smoothing: f32,
    /// Screen dimensions
    pub screen_width: f32,
    pub screen_height: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Point::default(),
            target: Point::default(),
            zoom: 1.0,
            target_zoom: 1.0,
            smoothing: 0.1,
            screen_width: 1920.0,
            screen_height: 1080.0,
        }
    }
}

impl Camera {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        let center = Point::new(screen_width / 2.0, screen_height / 2.0);
        Self {
            position: center,
            target: center,
            screen_width,
            screen_height,
            ..Default::default()
        }
    }

    /// Update camera position with smoothing
    pub fn update(&mut self, delta_time: f32) {
        // Smooth position interpolation
        let t = (self.smoothing * delta_time * 60.0).min(1.0);
        self.position = self.position.smooth_lerp(&self.target, t);

        // Smooth zoom interpolation
        self.zoom += (self.target_zoom - self.zoom) * t;

        // Clamp position to keep view within screen bounds
        self.clamp_position();
    }

    /// Set target to follow mouse cursor
    pub fn follow_cursor(&mut self, mouse_x: f32, mouse_y: f32) {
        self.target = Point::new(mouse_x, mouse_y);
    }

    /// Zoom in at a specific point
    pub fn zoom_at(&mut self, x: f32, y: f32, zoom_level: f32) {
        self.target = Point::new(x, y);
        self.target_zoom = zoom_level;
    }

    /// Reset zoom to default
    pub fn reset_zoom(&mut self) {
        self.target_zoom = 1.0;
        self.target = Point::new(self.screen_width / 2.0, self.screen_height / 2.0);
    }

    /// Get the visible viewport rectangle
    pub fn get_viewport(&self) -> (f32, f32, f32, f32) {
        let view_width = self.screen_width / self.zoom;
        let view_height = self.screen_height / self.zoom;

        let left = self.position.x - view_width / 2.0;
        let top = self.position.y - view_height / 2.0;

        (left.max(0.0), top.max(0.0), view_width, view_height)
    }

    fn clamp_position(&mut self) {
        let view_width = self.screen_width / self.zoom;
        let view_height = self.screen_height / self.zoom;

        let min_x = view_width / 2.0;
        let max_x = self.screen_width - view_width / 2.0;
        let min_y = view_height / 2.0;
        let max_y = self.screen_height - view_height / 2.0;

        self.position.x = self.position.x.clamp(min_x, max_x);
        self.position.y = self.position.y.clamp(min_y, max_y);
    }
}

/// Manages zoom behavior based on mode
pub struct ZoomController {
    pub camera: Camera,
    device_state: DeviceState,
    last_click_time: Instant,
    last_mouse_state: MouseState,
    idle_timeout: Duration,
    click_zoom_level: f32,
}

impl ZoomController {
    pub fn new(screen_width: f32, screen_height: f32, zoom_level: f32) -> Self {
        Self {
            camera: Camera::new(screen_width, screen_height),
            device_state: DeviceState::new(),
            last_click_time: Instant::now(),
            last_mouse_state: MouseState {
                coords: (0, 0),
                button_pressed: vec![],
            },
            idle_timeout: Duration::from_secs(2),
            click_zoom_level: zoom_level,
        }
    }

    /// Update for FollowCursor mode
    pub fn update_follow_cursor(&mut self, delta_time: f32) {
        let mouse = self.device_state.get_mouse();
        let (x, y) = mouse.coords;

        self.camera.follow_cursor(x as f32, y as f32);
        self.camera.target_zoom = self.click_zoom_level;
        self.camera.update(delta_time);
    }

    /// Update for ClickToZoom mode
    pub fn update_click_to_zoom(&mut self, delta_time: f32) {
        let mouse = self.device_state.get_mouse();
        let (x, y) = mouse.coords;

        // Detect new click
        let is_clicking = !mouse.button_pressed.is_empty();
        let was_clicking = !self.last_mouse_state.button_pressed.is_empty();

        if is_clicking && !was_clicking {
            // New click - zoom in
            self.camera
                .zoom_at(x as f32, y as f32, self.click_zoom_level);
            self.last_click_time = Instant::now();
        }

        // Check for idle timeout to zoom out
        if self.last_click_time.elapsed() > self.idle_timeout && self.camera.target_zoom > 1.0 {
            self.camera.reset_zoom();
        }

        self.last_mouse_state = mouse;
        self.camera.update(delta_time);
    }

    /// Get current camera viewport for frame cropping
    pub fn get_viewport(&self) -> (f32, f32, f32, f32) {
        self.camera.get_viewport()
    }
}
