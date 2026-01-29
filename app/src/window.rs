use glam::Vec2;

use pp_physics::World;

use pp_render::{ParticleRenderer, RenderContext};
use pp_scenes::{Scene, SceneType};
use std::sync::Arc; //Arc for dual ownership
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::{
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

enum AppState {
    Launcher {
        selected_scene: Option<SceneType>,
    },
    Running {
        scene: Box<dyn Scene>,
        scene_type: SceneType,
        world: World,
    },
}

pub struct RenderWindow {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    particle_renderer: ParticleRenderer,

    app_state: AppState,

    pub is_minimized: bool,

    egui_renderer: egui_wgpu::Renderer,
    egui_state: egui_winit::State,
}

impl RenderWindow {
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut last_time = std::time::Instant::now();

        let mut world = World::new();

        //creates fps calculation variables
        let mut fps_acc_time: f32 = 0.0;
        let mut fps_frames: u32 = 0;
        let mut fps: f32 = 0.0;

        // creates event loop and window
        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(
            //Arc is needed because window needs to be owned by the buffer AND the surface
            WindowBuilder::new()
                .with_title("Particle Playground")
                .with_inner_size(winit::dpi::LogicalSize::new(
                    world.WINDOW_WIDTH,
                    world.WINDOW_HEIGHT,
                ))
                .with_resizable(true)
                .build(&event_loop)?,
        );

        // initializes WGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // creates surface
        let surface = instance.create_surface(Arc::clone(&window))?;

        // requests adapter
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or("Failed to find an appropriate adapter")?;

        // requests device and queue
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ))?;

        // configures the surface of the screen
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: world.WINDOW_WIDTH,
            height: world.WINDOW_HEIGHT,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let egui_ctx = egui::Context::default();

        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::viewport::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
        );

        let egui_renderer = egui_wgpu::Renderer::new(&device, config.format, None, 1);

        //adding so the circle_collider is stays in the center while resizing
        //creates particle renderer
        let particle_renderer = ParticleRenderer::new(&device, &config);

        // creates render window state
        let mut render_window = Self {
            surface,
            device,
            queue,
            config,
            particle_renderer,
            app_state: AppState::Launcher {
                selected_scene: Some(SceneType::FallingParticles),
            },
            is_minimized: false,
            egui_renderer,
            egui_state,
        };

        // physics world + mausposition
        let mut mouse_pos = Vec2::ZERO;

        //Adding a const time step so the pixels dont excelerate when the window is resized
        const TIME_STEP: f32 = 1.0 / 120.0; // 60 Hz Physik
        let mut accumulator = 0.0; // "Zeit-Speicher"

        // runs the event loop
        event_loop.run(move |event, elwt| {
            match event {
                // 1. HAUPT-BLOCK: FENSTER EVENTS
                Event::WindowEvent {
                    event: ref win_event,
                    ..
                } => {
                    let response = render_window
                        .egui_state
                        .on_window_event(&*window, &win_event);

                    // ÄNDERUNG: Hier stand vorher 'match event'.
                    // Wir matchen jetzt direkt auf 'win_event', damit wir die Struktur nicht doppeln.
                    match win_event {
                        WindowEvent::CloseRequested => {
                            elwt.exit();
                        }
                        WindowEvent::KeyboardInput { event, .. } => {
                            if event.state == ElementState::Pressed {
                                if let Key::Named(NamedKey::Escape) = event.logical_key {
                                    // ESC: Zurück zum Launcher
                                    match &render_window.app_state {
                                        AppState::Running { .. } => {
                                            render_window.return_to_launcher();
                                        }
                                        AppState::Launcher { .. } => {
                                            elwt.exit();
                                        }
                                    }
                                }
                            }
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            let half_width = render_window.config.width as f32 / 2.0;
                            let half_height = render_window.config.height as f32 / 2.0;
                            mouse_pos = glam::Vec2::new(
                                position.x as f32 - half_width,
                                position.y as f32 - half_height,
                            );
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            if *state == ElementState::Pressed {
                                if let AppState::Running { scene, world, .. } =
                                    &mut render_window.app_state
                                {
                                    let is_left = *button == MouseButton::Left;
                                    let is_right = *button == MouseButton::Right;
                                    let is_middle = *button == MouseButton::Middle;

                                    scene.on_click(world, mouse_pos, is_right, is_left, is_middle);
                                }
                            }
                        }
                        WindowEvent::Resized(physical_size) => {
                            render_window.resize(physical_size.width, physical_size.height);
                        }
                        _ => {}
                    }
                }

                Event::AboutToWait => {
                    let current_time = std::time::Instant::now();
                    let mut frame_time = (current_time - last_time).as_secs_f32();

                    if frame_time > 0.25 {
                        frame_time = 0.25;
                    }

                    fps_acc_time += frame_time;
                    fps_frames += 1;

                    if fps_acc_time >= 0.5 {
                        fps = fps_frames as f32 / fps_acc_time;
                        fps_acc_time = 0.0;
                        fps_frames = 0;
                    }

                    last_time = current_time;
                    accumulator += frame_time;

                    match &mut render_window.app_state {
                        AppState::Running { scene, world, .. } => {
                            world.fps = fps;

                            while accumulator >= TIME_STEP {
                                scene.update(world, TIME_STEP);
                                accumulator -= TIME_STEP;
                            }
                        }
                        _ => {}
                    }

                    if let AppState::Running { world, .. } = &render_window.app_state {
                        println!("FPS: {:.1} | Particles: {}", fps, world.particles.len());
                        render_window
                            .particle_renderer
                            .update_particles(&world.particles, &render_window.queue);
                    }

                    let raw_input = render_window.egui_state.take_egui_input(&*window);
                    render_window.egui_state.egui_ctx().begin_frame(raw_input);

                    render_window.render_ui();

                    let full_output = render_window.egui_state.egui_ctx().end_frame();

                    match render_window.render(full_output) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => render_window
                            .resize(render_window.config.width, render_window.config.height),
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            eprintln!("Out of memory!");
                            elwt.exit();
                        }
                        Err(e) => eprintln!("Render error: {:?}", e),
                    }
                    window.request_redraw();
                }
                _ => {}
            }
            elwt.set_control_flow(ControlFlow::Poll);
        })?;
        Ok(())
    }

    fn render_ui(&mut self) {
        let ctx = self.egui_state.egui_ctx().clone();

        // check which state is active and do action accordingly
        let mut action: Option<SceneType> = None;
        let mut should_return = false;

        match &mut self.app_state {
            AppState::Launcher { selected_scene } => {
                egui::CentralPanel::default().show(&ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.0);
                        ui.heading("🎮 Particle Playground");
                        ui.add_space(10.0);
                        ui.label("Wähle eine Simulation:");
                        ui.add_space(30.0);
                    });

                    ui.vertical_centered(|ui| {
                        for scene_type in SceneType::all() {
                            let is_selected = *selected_scene == Some(*scene_type);

                            egui::Frame::none()
                                .fill(if is_selected {
                                    egui::Color32::from_rgb(60, 60, 80)
                                } else {
                                    egui::Color32::from_rgb(40, 40, 50)
                                })
                                .rounding(8.0)
                                .inner_margin(12.0)
                                .show(ui, |ui| {
                                    ui.set_min_width(300.0);
                                    ui.horizontal(|ui| {
                                        ui.radio_value(selected_scene, Some(*scene_type), "");
                                        ui.vertical(|ui| {
                                            ui.strong(scene_type.display_name());
                                            ui.label(scene_type.description());
                                        });
                                    });
                                });

                            ui.add_space(8.0);
                        }

                        ui.add_space(20.0);

                        let start_enabled = selected_scene.is_some();

                        if ui
                            .add_enabled(
                                start_enabled,
                                egui::Button::new("▶ Starten").min_size(egui::vec2(120.0, 40.0)),
                            )
                            .clicked()
                        {
                            action = *selected_scene;
                        }
                    });
                });
            }
            AppState::Running {
                scene,
                scene_type,
                world,
            } => {
                egui::Window::new("Navigation")
                    .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
                    .resizable(false)
                    .collapsible(false)
                    .show(&ctx, |ui| {
                        ui.label(format!("Aktive Scene: {}", scene_type.display_name()));
                        ui.separator();

                        if ui.button("🔙 Zurück zur Auswahl").clicked() {
                            should_return = true;
                        }

                        ui.label("(oder ESC drücken)");
                    });

                // scene-specific ui
                scene.ui(&ctx, world);
            }
        }

        if let Some(scene_type) = action {
            self.start_scene(scene_type);
        }

        if should_return {
            self.return_to_launcher();
        }
    }

    fn start_scene(&mut self, scene_type: SceneType) {
        println!("Starte Scene: {}", scene_type.display_name());

        let mut world = World::new();
        let scene = scene_type.create_scene(&mut world);

        self.app_state = AppState::Running {
            scene,
            scene_type,
            world,
        };
    }

    fn return_to_launcher(&mut self) {
        println!("Zurück zum Launcher...");

        self.app_state = AppState::Launcher {
            selected_scene: Some(SceneType::FallingParticles),
        };
    }

    fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_height == 0 || new_width == 0 {
            self.is_minimized = true;
            return;
        }
        self.is_minimized = false;

        if new_width > 0
            && new_height > 0
            && (new_width != self.config.width || new_height != self.config.height)
        {
            self.config.width = new_width;
            self.config.height = new_height;
            self.surface.configure(&self.device, &self.config);

            if let AppState::Running { world, .. } = &mut self.app_state {
                world.WINDOW_HEIGHT = new_height;
                world.WINDOW_WIDTH = new_width;
            }

            //new renderer incase window gets resized
            //changed it to the Uniform buffer
            self.particle_renderer
                .update_window_size(&self.queue, new_width, new_height);
            println!("Resized to: {}x{}", new_width, new_height);
        }
    }

    fn render(&mut self, full_output: egui::FullOutput) -> Result<(), wgpu::SurfaceError> {
        //render error if size of the window is zero otherwise
        if self.is_minimized {
            return Ok(());
        }

        // gets the current frame
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // upload all resources for egui
        for (id, image_delta) in full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, id, &image_delta);
        }

        let paint_jobs = self
            .egui_state
            .egui_ctx()
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: full_output.pixels_per_point,
        };

        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        for id in full_output.textures_delta.free {
            self.egui_renderer.free_texture(&id);
        }

        // creates render pass with black clear color to remove artifacts
        {
            //changed the render pass that we can define the color in the scene itself
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let AppState::Running { scene, world, .. } = &self.app_state {
                let ctx = RenderContext {
                    particle_renderer: &self.particle_renderer,
                    queue: &self.queue,
                    device: &self.device,
                };
                scene.render(world, &ctx, &mut render_pass);
            }

            self.egui_renderer
                .render(&mut render_pass, &paint_jobs, &screen_descriptor);

            // render pass ends here automatically when dropped
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    // Public access to WGPU resources for particle rendering
    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn config(&self) -> &SurfaceConfiguration {
        &self.config
    }
}

