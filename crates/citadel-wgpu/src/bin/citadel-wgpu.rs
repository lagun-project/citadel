//! High-performance 3D mesh visualization for Citadel topology.
//!
//! Controls:
//! - WASD: Move camera
//! - Space/Shift: Move up/down
//! - Right-click + mouse: Look around
//! - Scroll wheel: Adjust speed
//! - +/-: Increase/decrease speed
//! - Home: Reset camera to origin
//! - P: Toggle playback
//! - R: Reset playback
//! - [/]: Decrease/increase playback speed
//! - Escape: Quit
//!
//! Gamepad Controls:
//! - Left stick: Move camera
//! - Right stick: Look around
//! - LT/RT: Move down/up
//! - A/Start: Toggle playback
//! - B/Back: Reset playback
//! - LB/RB: Decrease/increase playback speed
//! - Y: Reset camera

use citadel_wgpu::Renderer;
use gilrs::{Axis, Button, Event as GilrsEvent, Gilrs};
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

const DEFAULT_NODE_COUNT: u32 = 100_000;
const DEADZONE: f32 = 0.15;

fn apply_deadzone(value: f32) -> f32 {
    if value.abs() < DEADZONE {
        0.0
    } else {
        (value - value.signum() * DEADZONE) / (1.0 - DEADZONE)
    }
}

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    gilrs: Option<Gilrs>,
    node_count: u32,
    last_frame: Instant,

    // Playback state
    playing: bool,
    playback_speed: f32,
    playback_frame: f32,
}

impl App {
    fn new(node_count: u32) -> Self {
        Self {
            window: None,
            renderer: None,
            gilrs: None,
            node_count,
            last_frame: Instant::now(),
            playing: false,
            playback_speed: 1000.0,
            playback_frame: 0.0,
        }
    }

