use pp_app::window::RenderWindow;

use pp_scenes::barnes_hut_nbody::BarnesHutNbody;
use pp_scenes::falling_particles::FallingParticles;
use pp_scenes::nbody_scene::NBodyScene;
use pp_scenes::test_scene::TestScene;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_scene = Box::new(NBodyScene::new());

    println!("Starting particle playground...");

    RenderWindow::run()?;

    Ok(())
}
