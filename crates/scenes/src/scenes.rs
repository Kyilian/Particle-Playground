use egui::Context;
use glam::Vec2;
use pp_physics::World;
use pp_render::RenderContext;

// Defining a Trait for Scenes,
// so app can the main function can call differnent scenes with the same functions.
pub trait Scene {
    fn update(&mut self, world: &mut World, dt: f32);

    fn render<'rpass>(
        &self,
        world: &World,
        ctx: &RenderContext<'rpass>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    );

    //defining the input in the scene itself and not in the window for customizability
    fn on_click(
        &mut self,
        world: &mut World,
        mouse_pos: Vec2,
        right_click: bool,
        left_click: bool,
        is_middle: bool,
    );
    // To drag the camera we need to know when the middle mouse button is pressed and released
    fn on_mouse_move(&mut self, world: &mut World, mouse_pos: Vec2);
    fn on_mouse_release(
        &mut self,
        world: &mut World,
        mouse_pos: Vec2,
        right_click: bool,
        left_click: bool,
        is_middle: bool,
    );

    fn handle_scroll(&mut self, world: &mut World, mouse_pos: Vec2, scroll_y: f32);

    fn reset(&mut self, world: &mut World);

    fn ui(&mut self, ctx: &Context, world: &mut World);
}
