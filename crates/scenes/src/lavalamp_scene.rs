use super::Scene;
use glam::Vec2;
use pp_physics::{world::Magnet, Particle, RectCollider, World};
use pp_render::RenderContext;
use rand::prelude::*;
pub struct LavaLampScene {
    time: f32,
    particle_radius: f32,
    magnet_strength: f32,
    ui_has_focus: bool,
    rect_width: f32,
    rect_height: f32,
    particles: i16,
    change_sized: f32,
}

impl LavaLampScene {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            particle_radius: 5.0,
            magnet_strength: 300.0,
            ui_has_focus: false,
            rect_width: 300.0,
            rect_height: 500.0,
            particles: 400,
            change_sized: 1.0,
        }
    }
}

impl Default for LavaLampScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for LavaLampScene {
    fn update(&mut self, world: &mut World, dt: f32) {
        self.time += dt;

        world.magnets.clear();

        let y_pos_1 = (self.time * 1.5).sin() * 150.0 * self.change_sized;
        world.magnets.push(Magnet {
            pos: Vec2::new(0.0, y_pos_1),
            strength: self.magnet_strength,
            radius: 160.0 * self.change_sized,
        });

        let y_pos_2 = (self.time * 0.8 + 2.0).sin() * 180.0 * self.change_sized;
        world.magnets.push(Magnet {
            pos: Vec2::new(-40.0, y_pos_2),
            strength: self.magnet_strength * 0.8,
            radius: 130.0 * self.change_sized,
        });

        world.magnets.push(Magnet {
            pos: Vec2::new(0.0, 240.0 * self.change_sized),
            strength: -self.magnet_strength * 2.0,
            radius: 190.0 * self.change_sized,
        });

        world.magnets.push(Magnet {
            pos: Vec2::new(0.0, -240.0 * self.change_sized),
            strength: -self.magnet_strength,
            radius: 160.0 * self.change_sized,
        });

        world.particle_radius = self.particle_radius;
        world.step(dt);

        for p in &mut world.particles {
            if p.is_magnet {
                p.old_pos = p.pos;
            }
        }
    }

    fn render<'rpass>(
        &self,
        _world: &World,
        ctx: &RenderContext<'rpass>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        ctx.particle_renderer
            .update_render_settings(ctx.queue, [0.0; 4], self.particle_radius);
        ctx.particle_renderer.render(render_pass);
    }

    fn on_click(
        &mut self,
        world: &mut World,
        mouse_pos: Vec2,
        _r: bool,
        left_click: bool,
        _m: bool,
    ) {
        if self.ui_has_focus {
            return;
        }

        if left_click {
            let blast_radius = 150.0;
            let blast_force = 500.0;

            for p in &mut world.particles {
                let dir = p.pos - mouse_pos;
                let dist_sq = dir.length_squared();
                if dist_sq < blast_radius * blast_radius && dist_sq > 1.0 {
                    let dist = dist_sq.sqrt();
                    let force = (1.0 - (dist / blast_radius)) * blast_force;
                    let push_vec = (dir / dist) * force;
                    p.old_pos -= push_vec * (1.0 / 60.0);
                }
            }
        }
    }

    fn handle_scroll(&mut self, _world: &mut World, _mouse_pos: Vec2, _scroll_y: f32) {}

    fn reset(&mut self, world: &mut World) {
        world.clear_particles();
        world.clear_rect_collider();
        world.clear_collider();
        world.magnets.clear();

        world.gravity = Vec2::ZERO;

        world.add_rect_collider(RectCollider {
            center: Vec2::ZERO,
            width: self.rect_width * self.change_sized,
            height: self.rect_height * self.change_sized,
        });

        let mut rng = rand::thread_rng();

        for _ in 0..self.particles {
            let x = rng.gen_range(-self.rect_width / 2.0 + 15.0..self.rect_width / 2.0 - 15.0);
            let y = rng.gen_range(-self.rect_height / 2.0 + 15.0..self.rect_height / 2.0 - 15.0);

            world.add_particle(Particle::new(Vec2::new(x, y), self.particle_radius));
        }

        self.time = 0.0;
    }

    fn ui(&mut self, ctx: &egui::Context, world: &mut World) {
        self.ui_has_focus = ctx.wants_pointer_input() || ctx.is_pointer_over_area();

        egui::Window::new("Simple Lava Lampe").show(ctx, |ui| {
            ui.label(format!("FPS: {:.1}", world.fps));
            ui.label(format!("Partikel: {}", world.particles.len()));
            ui.separator();
            ui.add(
                egui::Slider::new(&mut self.particle_radius, 2.0..=10.0)
                    .text("Particle Size change with new start"),
            );
            ui.add(egui::Slider::new(&mut self.magnet_strength, 0.0..=1000.0).text("Apply Force"));
            ui.separator();
            ui.add(
                egui::Slider::new(&mut self.particles, 100..=2000)
                    .text("Particle Count change with new start"),
            );
            ui.separator();
            ui.add(
                egui::Slider::new(&mut self.change_sized, 0.5..=2.0)
                    .text("Change size of the lava lamp"),
            );
            ui.separator();
            if ui.button("Reset Lava Lamp").clicked() {
                self.reset(world);
            }

            if ui.button("Lava starten").clicked() {
                self.reset(world);
            }
        });
    }
}
