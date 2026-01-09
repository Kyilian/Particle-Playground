use egui::Context;
use glam::Vec2;
use pp_physics::World;
use pp_render::RenderContext;

// Defining a Trait for Scenes,
// so app can the main function can call differnent scenes with the same functions.
pub trait Scene {
    fn update(&mut self, world: &mut World, dt: f32);

    fn render(&self, world: &World, ctx: &mut RenderContext);

    fn on_click(
        &mut self,
        world: &mut World,
        mouse_pos: Vec2,
        right_click: bool,
        left_click: bool,
        is_middle: bool,
    );

    fn reset(&mut self, world: &mut World);

    fn ui(&mut self, ctx: &Context, world: &mut World);
}
