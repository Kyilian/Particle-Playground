use pp_app::window::RenderWindow;

use pp_scenes::benchmark_scene::BenchmarkScene;
use pp_scenes::falling_particles::FallingParticles;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_scene = Box::new(BenchmarkScene::new());

    println!("Starte Particle Playground...");

    RenderWindow::run(start_scene)
}
 