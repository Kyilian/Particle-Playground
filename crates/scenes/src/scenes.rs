use egui::Context;
use pp_physics::World;
pub trait Scene {
    fn update(&mut self, world: &mut World, dt: f32);

    fn render(&self, world: &World, frame: &mut [u8]);

    fn on_click(&mut self, world: &mut World, x: f32, y: f32);

    fn ui(&mut self, ctx: &Context, world: &mut World);
}

