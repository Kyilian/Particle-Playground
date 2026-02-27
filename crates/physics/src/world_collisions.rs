use crate::World;
use glam::Vec2;
use std::collections::HashMap;

impl World {
    pub fn solve_collisions(&mut self) {
        let restitution = 0.96; // very bouncy, 0.3 less till no bounce

        for p in &mut self.particles {
            if p.is_magnet {
                continue;
            }
            for c in &self.colliders {
                let dir = p.pos - c.center; //test if the particles if out of bounds
                let dist = dir.length();

                if dist > c.radius {
                    let normal = dir.normalize();

                    p.pos = c.center + normal * c.radius; // fix position on border -position projection

                    let vel = p.pos - p.old_pos; // calculate colission speed
                    let reflected = vel - 2.0 * vel.dot(normal) * normal; // bounce with correct direction
                    let reflected = reflected * restitution; // include energyloss from bounce
                    p.old_pos = p.pos - reflected; // Verlet- trick, particle knows it bounced
                }
            }
        }
    }

    //complexity n^2
    pub fn solve_particle_collisions(&mut self) {
        let n = self.particles.len();
        for i in 0..n {
            if self.particles[i].is_magnet {
                continue;
            }
            for j in (i + 1)..n {
                if self.particles[j].is_magnet {
                    continue;
                }
                let (left, right) = self.particles.split_at_mut(j);
                let a = &mut left[i];
                let b = &mut right[0];

                // incase differently sized particles are implemented
                let min_dist = a.radius + b.radius;
                let dir = b.pos - a.pos;
                let dist = dir.length();

                // overlap
                if dist < min_dist {
                    let normal = if dist > 0.0 {
                        // unitvectors from a to b (normal) + dist != 0
                        dir / dist
                    } else {
                        Vec2::X // dodging vector for case dist = 0
                    };
                    let overlap = min_dist - dist;

                    // incase differently sized particles are implemented
                    let total = a.radius + b.radius;
                    let wa = b.radius / total; // a is moved less, if a is big
                    let wb = a.radius / total;
                    a.pos -= normal * overlap * wa;
                    b.pos += normal * overlap * wb;

                    // vel = pos - old_pos
                    let vel_a = a.pos - a.old_pos;
                    let vel_b = b.pos - b.old_pos;

                    let rel_vel = vel_b - vel_a;
                    let rel_normal_speed = rel_vel.dot(normal); // movementspeed on the normal colission

                    // smaller than 0 means they are moving towards each other -> bounce
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
        let max_radius = self
            .particles
            .iter()
            .map(|p| p.radius)
            .fold(self.particle_radius, f32::max);

        let cell_size = (max_radius * 2.0).max(1e-6);

        let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new(); // each gridsquare is a list of particles

        for (idx, p) in self.particles.iter().enumerate() {
            // assigns each particle a gridsquare
            let cx = (p.pos.x / cell_size).floor() as i32;
            let cy = (p.pos.y / cell_size).floor() as i32;
            grid.entry((cx, cy)).or_default().push(idx);
        }

        const OFFS: [(i32, i32); 5] = [(0, 0), (1, 0), (0, 1), (1, 1), (-1, 1)]; // avoid double checking
        let mut pairs: Vec<(usize, usize)> = Vec::new();

        // comparison of gridsquares and identification of comparison pairs
        for (&cell, indices) in grid.iter() {
            for (dx, dy) in OFFS {
                let ncell = (cell.0 + dx, cell.1 + dy);
                let Some(nindices) = grid.get(&ncell) else {
                    continue;
                };

                if dx == 0 && dy == 0 {
                    // pairs in the same gridsquare get saved in pairs for checking array
                    for a in 0..indices.len() {
                        for b in (a + 1)..indices.len() {
                            let i = indices[a];
                            let j = indices[b];
                            if i < j {
                                // correct order to avoid doublechecks
                                pairs.push((i, j));
                            } else {
                                pairs.push((j, i));
                            }
                        }
                    }
                } else {
                    // pairs between Gridsquare and neighbours gridsquare get saved in checking array
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
        // deterministic + remove duplications

        pairs.sort_unstable();
        pairs.dedup();

        // collision calculation for pairs

        let restitution = self.restitution;

        for (i, j) in pairs {
            let (left, right) = self.particles.split_at_mut(j);
            let a = &mut left[i];
            let b = &mut right[0];

            if a.is_magnet || b.is_magnet {
                continue;
            }

            let dir = b.pos - a.pos;
            let dist = dir.length();
            let min_dist = a.radius + b.radius;

            if dist < min_dist {
                let normal = if dist > 0.0 { dir / dist } else { Vec2::X };

                let overlap = min_dist - dist;

                let total_radius = a.radius + b.radius;
                let wa = b.radius / total_radius;
                let wb = a.radius / total_radius;

                a.pos -= normal * overlap * wa;
                b.pos += normal * overlap * wb;

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

                        // random direction bounce so it doesn't get stuck in a corner
                        let noise = (p.pos.x + p.pos.y).sin() * 0.1;
                        vel.x += noise;
                    } else {
                        // normal wall collide logic
                        if hit_x {
                            vel.x = -vel.x * restitution;
                            vel.y *= 0.99;
                        }
                        if hit_y {
                            vel.y = -vel.y * restitution;
                            vel.x *= 0.99;
                        }
                    }

                    // apply Verlet-update
                    p.old_pos = p.pos - vel;
                }
            }
        }
    }
}
