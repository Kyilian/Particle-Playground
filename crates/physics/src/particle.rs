use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub pos: Vec2,
    pub old_pos: Vec2,
    pub acc: Vec2, // force z.b gravity
    pub mass: f32,
    pub radius: f32,
    pub is_magnet: bool,
    pub color: [f32; 4],
}

impl Particle {
    /// Creates a new particle at the given position.
    pub fn new(pos: Vec2, radius: f32) -> Self {
        Self {
            pos,
            old_pos: pos,
            acc: Vec2::ZERO,
            mass: 10.0,
            radius: radius,
            is_magnet: false,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn add_force(&mut self, force: Vec2) {
        self.acc += force;
    }

    pub fn new_with_mass(pos: Vec2, mass: f32, radius: f32) -> Self {
        Self {
            pos,
            old_pos: pos,
            acc: Vec2::ZERO,
            mass: mass,
            radius: radius,
            is_magnet: false,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}
