use crate::galaxy;
pub use crate::neighbor_grid::NeighborGrid;
use crate::quadtree::Quadtree;
use crate::{CircleCollider, Particle, RectCollider};
use glam::Vec2;
use rayon::prelude::*;

//Adding const to simply change the values if needed
const DEFAULT_GRAVITY: Vec2 = Vec2::new(0.0, 9.81);
const DEFAULT_PARTICLE_RADIUS: f32 = 6.0;
const DEFAULT_RESTITUTION: f32 = 0.96;

pub struct Magnet {
    pub pos: Vec2,
    pub strength: f32,
    pub radius: f32,
}
pub struct World {
    pub particles: Vec<Particle>,
    pub gravity: Vec2,
    pub colliders: Vec<CircleCollider>,
    pub particle_radius: f32,
    pub restitution: f32,
    pub fps: f32,
    pub rect_colliders: Vec<RectCollider>,
    pub magnets: Vec<Magnet>,

    pub quadtree: Quadtree,

    pub window_width: u32,
    pub window_height: u32,
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
            magnets: Vec::new(),
            window_height: 400,
            window_width: 600,

            quadtree: Quadtree::new(0.5, 100.0),
        }
    }

    pub fn add_circle_collider(&mut self, c: CircleCollider) {
        self.colliders.push(c);
    }

    pub fn add_particle(&mut self, p: Particle) -> usize {
        self.particles.push(p);
        self.particles.len() - 1 //stellen rückgabe des eingefügten particels
    }

    pub fn update_positions(&mut self, dt: f32) {
        self.particles.par_iter_mut().for_each(|p| {
            //Implemented the rayon library enable parallelism in simple loops
            if p.is_magnet {
                // magnets dont move
                p.old_pos = p.pos;
                return;
            }
            //dt : delta time Pixel pro sekunde nicht pro frame
            let temp = p.pos;
            let vel = p.pos - p.old_pos;

            p.pos = p.pos + vel + p.acc * dt * dt; //Verlet Integration: moves particle based on velocity and acceleration
            p.old_pos = temp;

            // Reset acceleration for next frame
            p.acc = Vec2::ZERO;
        });
    }

    //ein "Simulationsschritt“ (forces → integration → collisions)
    pub fn step(&mut self, dt: f32) {
        self.apply_forces();
        self.apply_magnets();
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

    pub fn barnes_hut_nbody_step(&mut self, dt: f32) {
        self.apply_gravity_barnes_hut();
        self.update_positions(dt);
        //took out the collision detection, so that we can change it during runtime
    }

    //resets all particles
    pub fn clear(&mut self) {
        self.particles.clear();
        self.colliders.clear();
        self.rect_colliders.clear();
        self.gravity = DEFAULT_GRAVITY;
        self.particle_radius = DEFAULT_PARTICLE_RADIUS;
        self.restitution = DEFAULT_RESTITUTION;
        self.magnets.clear()
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

    pub fn add_galaxy(&mut self, n: usize, pos: Vec2) {
        let galaxy = galaxy(n, pos);

        for p in galaxy {
            self.add_particle(p);
        }
    }
}
