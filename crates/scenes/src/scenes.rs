use egui::Context;
use pp_physics::World;

// Defining a Trait for Scenes,
// so app can the main function can call differnent scenes with the same functions.
pub trait Scene {
    fn update(&mut self, world: &mut World, dt: f32);

    fn render(&self, world: &World, frame: &mut [u8]);

    fn on_click(&mut self, world: &mut World, x: f32, y: f32);

    fn reset(&mut self, world: &mut World);

    fn ui(&mut self, ctx: &Context, world: &mut World);
}
