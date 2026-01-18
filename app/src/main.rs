mod window;

use pp_scenes::falling_particles::FallingParticles;
use window::RenderWindow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_scene = Box::new(FallingParticles::new());

    println!("Starte Particle Playground...");

    RenderWindow::run(start_scene)
}
