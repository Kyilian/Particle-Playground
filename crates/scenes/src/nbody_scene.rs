use super::Scene;
use glam::Vec2;
use pp_physics::{world, Particle, RectCollider, World};
use pp_render::RenderContext;
use rand::prelude::*;

pub struct NBodyScene {
    gravity: f32,
    mass: f32,
    particle_radius: f32,

    color: [f32; 4],
    ui_has_focus: bool,
    camera_zoom: f32,
    camera_offset: Vec2,
}

impl NBodyScene {
    pub fn new() -> Self {
        Self {
            gravity: 0.0,
            mass: 1.0,
            color: [1.0, 0.2, 0.2, 1.0],
            particle_radius: 2.0,
            ui_has_focus: false,
            camera_offset: Vec2::ZERO,
            camera_zoom: 1.0,
        }
    }
}

impl Default for NBodyScene {
    fn default() -> Self {
        NBodyScene::new()
    }
}

impl Scene for NBodyScene {
    fn update(&mut self, _world: &mut pp_physics::World, _dt: f32) {
        _world.gravity = Vec2::new(0.0, self.gravity);
        _world.nbody_step(_dt);
        _world.particle_radius = self.particle_radius;
    }

    fn on_click(
        &mut self,
        world: &mut World,
        mouse_pos: glam::Vec2,
        right_click: bool,
        left_click: bool,
        is_middle: bool,
    ) {
        if self.ui_has_focus {
            return;
        }

        let mut rng = rand::thread_rng();
        if left_click {
            let id = world.add_particle(Particle::new(mouse_pos, self.particle_radius));
            println!("Spawned particle #{id} at {:?}", mouse_pos);
        }

        if right_click {
            for _ in 0..100 {
                let offset_x = rng.gen_range(-150.0..150.0);
                let offset_y = rng.gen_range(-150.0..150.0);
                let spawn_pos = mouse_pos + Vec2::new(offset_x, offset_y);

                let mut p = Particle::new_with_mass(
                    spawn_pos,
                    rng.gen_range(2.0..10.0),
                    self.particle_radius,
                );

                let dist_vec = spawn_pos - mouse_pos;
                let dist = dist_vec.length();

                if dist > 1.0 {
                    let tangent = Vec2::new(-dist_vec.y, dist_vec.x).normalize();

                    let orbit_speed = 0.1;

                    let velocity = tangent * orbit_speed;

                    p.old_pos = p.pos - velocity;
                }

                world.add_particle(p);
            }
        }

        if is_middle {
            let mut p = Particle::new_with_mass(mouse_pos, 2000.0, self.particle_radius);

            p.old_pos = p.pos;

            world.add_particle(p);
        }
    }

    fn handle_scroll(&mut self, world: &mut World, mouse_pos: Vec2, scroll_y: f32) {}
    fn on_mouse_move(&mut self, _world: &mut World, _mouse_pos: Vec2) {}

    fn on_mouse_release(
        &mut self,
        _world: &mut World,
        _mouse_pos: Vec2,
        _right_click: bool,
        _left_click: bool,
        _is_middle: bool,
    ) {
    }

    fn render<'rpass>(
        &self,
        _world: &World,
        ctx: &RenderContext<'rpass>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        ctx.particle_renderer.update_render_settings(
            ctx.queue,
            self.color,
            self.camera_offset,
            self.camera_zoom,
        );

        ctx.particle_renderer.render(render_pass);
    }

    fn reset(&mut self, _world: &mut pp_physics::World) {
        _world.clear_particles();
    }

    fn ui(&mut self, _ctx: &egui::Context, _world: &mut pp_physics::World) {
        self.ui_has_focus = _ctx.wants_pointer_input() || _ctx.is_pointer_over_area();

        egui::Window::new("N-Body Simulation").show(_ctx, |ui| {
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

            ui.add(egui::Slider::new(&mut self.particle_radius, 1.0..=100.0).text("Particle Size"));
            ui.separator();

            ui.label(format!("Partikel: {}", _world.particles.len()));

            if ui.button("Alles zurücksetzen").clicked() {
                self.reset(_world);
            }
        });
    }
}
