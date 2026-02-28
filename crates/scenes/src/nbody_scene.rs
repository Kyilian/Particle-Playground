use super::Scene;
use glam::Vec2;
use pp_physics::{Particle, World};
use pp_render::RenderContext;
use rand::prelude::*;

//A simple n-body simulation
//This scene uses the O(n^2) gravity calculation to see the difference between the gravity calculation between every particles and only the quadrats
//Controlls:
//Left Click: Spawn a single particle at the mouse position
//Right Click: Spawn a small galaxy of particles around the mouse position with a random velocity to simulate a small galaxy
//Middle Click: Spawn a heavy particle

pub struct NBodyScene {
    gravity: f32,
    mass: f32,
    particle_radius: f32,

    ui_has_focus: bool,
    camera_zoom: f32,
    camera_offset: Vec2,

    is_dragging: bool,
    last_mouse_pos: Vec2,
}

impl NBodyScene {
    pub fn new() -> Self {
        Self {
            gravity: 0.0,
            mass: 1.0,
            particle_radius: 2.0,
            ui_has_focus: false,
            camera_offset: Vec2::ZERO,
            camera_zoom: 1.0,

            is_dragging: false,
            last_mouse_pos: Vec2::ZERO,
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
        let world_mouse_pos = (mouse_pos - self.camera_offset) / self.camera_zoom;

        let mut rng = rand::thread_rng();
        if left_click {
            let p = Particle::new_with_mass(world_mouse_pos, self.mass, self.particle_radius);

            world.add_particle(p);
        }

        //spawn 100 particles in a radius of 150 around the mouse position
        if right_click {
            for _ in 0..100 {
                let offset_x = rng.gen_range(-150.0..150.0);
                let offset_y = rng.gen_range(-150.0..150.0);
                let spawn_pos = world_mouse_pos + Vec2::new(offset_x, offset_y);

                let mut p = Particle::new_with_mass(spawn_pos, self.mass, self.particle_radius);

                let dist_vec = spawn_pos - world_mouse_pos;
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

        //spawn a heavy particle that attracts other particles with a strong force
        //Work in progress: the heavy particle shouldnt move as much as a light particle
        if is_middle {
            self.is_dragging = true;
            self.last_mouse_pos = mouse_pos;
        }
    }

    //Zoom out with the mouse scroll
    fn handle_scroll(&mut self, _world: &mut World, mouse_pos: Vec2, scroll_y: f32) {
        let old_zoom = self.camera_zoom;

        self.camera_zoom += scroll_y * 0.1;
        self.camera_zoom = self.camera_zoom.clamp(0.1, 100.0);

        if self.camera_zoom == old_zoom {
            return;
        }

        let world_mouse_pos = (mouse_pos - self.camera_offset) / old_zoom;
        self.camera_offset = mouse_pos - (world_mouse_pos * self.camera_zoom);
    }

    // calculate the new camera offset when dragging the mouse
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

        egui::Window::new("N-Body Simulation O(n^2)").show(_ctx, |ui| {
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
            ui.add(egui::Slider::new(&mut self.mass, 0.0..=10000.0).text("Particle Mass"));

            ui.separator();
            ui.add(
                egui::Slider::new(&mut self.particle_radius, 0.0..=10000.0).text("Particle Radius"),
            );

            ui.label(format!("Particles: {}", _world.particles.len()));

            if ui.button("Reset Simulation").clicked() {
                self.reset(_world);
            }
        });
    }
}
