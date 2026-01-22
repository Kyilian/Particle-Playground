use pp_app::window::RenderWindow;

use pp_scenes::falling_particles::FallingParticles;
use pp_scenes::test_scene::TestScene;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_scene = Box::new(TestScene::new());

    println!("Starte Particle Playground...");

    RenderWindow::run(start_scene)
}
