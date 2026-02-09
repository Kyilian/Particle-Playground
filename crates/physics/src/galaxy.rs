use crate::Particle;
use glam::Vec2;
use rand::Rng;
//preset for an galaxy for the NBody Barnes Hut
pub fn galaxy(n: usize) -> Vec<Particle> {
    let inner_radius = 25.0;
    let outer_radius = (n as f32).sqrt() * 10.0;

    let mut rng = rand::thread_rng();
    let mut particles: Vec<Particle> = Vec::with_capacity(n);

    let m = 1e6;
    let center = Particle::new_with_mass(Vec2::ZERO, m, inner_radius);
    particles.push(center);

    while particles.len() < n {
        let angle = rng.gen::<f32>() * std::f32::consts::TAU;
        let (sin, cos) = angle.sin_cos();

        let t = inner_radius / outer_radius;
        let r = rng.gen::<f32>() * (1.0 - t * t) + t * t;

        let pos = Vec2::new(cos, sin) * outer_radius * r.sqrt();
        let mass = 1.0f32;
        let radius: f32 = mass.cbrt();

        particles.push(Particle::new_with_mass(pos, mass, radius));
    }

    particles.sort_by(|a, b| a.pos.length_squared().total_cmp(&b.pos.length_squared()));
    let mut current_mass_inside = 0.0;
    let g = 10.0;

    //calculating the inner mass
    for i in 0..n {
        let p = &mut particles[i];

        current_mass_inside += p.mass;

        if p.pos == Vec2::ZERO {
            continue;
        }
        //let dt = 1.0 / 60.0;

        //give the particles speed based on their mass
        //let dist = p.pos.length();
        //let orbit_speed = (g * current_mass_inside / dist).sqrt();

        //let tangent = Vec2::new(-p.pos.y, p.pos.x).normalize();

        //let velocity = tangent * orbit_speed;

        //p.old_pos = p.pos - (velocity * dt);

        p.acc = Vec2::ZERO;
    }
    particles
}
