#[cfg(test)]
mod test { // collider tests generated with Claude

    use crate::{CircleCollider, Particle, World};
    use glam::Vec2;

    // helper: creates a world with a circle collider at the origin
    fn setup_world_with_circle(radius: f32) -> World {
        let mut world = World::new();
        world.add_circle_collider(CircleCollider {
            center: Vec2::ZERO,
            radius,
        });
        world
    }

    // --- Tests from the TODO list ---

    #[test]
    fn particle_outside_circle_is_clamped_to_surface() {
        let mut world = setup_world_with_circle(100.0);

        // place particle clearly outside the boundary
        let mut p = Particle::new(Vec2::new(200.0, 0.0), 6.0);
        p.old_pos = p.pos; // no velocity
        world.add_particle(p);

        world.solve_collisions();

        let p_after = &world.particles[0];
        let dist = p_after.pos.length();

        assert!(
            dist <= 100.0 + 1e-4,
            "Particle should be clamped to circle surface, but dist = {}",
            dist
        );
    }

    #[test]
    fn particle_inside_circle_stays_inside() {
        let mut world = setup_world_with_circle(100.0);

        // place particle well inside the boundary
        let original_pos = Vec2::new(30.0, 20.0);
        let mut p = Particle::new(original_pos, 6.0);
        p.old_pos = p.pos; // no velocity
        world.add_particle(p);

        world.solve_collisions();

        let p_after = &world.particles[0];

        assert!(
            (p_after.pos - original_pos).length() < 1e-6,
            "Particle inside the circle should not move, but moved to {:?}",
            p_after.pos
        );
    }

    #[test]
    fn verlet_with_circle_collision() {
        let mut world = setup_world_with_circle(100.0);

        // particle starts inside but is moving outward fast
        let mut p = Particle::new(Vec2::new(95.0, 0.0), 6.0);
        p.old_pos = Vec2::new(80.0, 0.0); // velocity = (15, 0), heading outward

        world.add_particle(p);

        // run a full simulation step
        world.step(1.0 / 60.0);

        let p_after = &world.particles[0];
        let dist = p_after.pos.length();

        assert!(
            dist <= 100.0 + 1e-3,
            "Particle should stay inside circle after verlet step, but dist = {}",
            dist
        );
    }
}