    fn update_gamepad(&mut self) {
        let Some(gilrs) = &mut self.gilrs else {
            return;
        };
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        // Process gamepad events
        while let Some(GilrsEvent { id, event, .. }) = gilrs.next_event() {
            match event {
                gilrs::EventType::ButtonPressed(btn, _) => match btn {
                    Button::South | Button::Start => {
                        self.playing = !self.playing;
                        tracing::info!(
                            "Playback: {}",
                            if self.playing { "playing" } else { "paused" }
                        );
                    }
                    Button::East | Button::Select => {
                        self.playback_frame = 0.0;
                        if let Some(mesh) = &mut renderer.mesh_data {
                            mesh.set_visible(0);
                        }
                        tracing::info!("Playback reset");
                    }
                    Button::LeftTrigger => {
                        self.playback_speed = (self.playback_speed / 1.5).max(100.0);
                        tracing::info!("Playback speed: {:.0} nodes/s", self.playback_speed);
                    }
                    Button::RightTrigger => {
                        self.playback_speed = (self.playback_speed * 1.5).min(100000.0);
                        tracing::info!("Playback speed: {:.0} nodes/s", self.playback_speed);
                    }
                    Button::North => {
                        renderer.camera.reset();
                        tracing::info!("Camera reset");
                    }
                    Button::DPadUp => {
                        renderer.camera.speed *= 1.5;
                        tracing::info!("Speed: {:.0}", renderer.camera.speed);
                    }
                    Button::DPadDown => {
                        renderer.camera.speed /= 1.5;
                        tracing::info!("Speed: {:.0}", renderer.camera.speed);
                    }
                    _ => {}
                },
                _ => {}
            }
            tracing::debug!("Gamepad {:?} event: {:?}", id, event);
        }

        // Read current gamepad state
        for (_id, gamepad) in gilrs.gamepads() {
            if !gamepad.is_connected() {
                continue;
            }

            // Left stick for movement
            let move_x = apply_deadzone(gamepad.value(Axis::LeftStickX));
            let move_y = apply_deadzone(-gamepad.value(Axis::LeftStickY)); // Invert Y
            renderer.camera.set_gamepad_move(move_x, move_y);

            // Right stick for looking
            let look_x = apply_deadzone(gamepad.value(Axis::RightStickX));
            let look_y = apply_deadzone(gamepad.value(Axis::RightStickY));
            renderer.camera.set_gamepad_look(look_x, look_y);

            // Triggers for up/down (RT = up, LT = down)
            let trigger_up = gamepad.value(Axis::RightZ).max(0.0);
            let trigger_down = gamepad.value(Axis::LeftZ).max(0.0);
            renderer.camera.set_gamepad_triggers(trigger_up, trigger_down);

            // Only use first connected gamepad
            break;
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attrs = Window::default_attributes()
            .with_title(format!("Citadel Mesh - {} nodes", self.node_count))
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());
        self.window = Some(window.clone());

        // Initialize gilrs for gamepad support
        match Gilrs::new() {
            Ok(gilrs) => {
                for (_id, gamepad) in gilrs.gamepads() {
                    tracing::info!("Gamepad found: {} ({:?})", gamepad.name(), gamepad.power_info());
                }
                self.gilrs = Some(gilrs);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize gamepad support: {}", e);
            }
        }

        // Create renderer
        let mut renderer = pollster::block_on(Renderer::new(window));

        // Generate mesh
        tracing::info!("Generating mesh with {} nodes...", self.node_count);
        renderer.generate_mesh(self.node_count);

        // Start with all nodes visible
        if let Some(mesh) = &mut renderer.mesh_data {
            mesh.set_visible(self.node_count);
        }

        self.renderer = Some(renderer);
        self.last_frame = Instant::now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                renderer.resize(size);
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => {
                // Handle playback controls
                if state == ElementState::Pressed {
                    match key {
                        KeyCode::Escape => {
                            event_loop.exit();
                        }
                        KeyCode::KeyP => {
                            self.playing = !self.playing;
                            tracing::info!(
                                "Playback: {}",
                                if self.playing { "playing" } else { "paused" }
                            );
                        }
                        KeyCode::KeyR => {
                            self.playback_frame = 0.0;
                            if let Some(mesh) = &mut renderer.mesh_data {
                                mesh.set_visible(0);
                            }
                            tracing::info!("Playback reset");
                        }
                        KeyCode::BracketLeft => {
                            self.playback_speed = (self.playback_speed / 1.5).max(100.0);
                            tracing::info!("Playback speed: {:.0} nodes/s", self.playback_speed);
                        }
                        KeyCode::BracketRight => {
                            self.playback_speed = (self.playback_speed * 1.5).min(100000.0);
                            tracing::info!("Playback speed: {:.0} nodes/s", self.playback_speed);
                        }
                        _ => {}
                    }
                }

                // Forward to camera
                renderer.camera.handle_keyboard(key, state);
            }

            WindowEvent::MouseInput { button, state, .. } => {
                renderer.camera.handle_mouse_button(button, state);
            }

            WindowEvent::CursorMoved { position, .. } => {
                renderer.camera.handle_mouse_motion(position.x, position.y);
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                };
                renderer.camera.handle_scroll(scroll);
            }

            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last_frame).as_secs_f32();
                self.last_frame = now;

                // Update gamepad
                self.update_gamepad();

                // Update camera
                if let Some(renderer) = &mut self.renderer {
                    renderer.camera.update(dt);

                    // Update playback
                    if self.playing {
                        self.playback_frame += self.playback_speed * dt;
                        let visible = (self.playback_frame as u32).min(self.node_count);
                        if let Some(mesh) = &mut renderer.mesh_data {
                            mesh.set_visible(visible);
                        }

                        // Stop at end
                        if self.playback_frame >= self.node_count as f32 {
                            self.playing = false;
                        }
                    }

                    // Render
                    match renderer.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            renderer.resize(renderer.size());
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            tracing::error!("Out of GPU memory");
                            event_loop.exit();
                        }
                        Err(e) => {
                            tracing::warn!("Render error: {:?}", e);
                        }
                    }
                }

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }
}

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Parse node count from args
    let node_count = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_NODE_COUNT);

    tracing::info!("Citadel wgpu visualization");
    tracing::info!("Node count: {}", node_count);
    tracing::info!("Controls:");
    tracing::info!("  WASD - Move camera");
    tracing::info!("  Space/Shift - Up/Down");
    tracing::info!("  Right-click + mouse - Look around");
    tracing::info!("  Scroll wheel - Adjust speed");
    tracing::info!("  P - Toggle playback");
    tracing::info!("  R - Reset playback");
    tracing::info!("  [/] - Adjust playback speed");
    tracing::info!("  Home - Reset camera");
    tracing::info!("  Escape - Quit");
    tracing::info!("Gamepad:");
    tracing::info!("  Left stick - Move");
    tracing::info!("  Right stick - Look");
    tracing::info!("  LT/RT - Down/Up");
    tracing::info!("  A/Start - Toggle playback");
    tracing::info!("  B/Back - Reset playback");
    tracing::info!("  LB/RB - Playback speed");
    tracing::info!("  Y - Reset camera");
    tracing::info!("  DPad Up/Down - Speed");

    // Create event loop
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    // Run application
    let mut app = App::new(node_count);
    event_loop.run_app(&mut app).unwrap();
}
