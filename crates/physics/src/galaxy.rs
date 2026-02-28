use crate::Particle;
use glam::Vec2;
use rand::Rng;

//AI bugfix with Claude
//preset for an galaxy for the NBody Barnes Hut
pub fn galaxy(n: usize, pos: Vec2) -> Vec<Particle> {
    let mut rng = rand::thread_rng();
    let mut particles: Vec<Particle> = Vec::with_capacity(n);

    //black hole at the center
    let central_mass = 2000.0;
    let central_radius = 5.0;
    let center = Particle::new_with_mass(pos, central_mass, central_radius);
    particles.push(center);

    let inner_radius = 20.0;
    let outer_radius = (n as f32).sqrt() * 10.0;

    let e_sq: f32 = 10000.0;

    let dt: f32 = 1.0 / 120.0;

    while particles.len() < n {
        // Uniform area distribution in disc
        let angle = rng.gen::<f32>() * std::f32::consts::TAU;
        let (sin, cos) = angle.sin_cos();

        let t = inner_radius / outer_radius;
        let u: f32 = rng.gen::<f32>() * (1.0 - t * t) + t * t;
        let r = outer_radius * u.sqrt();

        let local_pos = Vec2::new(cos, sin) * r;
        let particle_pos = local_pos + pos;
        let mass = 1.0f32;
        let radius = mass.cbrt().max(1.5);

        let mut p = Particle::new_with_mass(particle_pos, mass, radius);

        // --- Orbital velocity ---
        // Quadtree acc gives: |a| = M / (r² + e²)
        // Circular orbit: v²/r = |a|  →  v = sqrt(r * M / (r² + e²))
        let dist_sq = r * r;
        let orbit_speed = (r * central_mass / (dist_sq + e_sq)).sqrt();

        // Tangent direction (perpendicular to radial, prograde)
        let tangent = Vec2::new(-local_pos.y, local_pos.x).normalize();
        let velocity = tangent * orbit_speed;

        // Small jitter for natural look
        let jitter = 1.0 + rng.gen_range(-0.03..0.03);

        //start speed for verlet integration
        p.old_pos = p.pos - velocity * jitter * dt;

        particles.push(p);
    }

    particles
}
