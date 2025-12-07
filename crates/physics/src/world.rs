//use  glam::Vec2;
use crate::Particle;

pub struct World {
    pub particles: Vec<Particle>,
}

impl World {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
        }
    }

    pub fn add_particle(&mut self, p: Particle) -> usize {
        self.particles.push(p);
        self.particles.len() - 1 //stellen rückgabe des eingefügten particels
    }

    pub fn update_positions(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.pos += p.vel * dt; //Pixel pro sekunde nicht pro frame
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::Particle;
    use glam::Vec2;

    #[test]
    fn test_1_particle_moves() {
        let mut world = World::new();
        let id = world.add_particle(Particle::new(Vec2::new(0.0, 0.0)));
        world.particles[id].vel = Vec2::new(1.0, 1.0);

        world.update_positions(1.0);
        assert_eq!(world.particles[id].pos, Vec2::new(1.0, 1.0));
    }
}
