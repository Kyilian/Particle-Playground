use pp_app::window::RenderWindow;

use pp_scenes::nbody_scene::NBodyScene;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting particle playground...");

    RenderWindow::run()?;

    Ok(())
}
