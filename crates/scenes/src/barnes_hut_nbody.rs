use super::Scene;
use glam::Vec2;
use pp_physics::{world, Particle, RectCollider, World};
use pp_render::RenderContext;
use rand::prelude::*;

pub struct BarnesHutNbody {
    gravity: f32,
    mass: f32,
    particle_radius: f32,
    camera_zoom: f32,
    camera_offset: Vec2,
    color: [f32; 4],

    is_dragging: bool,
    last_mouse_pos: Vec2,
}

impl BarnesHutNbody {
    pub fn new() -> Self {
        Self {
            gravity: 10.0,
            mass: 1.0,
            color: [1.0, 0.2, 0.2, 1.0],
            particle_radius: 2.0,
            camera_zoom: 1.0,
            camera_offset: Vec2::ZERO,
            is_dragging: false,
            last_mouse_pos: Vec2::ZERO,
        }
    }
}

impl Default for BarnesHutNbody {
    fn default() -> Self {
        BarnesHutNbody::new()
    }
}

impl Scene for BarnesHutNbody {
    fn update(&mut self, _world: &mut pp_physics::World, _dt: f32) {
        _world.gravity = Vec2::ZERO;
        _world.barnes_hut_nbody_step(_dt);
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
        let mut rng = rand::thread_rng();

        let world_mouse_pos = (mouse_pos - self.camera_offset) / self.camera_zoom;

        if left_click {
            let id = world.add_particle(Particle::new(world_mouse_pos, self.particle_radius));
            println!("Spawned particle #{id} at {:?}", world_mouse_pos);
        }

        if right_click {
            for _ in 0..100 {
                let offset_x = rng.gen_range(-150.0..150.0);
                let offset_y = rng.gen_range(-150.0..150.0);
                let spawn_pos = world_mouse_pos + Vec2::new(offset_x, offset_y);

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
            self.is_dragging = true;
            self.last_mouse_pos = mouse_pos;
            let r = self.particle_radius * 3.0;
            let mut p = Particle::new_with_mass(world_mouse_pos, 20000000000000000000.0, r);

            p.old_pos = p.pos;

            world.add_particle(p);
        }
    }

    fn handle_scroll(&mut self, _world: &mut World, mouse_pos: Vec2, scroll_y: f32) {
        let old_zoom = self.camera_zoom;

        self.camera_zoom += scroll_y * 0.1;
        self.camera_zoom = self.camera_zoom.clamp(0.1, 100.0);

        if self.camera_zoom == old_zoom {
            return;
        }

        let world_mouse_pos = (mouse_pos - self.camera_offset) / old_zoom;
        self.camera_offset = mouse_pos - (world_mouse_pos * self.camera_zoom);

        println!("Zoom zur Maus! Neuer Zoom: {:.2}", self.camera_zoom);
    }
    fn on_mouse_move(&mut self, _world: &mut World, _mouse_pos: Vec2) {
        if self.is_dragging {
            let delta = _mouse_pos - self.last_mouse_pos;
            self.camera_offset += delta;
            self.last_mouse_pos = _mouse_pos;
        }
    }
    fn on_mouse_release(
        &mut self,
        _world: &mut World,
        _mouse_pos: Vec2,
        _right_click: bool,
        _left_click: bool,
        _is_middle: bool,
    ) {
        self.is_dragging = false;
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

            ui.add(egui::Slider::new(&mut self.particle_radius, 1.0..=100.0).text("Particle Size"));
            ui.separator();

            ui.add(egui::Slider::new(&mut self.gravity, 0.0..=2000.0).text("Gravity"));
            ui.separator();

            ui.label(format!("Partikel: {}", _world.particles.len()));

            if ui.button("Alles zurücksetzen").clicked() {
                self.reset(_world);
            }
            ui.separator();

            if ui.button("Spawn Galaxy").clicked() {
                _world.add_galaxy(5000);
            }
        });
    }
}
