use crate::{CircleCollider, Particle};
use glam::Vec2;

pub struct World {
    pub particles: Vec<Particle>,
    pub gravity: Vec2,
    pub colliders: Vec<CircleCollider>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            gravity: Vec2::new(0.0, -9.81),
            colliders: Vec::new(),
        }
    }

    pub fn add_circle_collider(&mut self, c: CircleCollider) {
        self.colliders.push(c);
    }

    pub fn add_particle(&mut self, p: Particle) -> usize {
        self.particles.push(p);
        self.particles.len() - 1 //stellen rückgabe des eingefügten particels
    }

    /// Applies gravity to all particles.
    pub fn apply_forces(&mut self) {
        for p in &mut self.particles {
            p.add_force(self.gravity);
        }
    }

    pub fn update_positions(&mut self, dt: f32) {
        for p in &mut self.particles {
            //dt : delta time Pixel pro sekunde nicht pro frame
            let temp = p.pos;
            let vel = p.pos - p.old_pos;

            p.pos = p.pos + vel + p.acc * dt * dt; //Verlet Formel : bewegt partikel an geschwindigkeit + beschleunigug
            p.old_pos = temp;

            // Reset acceleration for next frame
            p.acc = Vec2::ZERO;
        }
    }

    pub fn solve_collisions(&mut self) {
        for p in &mut self.particles {
            for c in &self.colliders {
                let dir = p.pos - c.center;
                let dist = dir.length();

                // Wenn Partikel außerhalb des Kreises ist wird nach innen geschoben
                if dist > c.radius {
                    let correction = dir.normalize() * c.radius;
                    p.pos = c.center + correction;
                }
            }
        }
    }
    //evtl noch step() fn einbauen ... step = „ein Simulationsschritt“ (forces → integration → collisions)
}

#[cfg(test)]
mod test {

    use super::*;
    use glam::Vec2;

    #[test]
    fn verlet_moves_particle_with_gravity() {
        let mut world = World::new();
        let id = world.add_particle(Particle::new(Vec2::ZERO));

        world.apply_forces();
        world.update_positions(1.0);
        let p = world.particles[id];
        let expected = Vec2::new(0.0, -9.81);

        assert!(
            (p.pos - expected).length() < 0.001,
            "Expected {:?}, got {:?}",
            expected,
            p.pos
        );
    }
}
