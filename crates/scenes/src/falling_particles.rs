use super::Scene;
use glam::Vec2;
use pp_physics::{world::Magnet, CircleCollider, Particle, World};
use pp_render::RenderContext;
use rand::prelude::*;

//The main Pacticle playground scene, to play around with the Particles

#[derive(PartialEq)]
enum MouseClickMode {
    SpawnSingle,
    SpawnCluster,
    PlaceMagnet,
}
#[derive(PartialEq)]
enum MouseScrollMode {
    ResizeMagnet,
    AdjustMagnetStrength,
    // for implementation of other scroll modes
}

pub struct FallingParticles {
    gravity: f32,
    spawnrate: Option<i32>,
    particle_radius: f32,
    magnet_radius: f32,
    magnet_strength: f32,
    color: [f32; 4],
    pub collider_radius: f32,
    collider_old: f32,
    current_mode: MouseClickMode,
    current_scroll_mode: MouseScrollMode,
    ui_has_focus: bool,

    camera_offset: Vec2,
    camera_zoom: f32,
}

impl FallingParticles {
    pub fn new() -> Self {
        Self {
            gravity: 9.81, // standardvalue
            spawnrate: None,
            color: [1.0, 0.2, 0.2, 1.0],
            particle_radius: 2.0,
            magnet_radius: 200.0,
            magnet_strength: 200.0,
            collider_radius: 250.0,
            collider_old: 250.0,
            current_mode: MouseClickMode::SpawnSingle,
            current_scroll_mode: MouseScrollMode::ResizeMagnet,
            ui_has_focus: false,

            camera_offset: Vec2::ZERO,
            camera_zoom: 1.0,
        }
    }

    pub fn init_world(world: &mut World) {
        world.add_circle_collider(CircleCollider {
            center: Vec2::new(0.0, 0.0),
            radius: 250.0,
        });
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

        for p in &mut _world.particles {
            if p.is_magnet {
                p.old_pos = p.pos;
            }
        }

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
        world: &mut World,
        mouse_pos: Vec2,
        right_click: bool,
        left_click: bool,
        is_middle: bool,
    ) {
        if self.ui_has_focus {
            return;
        }

        let mut rng = rand::thread_rng();

        if left_click {
            match self.current_mode {
                MouseClickMode::SpawnSingle => {
                    world.add_particle(Particle::new(mouse_pos, self.particle_radius));
                }
                MouseClickMode::SpawnCluster => {
                    let random_spawn_num: u8 = rng.gen();
                    for _i in 0..random_spawn_num {
                        let x = rng.gen_range(mouse_pos.x - 20.0..mouse_pos.x + 20.0);
                        let y = rng.gen_range(mouse_pos.y - 20.0..mouse_pos.y + 20.0);
                        let random_pos: Vec2 = Vec2::new(x, y);
                        world.add_particle(Particle::new(random_pos, world.particle_radius));
                    }
                }
                MouseClickMode::PlaceMagnet => {
                    world.magnets.push(Magnet {
                        pos: mouse_pos,
                        strength: self.magnet_strength,
                        radius: self.magnet_radius,
                    });
                    let mut magnet_p = Particle::new(mouse_pos, self.magnet_radius);
                    let density = self.magnet_strength.abs() / 10000.0;
                    // red for pull, blue for push
                    let new_color = if self.magnet_strength >= 0.0 {
                        [1.0, 0.0, 0.0, density]
                    } else {
                        [0.0, 0.0, 1.0, density]
                    };
                    magnet_p.color = new_color;
                    magnet_p.is_magnet = true; // particle marked as magnet
                    world.add_particle(magnet_p);
                }
            }
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
        }

        if is_middle {
            let random_color: [f32; 4] = rng.gen();

            self.color = random_color;
        }
    }

    fn handle_scroll(&mut self, world: &mut World, mouse_pos: Vec2, scroll_y: f32) {
        if scroll_y == 0.0 || self.ui_has_focus {
            return;
        }

        let nearest_magnet = world.magnets.iter_mut().min_by(|a, b| {
            a.pos
                .distance_squared(mouse_pos)
                .partial_cmp(&b.pos.distance_squared(mouse_pos))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        match self.current_scroll_mode {
            MouseScrollMode::ResizeMagnet => {
                if let Some(m) = nearest_magnet {
                    m.radius += scroll_y * 15.0;

                    // synchronize particle value
                    if let Some(p) = world
                        .particles
                        .iter_mut()
                        .find(|p| p.is_magnet && p.pos.distance(m.pos) < 1.0)
                    {
                        p.radius = m.radius;
                    }
                }
            }
            MouseScrollMode::AdjustMagnetStrength => {
                if let Some(m) = nearest_magnet {
                    m.strength = (m.strength + scroll_y * 50.0).clamp(-1000.0, 1000.0);

                    // calculation of density based on strength
                    let density = m.strength.abs() / 10000.0;
                    // red for pull, blue for push
                    let new_color = if m.strength >= 0.0 {
                        [1.0, 0.0, 0.0, density]
                    } else {
                        [0.0, 0.0, 1.0, density]
                    };

                    if let Some(p) = world
                        .particles
                        .iter_mut()
                        .find(|p| p.is_magnet && p.pos.distance(m.pos) < 1.0)
                    {
                        p.color = new_color;
                    }
                }
            }
        }
    }
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
    //Reset the Simulation to Default values
    fn reset(&mut self, _world: &mut World) {
        self.spawnrate = None;
        self.gravity = 9.81;
        _world.clear_particles(); // changed to clear_particles so collider stays
    }

    //Basic UI to test Sliders and Buttos
    fn ui(&mut self, _ctx: &egui::Context, _world: &mut World) {
        self.ui_has_focus = _ctx.wants_pointer_input() || _ctx.is_pointer_over_area();

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

            ui.add(egui::Slider::new(&mut self.magnet_radius, 10.0..=1000.0).text("Magnet Size"));
            ui.separator();

            ui.add(
                egui::Slider::new(&mut self.magnet_strength, -1000.0..=1000.0)
                    .text("Magnet Strength"),
            );
            ui.separator();

            ui.label("Left Click Mode:");

            ui.radio_value(
                &mut self.current_mode,
                MouseClickMode::SpawnSingle,
                "Single",
            );
            ui.radio_value(
                &mut self.current_mode,
                MouseClickMode::SpawnCluster,
                "Cluster",
            );
            ui.radio_value(
                &mut self.current_mode,
                MouseClickMode::PlaceMagnet,
                "Magnet",
            );

            ui.separator();

            ui.label("Scroll Mode:");

            ui.radio_value(
                &mut self.current_scroll_mode,
                MouseScrollMode::ResizeMagnet,
                "Resize Magnet",
            );
            ui.radio_value(
                &mut self.current_scroll_mode,
                MouseScrollMode::AdjustMagnetStrength,
                "Adjust Magnet Strength",
            );

            ui.separator();

            ui.label(format!("Partikel: {}", _world.particles.len()));

            if ui.button("Remove Magnets").clicked() {
                _world.magnets.clear();
                _world.particles.retain(|p| !p.is_magnet);
            }

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
        // check standard values
        assert_eq!(scene.gravity, 9.81);
        assert_eq!(scene.particle_radius, 2.0);
        assert_eq!(scene.magnet_radius, 200.0);
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