//Ai Unit Tests Gemini
#[cfg(test)]
mod tests {
    #[test]
    fn test_coordinate_centering_logic() {
        let screen_size = [800.0, 600.0];
        let particle_pos = [0.0, 0.0];

        // Simulation der Shader-Logik:
        let center_offset = [screen_size[0] / 2.0, screen_size[1] / 2.0];
        let screen_pos = [
            particle_pos[0] + center_offset[0],
            particle_pos[1] + center_offset[1],
        ];

        let ndc_x: f64 = (screen_pos[0] / screen_size[0]) * 2.0 - 1.0;
        let ndc_y: f64 = (screen_pos[1] / screen_size[1]) * 2.0 - 1.0;

        assert!(
            ndc_x.abs() < 1e-6,
            "NDC X sollte 0 sein, ist aber {}",
            ndc_x
        );
        assert!(
            ndc_y.abs() < 1e-6,
            "NDC Y sollte 0 sein, ist aber {}",
            ndc_y
        );
    }

    #[test]
    fn test_mouse_to_world_conversion() {
        let window_size = (800.0, 600.0);

        let mouse_top_left = (0.0, 0.0);
        let world_x = mouse_top_left.0 - (window_size.0 / 2.0);
        let world_y = mouse_top_left.1 - (window_size.1 / 2.0);

        assert_eq!(world_x, -400.0);
        assert_eq!(world_y, -300.0);

        let mouse_center = (400.0, 300.0);
        let world_center_x = mouse_center.0 - (window_size.0 / 2.0);
        let world_center_y = mouse_center.1 - (window_size.1 / 2.0);

        assert_eq!(world_center_x, 0.0);
        assert_eq!(world_center_y, 0.0);
    }
}
