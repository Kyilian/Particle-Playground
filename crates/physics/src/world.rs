use crate::{CircleCollider, Particle};
use glam::Vec2;

//Adding const to simply change the values if needed
const DEFAULT_GRAVITY: Vec2 = Vec2::new(0.0, 9.81);

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
        let restitution = 0.96; //sehr bouncy verhalten..0.3 weniger bis garkein bounch

        for p in &mut self.particles {
            for c in &self.colliders {
                let dir = p.pos - c.center; //testen ob der partikel auserhalb der border ist
                let dist = dir.length();

                if dist > c.radius {
                    let normal = dir.normalize();

                    p.pos = c.center + normal * c.radius; // Position auf Rand korrigieren -Position Projection

                    let vel = p.pos - p.old_pos; // aufschlags geschwindigkeit brechenet
                    let reflected = vel - 2.0 * vel.dot(normal) * normal; //abprallen mit korrekt reflektierter Richtung
                    let reflected = reflected * restitution; //  Energieverlust einberechen
                    p.old_pos = p.pos - reflected; // Verlet- trick, partikel weiß er ist abgeprallt
                }
            }
        }
    }

    pub fn solve_particle_collisions(&mut self) {
        let restitution = 0.96;
        let radius = 6.0;
        let min_dist = 2.0 * radius;

        let n = self.particles.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let (left, right) = self.particles.split_at_mut(j);
                let a = &mut left[i];
                let b = &mut right[0];

                let dir = b.pos - a.pos;
                let dist = dir.length();

                // overlap
                if dist < min_dist {
                    let normal = dir / dist; // einheitsvektor von a nach b (normale)
                    let overlap = min_dist - dist;

                    let correction = normal * (overlap * 0.5);
                    a.pos -= correction;
                    b.pos += correction;

                    // vel = pos - old_pos
                    let vel_a = a.pos - a.old_pos;
                    let vel_b = b.pos - b.old_pos;

                    let rel_vel = vel_b - vel_a;
                    let rel_normal_speed = rel_vel.dot(normal); // geschwindigkeit auf der kollisionsnormalen

                    // kleiner 0 bedeutet sie bewegen sich aufeinander zu -> bounce
                    if rel_normal_speed < 0.0 {
                        let bounce = -(1.0 + restitution) * rel_normal_speed * 0.5;
                        let bounce_vec = normal * bounce;

                        let vel_a2 = vel_a - bounce_vec;
                        let vel_b2 = vel_b + bounce_vec;

                        a.old_pos = a.pos - vel_a2;
                        b.old_pos = b.pos - vel_b2;
                    }
                }
            }
        }
    }

    //ein "Simulationsschritt“ (forces → integration → collisions)
    pub fn step(&mut self, dt: f32) {
        self.apply_forces();
        self.update_positions(dt);
        self.solve_collisions();
        self.solve_particle_collisions();
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

        world.step(1.0);
        let p = world.particles[id];
        let expected = Vec2::new(0.0, 9.81);

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
        assert_eq!(world.gravity.y, 9.81, "Gravity sollte wieder Standard sein");
    }
    #[test]
    fn particle_stays_inside_circle() {
        let mut world = World::new();

        world.add_circle_collider(CircleCollider {
            center: Vec2::ZERO,
            radius: 10.0,
        });

        let id = world.add_particle(Particle::new(Vec2::new(20.0, 0.0)));

        world.solve_collisions();

        let p = world.particles[id];
        assert!(
            p.pos.length() <= 10.0 + 1e-4,
            "Particle escaped the boundary"
        );
    }
    #[test]
    fn particle_bounces_off_circle() {
        let mut world = World::new();

        let collider = CircleCollider {
            center: Vec2::ZERO,
            radius: 10.0,
        };
        world.add_circle_collider(collider);

        // Partikel außerhalb, kam von innen → echte Kollision
        let mut p = Particle::new(Vec2::new(12.0, 0.0));
        p.old_pos = Vec2::new(9.0, 0.0);

        let id = world.add_particle(p);

        world.solve_collisions();

        let p = &world.particles[id];

        let vel = p.pos - p.old_pos;
        let normal = (p.pos - collider.center).normalize();

        // Nach der Kollision darf das Partikel nicht weiter nach außen laufen
        assert!(
            vel.dot(normal) <= 0.0,
            "Particle velocity still points outward: vel={:?}, normal={:?}",
            vel,
            normal
        );
    }
}
