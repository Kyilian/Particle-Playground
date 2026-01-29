use super::Scene;
use glam::Vec2;
use pp_physics::{CircleCollider, Particle, World};
use pp_render::{ParticleRenderer, RenderContext};
use rand::prelude::*;

//Using constant placeholders for window size
//Need to get the User Window directly from Renderwindow or use a constant size for the Simulation for everyone

pub struct FallingParticles {
    gravity: f32,
    spawnrate: Option<i32>,
    particle_radius: f32,
    color: [f32; 4],
    pub collider_radius: f32,
    collider_old: f32,
}

impl FallingParticles {
    pub fn new() -> Self {
        Self {
            gravity: 9.81, // Standardwert, vielleicht anpassen, bin mir über die Auswirkungen nicht ganz sicher
            spawnrate: None,
            color: [1.0, 0.2, 0.2, 1.0],
            particle_radius: 2.0,
            collider_radius: 250.0,
            collider_old: 250.0,
        }
    }
}

impl Default for FallingParticles {
    fn default() -> Self {
        FallingParticles::new()
    }
}

impl Scene for FallingParticles {
    //update particle position in world
    fn update(&mut self, _world: &mut World, _dt: f32) {
        _world.gravity = Vec2::new(0.0, self.gravity);
        _world.step(_dt);

        if self.collider_radius != self.collider_old {
            _world.clear_collider();
            _world.add_circle_collider(CircleCollider {
                center: Vec2::new(0.0, 0.0), // (0,0) is now the center
                radius: self.collider_radius,
            });
            self.collider_old = self.collider_radius;
        }

        _world.particle_radius = self.particle_radius;
    }

    //call to the render function
    fn render<'rpass>(
        &self,
        world: &World,
        ctx: &RenderContext<'rpass>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        //let width = ctx.renderer.config.width as f32;

        ctx.particle_renderer.update_render_settings(
            ctx.queue,
            self.color, // Rot
            self.particle_radius,
        );

        //draw the particles
        ctx.particle_renderer.render(render_pass);
    }

    // Spawn a particle with a click
    fn on_click(
        &mut self,
        world: &mut World,
        mouse_pos: Vec2,
        right_click: bool,
        left_click: bool,
        is_middle: bool,
    ) {
        let mut rng = rand::thread_rng();

        if left_click {
            let id = world.add_particle(Particle::new(mouse_pos, self.particle_radius));
            println!("Spawned particle #{id} at {:?}", mouse_pos);
        }

        if right_click {
            if let Some(nearest) = world.find_nearest_particle(mouse_pos) {
                println!(
                    "Nearest particle is #{nearest} at pos {:?} to mouse {:?}",
                    world.particles[nearest].pos, mouse_pos
                );
            } else {
                println!("No particle close to {:?}", mouse_pos);
            }

            let random_spawn_num: u8 = rng.gen();

            for _i in 0..random_spawn_num {
                let x = rng.gen_range(mouse_pos.x - 20.0..mouse_pos.x + 20.0);
                let y = rng.gen_range(mouse_pos.y - 20.0..mouse_pos.y + 20.0);
                let random_pos: Vec2 = Vec2::new(x, y);
                world.add_particle(Particle::new(random_pos, world.particle_radius));
            }
        }

        if is_middle {
            let random_color: [f32; 4] = rng.gen();

            self.color = random_color;
        }
    }

    //Reset the Simulation to Default values
    fn reset(&mut self, _world: &mut World) {
        self.spawnrate = None;
        self.gravity = 9.81;
        _world.clear_particles(); //zu clear_particles geändert damit collidor vorhanden bleibt
    }

    //Basic UI to test Sliders and Buttos
    fn ui(&mut self, _ctx: &egui::Context, _world: &mut World) {
        egui::Window::new("Falling Particle Simulation").show(_ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("FPS:");

                let color = if _world.fps < 30.0 {
                    egui::Color32::RED
                } else {
                    egui::Color32::GREEN
                };

                ui.colored_label(color, format!("{:.1}", _world.fps));
            });

            ui.separator();
            ui.label("Test Parameter");

            ui.add(egui::Slider::new(&mut self.gravity, 0.0..=2000.0).text("Gravity"));
            ui.separator();

            ui.add(egui::Slider::new(&mut self.collider_radius, 50.0..=1000.0).text("Collider"));
            ui.separator();

            ui.add(egui::Slider::new(&mut self.particle_radius, 1.0..=100.0).text("Particle Size"));
            ui.separator();

            ui.label(format!("Partikel: {}", _world.particles.len()));

            if ui.button("Alles zurücksetzen").clicked() {
                self.reset(_world);
            }
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    use pp_physics::World;

    #[test]
    fn test_initial_state() {
        let scene = FallingParticles::new();
        // Standardwerte prüfen
        assert_eq!(scene.gravity, 9.81);
        assert_eq!(scene.particle_radius, 2.0);
        assert_eq!(scene.color, [1.0, 0.2, 0.2, 1.0]);
    }

    #[test]
    fn test_update_propagates_gravity_to_world() {
        let mut scene = FallingParticles::new();
        let mut world = World::new();
        scene.gravity = 20.0;

        scene.update(&mut world, 0.016);

        assert_eq!(world.gravity.y, 20.0);
        assert_eq!(world.gravity.x, 0.0);
    }

    #[test]
    fn test_left_click_spawns_single_particle() {
        let mut scene = FallingParticles::new();
        let mut world = World::new();
        let click_pos = Vec2::new(100.0, 100.0);

        scene.on_click(&mut world, click_pos, false, true, false);

        assert_eq!(
            world.particles.len(),
            1,
            "Es sollte genau 1 Partikel gespawnt sein"
        );

        let p = &world.particles[0];

        let diff = p.pos - click_pos;
        assert!(diff.length() < 0.001);
    }

    #[test]
    fn test_middle_click_changes_color() {
        let mut scene = FallingParticles::new();
        let mut world = World::new();
        let old_color = scene.color;

        let mut color_changed = false;

        for _ in 0..5 {
            scene.on_click(&mut world, Vec2::ZERO, false, false, true);

            if scene.color != old_color {
                color_changed = true;
                break;
            }
        }

        assert!(
            color_changed,
            "Middle click should change the particle color"
        );
    }

    #[test]
    fn test_reset_clears_particles_and_resets_gravity() {
        let mut scene = FallingParticles::new();
        let mut world = World::new();

        scene.gravity = 500.0;

        scene.on_click(&mut world, Vec2::ZERO, false, true, false);
        scene.on_click(&mut world, Vec2::ZERO, false, true, false);
        assert_eq!(world.particles.len(), 2);

        scene.reset(&mut world);

        assert_eq!(
            scene.gravity, 9.81,
            "Gravity sollte auf Default zurückgesetzt sein"
        );
        assert_eq!(world.particles.len(), 0, "Partikel sollten gelöscht sein");
    }
}
