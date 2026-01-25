use pp_app::window::RenderWindow;

use pp_scenes::falling_particles::FallingParticles;
use pp_scenes::test_scene::TestScene;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("Starting particle playground...");

    RenderWindow::run()?;

    Ok(())
}
