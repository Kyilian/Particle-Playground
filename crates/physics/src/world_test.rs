#[cfg(test)]
mod test {

    use super::*;
    use crate::{CircleCollider, Particle, RectCollider, World};
    use glam::Vec2;

    #[test]
    fn verlet_moves_particle_with_gravity() {
        let mut world = World::new();
        let id = world.add_particle(Particle::new(Vec2::ZERO, 6.0));

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
            radius: 6.0,
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

        let id = world.add_particle(Particle::new(Vec2::new(20.0, 0.0), 6.0));

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
        let mut p = Particle::new(Vec2::new(12.0, 0.0), 6.0);
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

        let mut a = Particle::new(Vec2::new(0.0, 0.0), 6.0);
        let mut b = Particle::new(Vec2::new(5.0, 0.0), 6.0); // overlap (min_dist = 12)

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

        let mut a = Particle::new(Vec2::new(0.0, 0.0), 6.0);
        let mut b = Particle::new(Vec2::new(20.0, 0.0), 6.0); // no overlap

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

        let mut a = Particle::new(Vec2::new(0.0, 0.0), 6.0);
        a.old_pos = Vec2::new(-1.0, 0.0);

        let mut b = Particle::new(Vec2::new(11.0, 0.0), 6.0); // overlap
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

        let mut a = Particle::new(Vec2::new(0.0, 0.0), 6.0);
        let mut b = Particle::new(Vec2::new(0.0, 0.0), 6.0); // dist == 0

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

        let mut a = Particle::new(Vec2::new(0.0, 0.0), 6.0);
        let mut b = Particle::new(Vec2::new(5.0, 0.0), 6.0); // overlap (min_dist = 12)

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

        let mut a = Particle::new(Vec2::new(0.0, 0.0), 6.0);
        let mut b = Particle::new(Vec2::new(20.0, 0.0), 6.0); // no overlap

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

        let mut a = Particle::new(Vec2::new(0.0, 0.0), 6.0);
        let mut b = Particle::new(Vec2::new(0.0, 0.0), 6.0); // dist == 0

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
        let mut p0 = Particle::new(Vec2::new(0.0, 0.0), 6.0);
        let mut p1 = Particle::new(Vec2::new(11.5, 0.0), 6.0); // overlap (min_dist=12)
        let mut p2 = Particle::new(Vec2::new(30.0, 0.0), 6.0); // no overlap
        let mut p3 = Particle::new(Vec2::new(11.5, 11.5), 6.0); // diagonal neighbor

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

    //Gemini Unit Tests

    #[cfg(test)]
    // Helper to create a standard test world with a 100x100 box
    fn setup_world_with_rect() -> World {
        let mut world = World::new();
        // Box from -50 to +50 on X and Y
        world.add_rect_collider(RectCollider {
            center: Vec2::ZERO,
            width: 100.0,
            height: 100.0,
        });
        // Set standard radius for predictable math (e.g., 5.0)
        world.particle_radius = 5.0;
        world.restitution = 1.0; // Perfect elasticity for easier math checks
        world
    }

    #[test]
    fn test_rect_containment() {
        let mut world = setup_world_with_rect();

        // Spawn particle WAY outside (X=100, Y=100)
        // Max bounds are 50.0. Minus radius 5.0 = 45.0 is the limit.
        let mut p = Particle::new(Vec2::new(100.0, 100.0), 6.0);
        p.old_pos = Vec2::new(100.0, 100.0); // No velocity

        world.add_particle(p);
        world.solve_rect_collisions();

        let p_after = world.particles[0];

        // Should be clamped exactly to max_x - radius
        assert_eq!(
            p_after.pos.x, 45.0,
            "Particle X was not clamped to boundary"
        );
        assert_eq!(
            p_after.pos.y, 45.0,
            "Particle Y was not clamped to boundary"
        );
    }

    #[test]
    fn test_rect_wall_bounce_x() {
        let mut world = setup_world_with_rect();

        // Max X bound is 50.0. Limit is 45.0.
        // Place particle at 46.0 (1.0 pixel inside the wall)
        // Moving Right: old_pos at 40.0 (Velocity +6)
        let mut p = Particle::new(Vec2::new(46.0, 0.0), 6.0);
        p.old_pos = Vec2::new(40.0, 0.0);

        world.add_particle(p);
        world.solve_rect_collisions();

        let p_after = world.particles[0];

        // 1. Position should be clamped
        assert_eq!(p_after.pos.x, 45.0);

        // 2. Velocity should be inverted (pointing Left)
        // Current Pos (45) < Old Pos implies velocity is negative (moving left)
        assert!(p_after.pos.x < p_after.old_pos.x, "Velocity X did not flip");

        // 3. Y velocity should remain roughly unchanged (0.0)
        let vel_y = p_after.pos.y - p_after.old_pos.y;
        assert!(
            vel_y.abs() < 0.001,
            "Velocity Y should not change on vertical wall hit"
        );
    }

    #[test]
    fn test_rect_wall_bounce_y() {
        let mut world = setup_world_with_rect();

        // Hitting Top/Bottom wall (Y axis)
        // Place at Y = 46.0
        let mut p = Particle::new(Vec2::new(0.0, 46.0), 6.0);
        p.old_pos = Vec2::new(0.0, 40.0); // Moving Up

        world.add_particle(p);
        world.solve_rect_collisions();

        let p_after = world.particles[0];

        assert_eq!(p_after.pos.y, 45.0); // Clamped
        assert!(p_after.pos.y < p_after.old_pos.y, "Velocity Y did not flip");
    }
}
