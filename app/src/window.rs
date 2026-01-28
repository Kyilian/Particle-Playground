use glam::Vec2;
use pp_physics::{CircleCollider, World};
use pp_render::{ParticleRenderer, RenderContext};
use pp_scenes::Scene;
use std::sync::Arc; //Arc for dual ownership
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::{
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

pub struct RenderWindow {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    particle_renderer: ParticleRenderer,

    current_scene: Box<dyn Scene>,
    world: World,

    pub is_minimized: bool,

    egui_renderer: egui_wgpu::Renderer,
    egui_state: egui_winit::State,
}

impl RenderWindow {
    pub fn run(initial_scene: Box<dyn Scene>) -> Result<(), Box<dyn std::error::Error>> {
        let mut last_time = std::time::Instant::now();

        let mut world = World::new();

        //creates fps calculation variables
        let mut fps_acc_time: f32 = 0.0;
        let mut fps_frames: u32 = 0;

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

        world.add_circle_collider(CircleCollider {
            center: Vec2::new(0.0, 0.0), // (0,0) is now the center
            radius: 250.0,
        });

        // creates render window state
        let mut render_window = Self {
            surface,
            device,
            queue,
            config,
            particle_renderer,
            current_scene: initial_scene,
            world,

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
                                    elwt.exit();
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
                            if !response.consumed {
                                // Deine Klick-Logik (unverändert übernommen)
                                if *state == ElementState::Pressed {
                                    let is_left = *button == MouseButton::Left;
                                    let is_right = *button == MouseButton::Right;
                                    let is_middle = *button == MouseButton::Middle;

                                    render_window.current_scene.on_click(
                                        &mut render_window.world,
                                        mouse_pos,
                                        is_right,
                                        is_left,
                                        is_middle,
                                    );
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
                        let fps: f32 = fps_frames as f32 / fps_acc_time;

                        render_window.world.fps = fps;
                        println!(
                            "FPS: {:.1} | Particles: {}",
                            fps,
                            render_window.world.particles.len()
                        );
                        fps_acc_time = 0.0;
                        fps_frames = 0;
                    }

                    last_time = current_time;
                    accumulator += frame_time;

                    while accumulator >= TIME_STEP {
                        render_window
                            .current_scene
                            .update(&mut render_window.world, TIME_STEP);
                        accumulator -= TIME_STEP;
                    }

                    render_window
                        .particle_renderer
                        .update_particles(&render_window.world.particles, &render_window.queue);

                    let raw_input = render_window.egui_state.take_egui_input(&*window);
                    render_window.egui_state.egui_ctx().begin_frame(raw_input);

                    // Deine UI Definition
                    render_window.current_scene.ui(
                        render_window.egui_state.egui_ctx(),
                        &mut render_window.world,
                    );

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

            self.world.WINDOW_HEIGHT = new_height;
            self.world.WINDOW_WIDTH = new_width;

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

            let ctx = RenderContext {
                particle_renderer: &self.particle_renderer,
                queue: &self.queue,
                device: &self.device,
            };

            self.current_scene
                .render(&self.world, &ctx, &mut render_pass);

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
