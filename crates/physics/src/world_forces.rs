use crate::quadtree::Quadrant;
use crate::World;

impl World {
    /// Applies gravity to all particles.
    pub fn apply_forces(&mut self) {
        for p in &mut self.particles {
            p.add_force(self.gravity);
        }
    }

    //O(n^2) gravity calculation for small number of particles
    //Added an optimized version with Barnes-Hut algorithm
    pub fn apply_gravity(&mut self) {
        let g = 4.0; // Variable to apply force
        let softening = 500.0; // to prevent bounce effect

        let n = self.particles.len();

        for i in 0..n {
            for j in (i + 1)..n {
                let (left, right) = self.particles.split_at_mut(j);
                let p1 = &mut left[i];
                let p2 = &mut right[0];

                let direction = p2.pos - p1.pos;
                let dist_sq = direction.length_squared();

                // Newtons third law
                let gravity_force = g / (dist_sq + softening);
                let dir_norm = direction.normalize_or_zero();

                // including mass to the acceleration
                p1.acc += dir_norm * (gravity_force * p2.mass);
                p2.acc -= dir_norm * (gravity_force * p1.mass);
            }
        }
    }

    pub fn apply_magnets(&mut self) {
        for m in &self.magnets {
            for p in &mut self.particles {
                let direction = m.pos - p.pos;
                let dist_sq = direction.length_squared();
                let radius_sq = m.radius * m.radius;

                if dist_sq < radius_sq && dist_sq > 0.01 {
                    let dist = dist_sq.sqrt();
                    let normalize_dir = direction / dist;

                    let force = (m.strength * (1.0 - (dist / m.radius))).clamp(-1000.0, 1000.0);

                    p.acc += normalize_dir * force;
                }
            }
        }
    }

    //Optimized gravity calculation using Barnes-Hut algorithm, O(n log n)
    pub fn apply_gravity_barnes_hut(&mut self) {
        if self.particles.len() < 2 {
            return;
        }
        //the gravity calculation is done by a quadtree, so we dont have to calculate the force between every particle,
        //but can approximate the force of distant particles by treating them as a single mass at their center of mass
        let root_quadrant = Quadrant::new(&self.particles);

        self.quadtree.clear(root_quadrant);

        for p in &self.particles {
            self.quadtree.insert(p.pos, p.mass);
        }

        let gravity = self.gravity.y; //takes the gravity from the scene, so we can change it in runtime

        self.quadtree.propagate();

        for particle in &mut self.particles {
            particle.acc = self.quadtree.acc(particle.pos) * gravity;
        }
    }
}
