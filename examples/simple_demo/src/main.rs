use pp_app::window::RenderWindow;

use pp_scenes::falling_particles::FallingParticles;
use pp_scenes::nbody_scene::NBodyScene;
use pp_scenes::test_scene::TestScene;

fn main() -> Result<(), Box<dyn std::error::Error>> {
<<<<<<< HEAD
    env_logger::init();
=======
    let start_scene = Box::new(NBodyScene::new());
>>>>>>> 441624cbc705f0ed848340e243b661d469c7a58a

    println!("Starting particle playground...");

    RenderWindow::run()?;

    Ok(())
}
