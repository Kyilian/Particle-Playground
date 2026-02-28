use super::Scene;
use glam::Vec2;
use pp_physics::{Particle, World};
use pp_render::RenderContext;
use rand::prelude::*;

//A N-Body simutlation using the Barnes-Hut algorithm to optimize the gravity calculation
//Controlls:
//Left Click: Spawn a single particle at the mouse position
//Right Click: Spawn a small galaxy of particles around the mouse position with a random velocity to simulate a small galaxy
//Middle Click: Spawn a heavy particle that attracts other particles with a strong force, also allows to drag the camera
//Mouse Scroll: Zoom in and out to the mouse position
pub struct BarnesHutNbody {
    gravity: glam::Vec2,
    mass: f32,
    particle_radius: f32,
    camera_zoom: f32,
    camera_offset: Vec2,
    color: [f32; 4],

    is_dragging: bool,
    last_mouse_pos: Vec2,

    colision_on: bool,
}

impl BarnesHutNbody {
    pub fn new() -> Self {
        Self {
            gravity: Vec2::new(1.0, 1.0),
            mass: 1.0,
            color: [1.0, 0.2, 0.2, 1.0],
            particle_radius: 2.0,
            camera_zoom: 1.0,
            camera_offset: Vec2::ZERO,
            is_dragging: false,
            last_mouse_pos: Vec2::ZERO,
            colision_on: false,
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
        _world.gravity = self.gravity;
        _world.barnes_hut_nbody_step(_dt);
        _world.particle_radius = self.particle_radius;

        if self.colision_on {
            _world.solve_particle_collisions_grid();
        }
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

        //spawn a single particle
        if left_click {
            let id = world.add_particle(Particle::new_with_mass(
                world_mouse_pos,
                self.mass,
                self.particle_radius,
            ));
            println!("Spawned particle #{id} at {:?}", world_mouse_pos);
        }

        if right_click {
            //spawn a galaxy of particles around the mouse position
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
        // Middle click: at the moment, spawnes a heavy particle and drags the camera
        if is_middle {
            self.is_dragging = true;
            self.last_mouse_pos = mouse_pos;
            let r = self.particle_radius * 3.0;
            let mut p = Particle::new_with_mass(world_mouse_pos, 20000.0, r);

            p.old_pos = p.pos;

            world.add_particle(p);
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

        println!("Zoom zur Maus! Neuer Zoom: {:.2}", self.camera_zoom);
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
            ui.label("Parameter");

            ui.add(
                egui::Slider::new(&mut self.particle_radius, 1.0..=100.0)
                    .text("Particle Size and Mass"),
            );
            ui.separator();

            ui.add(egui::Slider::new(&mut self.gravity.y, 0.0..=1000.0).text("Gravity "));

            ui.label(format!("Partikel: {}", _world.particles.len()));

            if ui.button("Alles zurücksetzen").clicked() {
                self.reset(_world);
            }
            ui.separator();

            if ui.button("Spawn Galaxy").clicked() {
                _world.add_galaxy(3000);
            }

            ui.separator();
            ui.checkbox(&mut self.colision_on, "Colisions On/Off").on_hover_text("Toggle particle collisions. Warning: Can be very performance intensive with many particles!");
        });
    }
}

//Gemini Unit Tests
#[cfg(test)]
mod camera_tests {
    use glam::Vec2;

    // Hilfsfunktion: Simuliert die Umrechnung vom Monitor in die Physik-Welt (wie beim Mausklick)
    fn screen_to_world(screen_pos: Vec2, offset: Vec2, zoom: f32) -> Vec2 {
        (screen_pos - offset) / zoom
    }

    // Hilfsfunktion: Simuliert die Umrechnung von der Physik-Welt auf den Monitor (wie im Shader)
    fn world_to_screen(world_pos: Vec2, offset: Vec2, zoom: f32) -> Vec2 {
        (world_pos * zoom) + offset
    }

    #[test]
    fn test_screen_to_world_conversion() {
        let zoom = 2.0;
        let offset = Vec2::new(100.0, 50.0);
        let mouse_screen_pos = Vec2::new(500.0, 350.0);

        // Rechnung: (500 - 100) / 2 = 200  |  (350 - 50) / 2 = 150
        let expected_world_pos = Vec2::new(200.0, 150.0);
        let calculated_world_pos = screen_to_world(mouse_screen_pos, offset, zoom);

        assert_eq!(
            calculated_world_pos, expected_world_pos,
            "Die Mauskoordinaten wurden falsch in die Welt umgerechnet!"
        );
    }

    #[test]
    fn test_world_to_screen_conversion() {
        // Dies testet quasi trocken, ob dein GPU-Shader richtig rechnet
        let zoom = 0.5; // Rausgezoomt
        let offset = Vec2::new(-50.0, -50.0);
        let particle_world_pos = Vec2::new(1000.0, 800.0);

        // Rechnung: (1000 * 0.5) - 50 = 450  |  (800 * 0.5) - 50 = 350
        let expected_screen_pos = Vec2::new(450.0, 350.0);
        let calculated_screen_pos = world_to_screen(particle_world_pos, offset, zoom);

        assert_eq!(
            calculated_screen_pos, expected_screen_pos,
            "Das Partikel würde an der falschen Stelle auf dem Monitor landen!"
        );
    }

    #[test]
    fn test_roundtrip_conversion() {
        // Testet, ob hin- und zurückrechnen wieder exakt den Startwert ergibt
        let initial_world_pos = Vec2::new(1337.0, -42.0);
        let zoom = 3.5;
        let offset = Vec2::new(123.0, 456.0);

        // 1. In Bildschirm-Koordinaten umwandeln (wie der Shader es macht)
        let screen_pos = world_to_screen(initial_world_pos, offset, zoom);

        // 2. Wieder zurück in Welt-Koordinaten (wie der Mausklick es macht)
        let world_pos_back = screen_to_world(screen_pos, offset, zoom);

        // Erlaubt eine minimale Ungenauigkeit bei Fließkommazahlen (Floats)
        let diff = (initial_world_pos - world_pos_back).length();
        assert!(
            diff < 0.0001,
            "Roundtrip fehlgeschlagen! Start: {:?}, Ende: {:?}",
            initial_world_pos,
            world_pos_back
        );
    }

    #[test]
    fn test_zoom_to_mouse_logic() {
        // Dieser Test prüft die komplizierte "Zoom zum Fadenkreuz"-Mathematik
        let mouse_pos = Vec2::new(300.0, 200.0); // Maus ist starr auf dem Monitor

        let old_zoom = 1.0;
        let old_offset = Vec2::new(0.0, 0.0);

        // Welchen Punkt in der Welt schauen wir uns gerade an?
        let world_target = screen_to_world(mouse_pos, old_offset, old_zoom);

        // Jetzt zoomen wir rein!
        let new_zoom = 2.0;

        // Die Formel aus unserer handle_scroll Funktion:
        let new_offset = mouse_pos - (world_target * new_zoom);

        // Wenn wir jetzt mit dem NEUEN Zoom und dem NEUEN Offset prüfen,
        // was unter unserer starr gebliebenen Maus liegt...
        let new_world_target = screen_to_world(mouse_pos, new_offset, new_zoom);

        // ... MUSS es exakt der gleiche physikalische Punkt sein wie vorher!
        assert_eq!(
            world_target, new_world_target,
            "Der 'Zoom to Mouse' ist verrutscht! Das Ziel unter der Maus hat sich geändert."
        );
    }
}
