use super::Scene;
use glam::Vec2;
use pp_physics::{CircleCollider, Particle, World};
use pp_render::RenderContext;

pub struct BenchmarkScene {
    gravity: f32,
    particle_radius: f32,
    color: [f32; 4],
    pub collider_radius: f32,
    collider_old: f32,

    bench_running: bool,
    particles_per_sec: usize,
    accum: f32,

    speed: f32,

    camera_zoom: f32,
    camera_offset: Vec2,
}

impl BenchmarkScene {
    pub fn new() -> Self {
        Self {
            gravity: 9.81,
            color: [1.0, 0.4, 0.4, 1.0],
            particle_radius: 2.0,
            collider_radius: 450.0,
            collider_old: 450.0,

            bench_running: false,
            particles_per_sec: 1000,
            accum: 0.0,
            speed: 800.0,

            camera_zoom: 1.0,
            camera_offset: Vec2::ZERO,
        }
    }

    pub fn init_world(world: &mut World) {
        world.add_circle_collider(CircleCollider {
            center: Vec2::new(0.0, 0.0),
            radius: 450.0,
        });
    }

    fn spawn_line(&mut self, world: &mut World, count: usize, dt: f32) {
        let y = 0.8 * self.collider_radius;
        let spacing = (2.2 * self.particle_radius).max(0.1);

        for i in 0..count {
            let x = (i as f32 - count as f32 * 0.5) * spacing;

            let mut p = Particle::new(Vec2::new(x, y), self.particle_radius);
            // shoot downards
            p.old_pos = p.pos - Vec2::new(0.0, self.speed * dt);

            world.add_particle(p);
        }
    }
}

impl Default for BenchmarkScene {
    fn default() -> Self {
        BenchmarkScene::new()
    }
}

impl Scene for BenchmarkScene {
    fn update(&mut self, world: &mut World, dt: f32) {
        world.gravity = Vec2::new(0.0, self.gravity);
        world.particle_radius = self.particle_radius;

        if self.collider_radius != self.collider_old {
            world.clear_collider();
            world.add_circle_collider(CircleCollider {
                center: Vec2::new(0.0, 0.0),
                radius: self.collider_radius,
            });
            self.collider_old = self.collider_radius;
        }

        // spawnrate per sec
        if self.bench_running {
            self.accum += dt;

            while self.accum >= 1.0 {
                self.accum -= 1.0;
                self.spawn_line(world, self.particles_per_sec, dt);
            }
        }

        world.step(dt);

        // stops at under 30 fps
        if self.bench_running && world.fps < 30.0 {
            self.bench_running = false;
            self.accum = 0.0;
        }
    }

    //call to the render function
    fn render<'rpass>(
        &self,
        _world: &World,
        ctx: &RenderContext<'rpass>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        //let width = ctx.renderer.config.width as f32;

        ctx.particle_renderer.update_render_settings(
            ctx.queue,
            self.color, // Rot
            self.camera_offset,
            self.camera_zoom,
        );

        //draw the particles
        ctx.particle_renderer.render(render_pass);
    }

    fn on_click(
        &mut self,
        _world: &mut World,
        _mouse_pos: Vec2,
        _right_click: bool,
        _left_click: bool,
        _is_middle: bool,
    ) {
    }

    fn handle_scroll(&mut self, _world: &mut World, _mouse_pos: Vec2, _scroll_y: f32) {}
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

    fn reset(&mut self, world: &mut World) {
        world.clear_particles();
        self.bench_running = false;
        self.accum = 0.0;
    }
    fn ui(&mut self, ctx: &egui::Context, world: &mut World) {
        egui::Window::new("Benchmark Scene").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("FPS:");
                let color = if world.fps < 30.0 {
                    egui::Color32::RED
                } else {
                    egui::Color32::GREEN
                };
                ui.colored_label(color, format!("{:.1}", world.fps));
            });

            ui.label(format!("Particles: {}", world.particles.len()));
            ui.separator();

            // StartStop per Button
            if ui
                .button(if self.bench_running { "Stop" } else { "Start" })
                .clicked()
            {
                self.bench_running = !self.bench_running;
                self.accum = 0.0;
            }

            // StartStop per Enter
            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.bench_running = !self.bench_running;
                self.accum = 0.0;
            }

            ui.separator();
            ui.add(
                egui::Slider::new(&mut self.particles_per_sec, 0..=20000).text("Particles / sec"),
            );
            ui.separator();
            ui.add(egui::Slider::new(&mut self.speed, 0.0..=3000.0).text("Start speed"));
            ui.separator();
            ui.add(egui::Slider::new(&mut self.gravity, 0.0..=2000.0).text("Gravity"));
            ui.separator();
            ui.add(
                egui::Slider::new(&mut self.collider_radius, 50.0..=1000.0).text("Collider radius"),
            );
            ui.separator();
            ui.add(
                egui::Slider::new(&mut self.particle_radius, 1.0..=20.0).text("Particle radius"),
            );

            ui.separator();
            if ui.button("Alles zurücksetzen").clicked() {
                self.reset(world);
            }
        });
    }
}
