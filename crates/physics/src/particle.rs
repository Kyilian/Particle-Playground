use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub pos: Vec2,
    pub old_pos: Vec2,
    pub acc: Vec2, // force z.b gravity
}

impl Particle {
    /// Creates a new particle at the given position.
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            old_pos: pos,
            acc: Vec2::ZERO,
        }
    }

    pub fn add_force(&mut self, force: Vec2) {
        self.acc += force;
    }
}
