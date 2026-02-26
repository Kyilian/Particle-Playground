use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{Key, NamedKey};

pub struct InputState {
    pub quit_requested: bool,
    pub cursor_pos: (f32, f32),
    pub mouse_left_down: bool,
    pub mouse_right_down: bool,
    pub scroll_delta: f32,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            quit_requested: false,
            cursor_pos: (0.0, 0.0),
            mouse_left_down: false,
            mouse_right_down: false,
            scroll_delta: 0.0,
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            // save mouse position on move
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.x as f32, position.y as f32);
            }

            // left-/rightclick
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = *state == ElementState::Pressed;
                match button {
                    MouseButton::Left => self.mouse_left_down = pressed,
                    MouseButton::Right => self.mouse_right_down = pressed,
                    _ => {}
                }
            }

            // esc -> quit
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let Key::Named(NamedKey::Escape) = event.logical_key {
                        self.quit_requested = true;
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => {
                    self.scroll_delta = *y;
                }
                winit::event::MouseScrollDelta::PixelDelta(pos) => {
                    self.scroll_delta = (pos.y as f32 / 10.0);
                }
            },
            _ => {}
        }
    }
}
