mod window;

use window::RenderWindow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("Starting particle playground...");

    RenderWindow::run()?;

    Ok(())
}