use super::Scene;
use glam::Vec2;
use pp_physics::{Particle, World};
use pp_render::{ParticleRenderer, RenderContext};
use rand::prelude::*;

//Using constant placeholders for window size
//Need to get the User Window directly from Renderwindow or use a constant size for the Simulation for everyone

pub struct FallingParticles {
    gravity: f32,
    spawnrate: Option<i32>,
    particle_radius: f32,
    color: [f32; 4],
}

impl FallingParticles {
    pub fn new() -> Self {
        Self {
            gravity: 9.81, // Standardwert, vielleicht anpassen, bin mir über die Auswirkungen nicht ganz sicher
            spawnrate: None,
            color: [1.0, 0.2, 0.2, 1.0],
            particle_radius: 2.0,
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
    fn render<'rpass>(&self, world: &World, ctx: &RenderContext<'rpass>, render_pass: &mut wgpu::RenderPass<'rpass>) {  
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
            let id = world.add_particle(Particle::new(mouse_pos));
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

            let random_spawn_num: i8 = rng.gen();

            for _i in 0..random_spawn_num {
                let x = rng.gen_range(mouse_pos.x - 20.0..mouse_pos.x + 20.0);
                let y = rng.gen_range(mouse_pos.y - 20.0..mouse_pos.y + 20.0);
                let random_pos: Vec2 = Vec2::new(x, y);
                world.add_particle(Particle::new(random_pos));
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
        _world.clear_particles();       //zu clear_particles geändert damit collidor vorhanden bleibt
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
