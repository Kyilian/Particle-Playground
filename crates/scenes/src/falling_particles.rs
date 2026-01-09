use super::Scene;
use glam::Vec2;
use pp_physics::{Particle, World};
use pp_render::{ParticleRenderer, RenderContext};

//Using constant placeholders for window size
//Need to get the User Window directly from Renderwindow or use a constant size for the Simulation for everyone
const WIDTH: usize = 800;
const HEIGHT: usize = 600;

pub struct FallingParticles {
    gravity: f32,
    spawn_position: Vec2,
    spawnrate: Option<i32>,
}
impl FallingParticles {
    pub fn new() -> Self {
        Self {
            gravity: 9.81, // Standardwert, vielleicht anpassen, bin mir über die Auswirkungen nicht ganz sicher
            spawn_position: Vec2::new(100.0, 200.0),
            spawnrate: None,
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
    }

    //call to the render function
    fn render(&self, world: &World, ctx: &mut RenderContext) {
        //let width = ctx.renderer.config.width as f32;

        ctx.renderer.update_render_settings(
            ctx.queue,
            [1.0, 0.2, 0.2, 1.0], // Rot
            20.0,
        );

        ctx.renderer.update_particles(&world.particles, ctx.queue);

        //draw the particles
        ctx.renderer.render(ctx.pass);
    }

    // Spawn a particle with a click
    fn on_click(&mut self, _world: &mut World, _x: f32, _y: f32) {
        let p = Particle {
            pos: Vec2::new(_x, _y),
            old_pos: Vec2::new(_x, _y),
            acc: Vec2::ZERO,
        };
        _world.particles.push(p);
    }

    //Reset the Simulation to Default values
    fn reset(&mut self, _world: &mut World) {
        self.spawnrate = None;
        self.spawn_position = Vec2::new(100.0, 200.0);
        self.gravity = 9.81;
        _world.clear();
    }

    //Basic UI to test Sliders and Buttos
    fn ui(&mut self, _ctx: &egui::Context, _world: &mut World) {
        egui::Window::new("Falling Particle Simulation").show(_ctx, |ui| {
            ui.label("Test Parameter");

            ui.add(egui::Slider::new(&mut self.gravity, 0.0..=2000.0).text("Gravity"));
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

    #[test]
    fn on_click_spawns_particle_at_position() {
        let mut world = World::new();
        let mut scene = FallingParticles::new();

        scene.on_click(&mut world, 100.0, 200.0);

        assert_eq!(world.particles.len(), 1);
        let p = world.particles[0];
        assert!((p.pos - Vec2::new(100.0, 200.0)).length() < 0.001);
    }

    #[test]
    fn reset_to_default() {
        let mut world = World::new();
        let mut scene = FallingParticles::new();

        scene.on_click(&mut world, 100.0, 200.0);
        scene.on_click(&mut world, 2.0, 200.0);
        scene.on_click(&mut world, 332.0, 200.0);

        scene.gravity = 100.0;
        scene.spawn_position = Vec2::new(1.0, 1.0);
        scene.reset(&mut world);

        assert_eq!(world.particles.len(), 0);
        assert_eq!(scene.gravity, 9.81);
    }
}
