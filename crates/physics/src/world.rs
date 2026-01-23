use crate::{CircleCollider, Particle};
use glam::Vec2;
use std::collections::HashMap;

//Adding const to simply change the values if needed
const DEFAULT_GRAVITY: Vec2 = Vec2::new(0.0, 9.81);
const DEFAULT_PARTICLE_RADIUS: f32 = 6.0;
const DEFAULT_RESTITUTION: f32 = 0.96;
pub struct World {
    pub particles: Vec<Particle>,
    pub gravity: Vec2,
    pub colliders: Vec<CircleCollider>,
    pub particle_radius: f32,
    pub restitution: f32,
    pub fps: f32,
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
            particle_radius: DEFAULT_PARTICLE_RADIUS,
            restitution: DEFAULT_RESTITUTION,
            fps: 0.0,
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

    //komplexität n^2
    pub fn solve_particle_collisions(&mut self) {
        let n = self.particles.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let (left, right) = self.particles.split_at_mut(j);
                let a = &mut left[i];
                let b = &mut right[0];

                let min_dist = 2.0 * self.particle_radius;
                //falls unterschiedlich große partikel implementiert
                //let min_dist = a.radius + b.radius;
                let dir = b.pos - a.pos;
                let dist = dir.length();

                // overlap
                if dist < min_dist {
                    let normal = if dist > 0.0 {
                        // einheitsvektor von a nach b (normale) + dist != 0
                        dir / dist
                    } else {
                        Vec2::X // ausweichvektor für case dist = 0
                    };
                    let overlap = min_dist - dist;

                    let correction = normal * (overlap * 0.5);
                    a.pos -= correction;
                    b.pos += correction;

                    //falls unterschiedlich große partikel implementiert
                    //let total = a.radius + b.radius;
                    //let wa = b.radius / total; // a wird weniger bewegt, wenn a groß ist
                    //let wb = a.radius / total;
                    //a.pos -= normal * overlap * wa;
                    //b.pos += normal * overlap * wb;

                    // vel = pos - old_pos
                    let vel_a = a.pos - a.old_pos;
                    let vel_b = b.pos - b.old_pos;

                    let rel_vel = vel_b - vel_a;
                    let rel_normal_speed = rel_vel.dot(normal); // geschwindigkeit auf der kollisionsnormalen

                    // kleiner 0 bedeutet sie bewegen sich aufeinander zu -> bounce
                    if rel_normal_speed < 0.0 {
                        let bounce = -(1.0 + self.restitution) * rel_normal_speed * 0.5;
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

    pub fn solve_particle_collisions_grid(&mut self) {
        let min_dist = 2.0 * self.particle_radius;
        let cell_size = min_dist.max(1e-6);

        let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new(); //eine Zelle ist eine Liste von Partikeln

        for (idx, p) in self.particles.iter().enumerate() {
            //ordnet jedem Partikel eine Zelle zu
            let cx = (p.pos.x / cell_size).floor() as i32;
            let cy = (p.pos.y / cell_size).floor() as i32;
            grid.entry((cx, cy)).or_default().push(idx);
        }

        const OFFS: [(i32, i32); 5] = [(0, 0), (1, 0), (0, 1), (1, 1), (-1, 1)]; // doppelchecken vermeiden
        let mut pairs: Vec<(usize, usize)> = Vec::new();

        //vergelich von Zellen und identifizierne von vergleichs pairs
        for (&cell, indices) in grid.iter() {
            for (dx, dy) in OFFS {
                let ncell = (cell.0 + dx, cell.1 + dy);
                let Some(nindices) = grid.get(&ncell) else {
                    continue;
                };

                if dx == 0 && dy == 0 {
                    //paar in der gleichen Zelle werden in überprüfungsarray pairs gespeichert
                    for a in 0..indices.len() {
                        for b in (a + 1)..indices.len() {
                            let i = indices[a];
                            let j = indices[b];
                            if i < j {
                                // richtige reinfolge um doppelchecks zu vermeiden
                                pairs.push((i, j));
                            } else {
                                pairs.push((j, i));
                            }
                        }
                    }
                } else {
                    //paare zwischen Zelle und Nachbarzelle werden in überprüfungsarray pairs gespeichert
                    for &i in indices {
                        for &j in nindices {
                            if i == j {
                                continue;
                            }
                            if i < j {
                                pairs.push((i, j));
                            } else {
                                pairs.push((j, i));
                            }
                        }
                    }
                }
            }
        }
        // deterministisch + doppelte raus

        pairs.sort_unstable();
        pairs.dedup();

        //kollisionsberechnung für paare

        let restitution = self.restitution;

        for (i, j) in pairs {
            let (left, right) = self.particles.split_at_mut(j);
            let a = &mut left[i];
            let b = &mut right[0];

            let dir = b.pos - a.pos;
            let dist = dir.length();

            if dist < min_dist {
                let normal = if dist > 0.0 { dir / dist } else { Vec2::X };

                let overlap = min_dist - dist;
                let correction = normal * (overlap * 0.5);
                a.pos -= correction;
                b.pos += correction;

                let vel_a = a.pos - a.old_pos;
                let vel_b = b.pos - b.old_pos;

                let rel_vel = vel_b - vel_a;
                let rel_normal_speed = rel_vel.dot(normal);

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

    //ein "Simulationsschritt“ (forces → integration → collisions)
    pub fn step(&mut self, dt: f32) {
        self.apply_forces();
        self.update_positions(dt);
        self.solve_collisions();
        // self.solve_particle_collisions();
        self.solve_particle_collisions_grid();
    }

    //resets all particles
    pub fn clear(&mut self) {
        self.particles.clear();
        self.colliders.clear();
        self.gravity = DEFAULT_GRAVITY;
    }

    pub fn clear_particles(&mut self) {
        self.particles.clear()
    }

    pub fn clear_collider(&mut self) {
        self.colliders.clear();
    }

    // funktion für nearest neighbour search
    //moved the nearest neighbour search into physics
    pub fn find_nearest_particle(&mut self, mouse_pos: Vec2) -> Option<usize> {
        let mut nearest: Option<usize> = None;
        let mut nearest_dist2 = f32::MAX;

        for (i, p) in self.particles.iter().enumerate() {
            let d = p.pos - mouse_pos;
            let dist2 = d.length_squared();

            if dist2 < nearest_dist2 {
                nearest_dist2 = dist2;
                nearest = Some(i);
            }
        }
        nearest
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
    //Ai Unit Tests Gemini
    #[test]
    fn particles_separate_when_overlapping() {
        let mut world = World::new();

        let mut a = Particle::new(Vec2::new(0.0, 0.0));
        let mut b = Particle::new(Vec2::new(5.0, 0.0)); // overlap (min_dist = 12)

        a.old_pos = a.pos;
        b.old_pos = b.pos;

        world.add_particle(a);
        world.add_particle(b);

        world.solve_particle_collisions();

        let d = (world.particles[1].pos - world.particles[0].pos).length();
        assert!(d >= 12.0 - 1e-4, "Particles still overlap: d={}", d);
    }
    #[test]
    fn particles_do_not_move_when_not_overlapping() {
        let mut world = World::new();

        let mut a = Particle::new(Vec2::new(0.0, 0.0));
        let mut b = Particle::new(Vec2::new(20.0, 0.0)); // no overlap

        a.old_pos = a.pos;
        b.old_pos = b.pos;

        let a0 = a.pos;
        let b0 = b.pos;

        world.add_particle(a);
        world.add_particle(b);

        world.solve_particle_collisions();

        assert!((world.particles[0].pos - a0).length() < 1e-6);
        assert!((world.particles[1].pos - b0).length() < 1e-6);
    }
    #[test]
    fn particles_bounce_when_moving_towards_each_other() {
        let mut world = World::new();

        let mut a = Particle::new(Vec2::new(0.0, 0.0));
        a.old_pos = Vec2::new(-1.0, 0.0);

        let mut b = Particle::new(Vec2::new(11.0, 0.0)); // overlap
        b.old_pos = Vec2::new(12.0, 0.0);

        world.add_particle(a);
        world.add_particle(b);

        world.solve_particle_collisions();

        let p0 = world.particles[0];
        let p1 = world.particles[1];

        let dir = p1.pos - p0.pos;
        let dist = dir.length().max(1e-6);
        let normal = dir / dist;

        let vel_a = p0.pos - p0.old_pos;
        let vel_b = p1.pos - p1.old_pos;

        let rel = (vel_b - vel_a).dot(normal);

        assert!(rel >= -1e-4, "Still moving towards each other: rel={}", rel);
    }

    #[test]
    fn particles_with_same_position_get_separated() {
        let mut world = World::new();

        let mut a = Particle::new(Vec2::new(0.0, 0.0));
        let mut b = Particle::new(Vec2::new(0.0, 0.0)); // dist == 0

        a.old_pos = a.pos;
        b.old_pos = b.pos;

        world.add_particle(a);
        world.add_particle(b);

        world.solve_particle_collisions();

        let d = (world.particles[1].pos - world.particles[0].pos).length();
        assert!(d >= 12.0 - 1e-4, "dist==0 case not separated: d={}", d);
    }
    //Generated with ChatGPT5
    #[test]
    fn grid_particles_separate_when_overlapping() {
        let mut world = World::new();

        let mut a = Particle::new(Vec2::new(0.0, 0.0));
        let mut b = Particle::new(Vec2::new(5.0, 0.0)); // overlap (min_dist = 12)

        a.old_pos = a.pos;
        b.old_pos = b.pos;

        world.add_particle(a);
        world.add_particle(b);

        world.solve_particle_collisions_grid();

        let d = (world.particles[1].pos - world.particles[0].pos).length();
        assert!(d >= 12.0 - 1e-4, "Particles still overlap: d={}", d);
    }

    #[test]
    fn grid_particles_do_not_move_when_not_overlapping() {
        let mut world = World::new();

        let mut a = Particle::new(Vec2::new(0.0, 0.0));
        let mut b = Particle::new(Vec2::new(20.0, 0.0)); // no overlap

        a.old_pos = a.pos;
        b.old_pos = b.pos;

        let a0 = a.pos;
        let b0 = b.pos;

        world.add_particle(a);
        world.add_particle(b);

        world.solve_particle_collisions_grid();

        assert!((world.particles[0].pos - a0).length() < 1e-6);
        assert!((world.particles[1].pos - b0).length() < 1e-6);
    }

    #[test]
    fn grid_particles_with_same_position_get_separated() {
        let mut world = World::new();

        let mut a = Particle::new(Vec2::new(0.0, 0.0));
        let mut b = Particle::new(Vec2::new(0.0, 0.0)); // dist == 0

        a.old_pos = a.pos;
        b.old_pos = b.pos;

        world.add_particle(a);
        world.add_particle(b);

        world.solve_particle_collisions_grid();

        let d = (world.particles[1].pos - world.particles[0].pos).length();
        assert!(d >= 12.0 - 1e-4, "dist==0 case not separated: d={}", d);
    }
    #[test]
    fn grid_matches_naive_for_small_case() {
        let mut w1 = World::new();
        let mut w2 = World::new();

        // Mix aus Kollision / keine Kollision / diagonal
        let mut p0 = Particle::new(Vec2::new(0.0, 0.0));
        let mut p1 = Particle::new(Vec2::new(11.5, 0.0)); // overlap (min_dist=12)
        let mut p2 = Particle::new(Vec2::new(30.0, 0.0)); // no overlap
        let mut p3 = Particle::new(Vec2::new(11.5, 11.5)); // diagonal neighbor

        for p in [&mut p0, &mut p1, &mut p2, &mut p3] {
            p.old_pos = p.pos; // keine Bewegung, nur overlap resolution
        }

        for p in [p0, p1, p2, p3] {
            w1.add_particle(p);
            w2.add_particle(p);
        }

        w1.solve_particle_collisions(); // naive
        w2.solve_particle_collisions_grid(); // grid

        for i in 0..w1.particles.len() {
            let d = (w1.particles[i].pos - w2.particles[i].pos).length();
            assert!(d < 1e-5, "Mismatch at {}: d={}", i, d);
        }
    }
}
