use glam::Vec2;

///simple circle collider for MVP

#[derive(Debug, Clone, Copy)]
pub struct CircleCollider {
    pub center: Vec2,
    pub radius: f32,
}

impl CircleCollider {
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self { center, radius }
    }
}
