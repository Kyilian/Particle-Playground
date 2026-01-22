use glam::Vec2;

#[derive(Clone, Copy)]
pub struct RectCollider {
    pub center: Vec2,
    pub width: f32,
    pub height: f32,
}

impl RectCollider {
    pub fn new(center: Vec2, width: f32, height: f32) -> Self {
        return Self {
            center,
            width,
            height,
        };
    }
}
