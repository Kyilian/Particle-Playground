use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{Key, NamedKey};

pub struct InputState {
    pub quit_requested: bool,
    pub cursor_pos: (f32, f32),
    pub mouse_left_down: bool,
    pub mouse_right_down: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            quit_requested: false,
            cursor_pos: (0.0, 0.0),
            mouse_left_down: false,
            mouse_right_down: false,
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            // mausosition bei move speichern
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.x as f32, position.y as f32);
            }

            // links-/rechtsklick
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

            _ => {}
        }
    }
}
