use crate::{CircleCollider, Particle};
use glam::Vec2;

//Adding const to simply change the values if needed
const DEFAULT_GRAVITY: Vec2 = Vec2::new(0.0, -9.81);

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
            gravity: DEFAULT_GRAVITY,
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
    //ein "Simulationsschritt“ (forces → integration → collisions)
    pub fn step(&mut self, dt: f32) {
        self.apply_forces();
        self.update_positions(dt);
        self.solve_collisions();
    }

    //resets all particles
    pub fn clear(&mut self) {
        self.particles.clear();
        self.colliders.clear();
        self.gravity = DEFAULT_GRAVITY;
    }
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

    #[test]
    fn test_world_clear_resets_everything() {
        let mut world = World::new();
        world.add_particle(Particle {
            pos: (Vec2::ZERO),
            old_pos: (Vec2::ZERO),
            acc: (Vec2::ZERO),
        });
        world.add_circle_collider(CircleCollider {
            center: Vec2::ZERO,
            radius: 10.0,
        });
        world.gravity = Vec2::new(100.0, 100.0);

        world.clear();

        assert_eq!(world.particles.len(), 0, "Partikel sollten weg sein");
        assert_eq!(world.colliders.len(), 0, "Collider sollten weg sein");
        assert_eq!(
            world.gravity.y, -9.81,
            "Gravity sollte wieder Standard sein"
        );
    }
}
