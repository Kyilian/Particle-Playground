use crate::{CircleCollider, Particle, RectCollider};
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
    pub rect_colliders: Vec<RectCollider>,

    pub WINDOW_WIDTH: u32,
    pub WINDOW_HEIGHT: u32,
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
            rect_colliders: Vec::new(),
            WINDOW_HEIGHT: 400,
            WINDOW_WIDTH: 600,
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
        self.solve_rect_collisions();
    }

    pub fn nbody_step(&mut self, dt: f32) {
        self.apply_gravity();
        self.update_positions(dt);
        self.solve_rect_collisions();
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

    pub fn add_rect_collider(&mut self, rc: RectCollider) {
        self.rect_colliders.push(rc);
    }

    pub fn clear_rect_collider(&mut self) {
        self.rect_colliders.clear();
    }

    pub fn solve_rect_collisions(&mut self) {
        let r = self.particle_radius;
        let restitution = self.restitution;

        let corner_damping = 0.7;

        for rc in &self.rect_colliders {
            let half_w = rc.width / 2.0;
            let half_h = rc.height / 2.0;

            let min_x = rc.center.x - half_w;
            let max_x = rc.center.x + half_w;
            let min_y = rc.center.y - half_h;
            let max_y = rc.center.y + half_h;

            for p in &mut self.particles {
                let mut hit_x = false;
                let mut hit_y = false;

                if p.pos.x + r > max_x {
                    p.pos.x = max_x - r;
                    hit_x = true;
                } else if p.pos.x - r < min_x {
                    p.pos.x = min_x + r;
                    hit_x = true;
                }

                if p.pos.y + r > max_y {
                    p.pos.y = max_y - r;
                    hit_y = true;
                } else if p.pos.y - r < min_y {
                    p.pos.y = min_y + r;
                    hit_y = true;
                }

                if hit_x || hit_y {
                    let mut vel = p.pos - p.old_pos;

                    if hit_x && hit_y {
                        vel.x = -vel.x;
                        vel.y = -vel.y;

                        vel *= corner_damping;

                        //random direction bounce so it dont get stuck in a corner
                        let noise = (p.pos.x + p.pos.y).sin() * 0.1;
                        vel.x += noise;
                    } else {
                        //normal wall collide logic
                        if hit_x {
                            vel.x = -vel.x * restitution;
                            vel.y *= 0.99;
                        }
                        if hit_y {
                            vel.y = -vel.y * restitution;
                            vel.x *= 0.99;
                        }
                    }

                    // Verlet-Update anwenden
                    p.old_pos = p.pos - vel;
                }
            }
        }
    }

    //simple O(n^2) gravity apply, maybe use Barnes Hut later
    //Gravity for N-Body Scene
    pub fn apply_gravity(&mut self) {
        let g = 4.0; // Variable to apply force
        let softening = 500.0; //to prevent bounce effect

        let n = self.particles.len();

        for i in 0..n {
            for j in (i + 1)..n {
                let (left, right) = self.particles.split_at_mut(j);
                let p1 = &mut left[i];
                let p2 = &mut right[0];

                let direction = p2.pos - p1.pos;
                let dist_sq = direction.length_squared();

                //Newtons third law
                let gravity_force = g / (dist_sq + softening);
                let dir_norm = direction.normalize_or_zero();

                //including mass to the acceleration
                p1.acc += dir_norm * (gravity_force * p2.mass);
                p2.acc -= dir_norm * (gravity_force * p1.mass);
            }
        }
    }
}
