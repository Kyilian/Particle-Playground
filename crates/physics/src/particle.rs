use glam::Vec2;

pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
}

impl Particle {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            vel: Vec2::ZERO, //Zero for 1. weeks tests-> velocity in week2
        }
    }
}
