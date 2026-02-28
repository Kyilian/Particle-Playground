use pp_app::window::RenderWindow;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting particle playground...");

    RenderWindow::run()?;

    Ok(())
}
