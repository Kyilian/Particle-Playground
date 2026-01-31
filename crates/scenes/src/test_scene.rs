use super::Scene;
use glam::Vec2;
use pp_physics::{CircleCollider, Particle, RectCollider, World};
use pp_render::RenderContext;
use rand::prelude::*;

//Using constant placeholders for window size
//Need to get the User Window directly from Renderwindow or use a constant size for the Simulation for everyone

pub struct TestScene {
    gravity: f32,
    spawnrate: Option<i32>,
    particle_radius: f32,
    color: [f32; 4],
    pub collider_radius: f32,
    collider_old: f32,

    fps: f32,

    //to add a rect_collider
    rect_collider_active: bool,
    rect_size: Vec2,
    rect_size_old: Vec2,
}

impl TestScene {
    pub fn new() -> Self {
        Self {
            gravity: 9.81, // Standardwert, vielleicht anpassen, bin mir über die Auswirkungen nicht ganz sicher
            spawnrate: None,
            color: [1.0, 0.2, 0.2, 1.0],
            particle_radius: 2.0,
            collider_radius: 250.0,
            collider_old: 250.0,
            fps: 60.0,

            rect_collider_active: false,
            rect_size: Vec2::new(400.0, 400.0),
            rect_size_old: Vec2::new(400.0, 400.0),
        }
    }

    pub fn init_world(world: &mut World) {
        world.add_circle_collider(CircleCollider {
            center: Vec2::new(0.0, 0.0),
            radius: 250.0,
        });
    }
}

impl Default for TestScene {
    fn default() -> Self {
        TestScene::new()
    }
}

impl Scene for TestScene {
    //update particle position in world
    fn update(&mut self, _world: &mut World, _dt: f32) {
        _world.gravity = Vec2::new(0.0, self.gravity);
        _world.step(_dt);

        if self.collider_radius != self.collider_old && !self.rect_collider_active {
            _world.clear_collider();
            _world.clear_rect_collider();
            _world.add_circle_collider(CircleCollider {
                center: Vec2::new(0.0, 0.0), // (0,0) is now the center
                radius: self.collider_radius,
            });

            self.collider_old = self.collider_radius;
        } else if self.rect_size != self.rect_size_old && self.rect_collider_active {
            _world.clear_collider();
            _world.clear_rect_collider();

            _world.add_rect_collider(RectCollider {
                center: Vec2::new(0.0, 0.0),
                width: self.rect_size.y,
                height: self.rect_size.x,
            });
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
            let id = world.add_particle(Particle::new(mouse_pos, world.particle_radius));
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

    fn handle_scroll(&mut self, world: &mut World, mouse_pos: Vec2, scroll_y: f32) {}

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

            if ui.button("Switch Colliders").clicked() {
                self.rect_collider_active = !self.rect_collider_active;
            }
            ui.add(
                egui::Slider::new(&mut self.collider_radius, 50.0..=1000.0).text("Circle Collider"),
            );
            ui.separator();

            ui.add(
                egui::Slider::new(&mut self.rect_size.x, 50.0..=1000.0)
                    .text("Rect Collider Height"),
            );
            ui.add(
                egui::Slider::new(&mut self.rect_size.y, 50.0..=1000.0).text("Rect Collider Width"),
            );
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
