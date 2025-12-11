use super::Scene;
use glam::Vec2;
use pp_physics::World;

pub struct FallingParticles {
    gravity: f32,
    spawn_position: Vec2,
    spawnrate: Option<i32>,
}
impl FallingParticles {
    pub fn new() -> Self {
        Self {
            gravity: 500.0, // Standardwert, vielleicht anpassen, bin mir über die Auswirkungen nicht ganz sicher
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
        // world.step(dt); step ist noch nicht implementiert
    }

    //call to the render function
    fn render(&self, _world: &World, frame: &mut [u8]) {

    //To set the background to black
    for pixel in frame.chunks_exact_mut(4) {
        pixel[0] = 0; 
        pixel[1] = 0; 
        pixel[2] = 0; 
        pixel[3] = 255; 
    }
}

    // Spawn a particle with a click
    fn on_click(&mut self, _world: &mut World, _x: f32, _y: f32) {}


    //Reset the Simulation to Default values
    fn reset(&mut self) {
        self.spawnrate = None;
        self.gravity = 500.0;
        self.spawn_position = Vec2::new(100.0, 200.0);
        //self.world.clear();
    }
    
    //Basic UI to test Sliders and Buttos
    fn ui(&mut self, _ctx: &egui::Context, _world: &mut World) {
        egui::Window::new("Falling Particle Simulation").show(_ctx, |ui| {
            ui.label("Test Parameter");

            ui.add(egui::Slider::new(&mut self.gravity, 0.0..=2000.0).text("Gravity"));
            ui.separator();

            ui.label(format!("Partikel: {}", _world.particles.len()));

            if ui.button("Alles zurücksetzen").clicked() {
                self.reset();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_click_spawns_particle_at_position() {
        let mut world = World::new();
        let mut scene = FallingParticles::new();

        scene.on_click(&mut world, 100.0, 200.0);

        assert_eq!(world.particles.len(), 1);
        let p = world.particles[0];
        assert!((p.pos - Vec2::new(100.0, 200.0)).length() < 0.001);
    }
}
