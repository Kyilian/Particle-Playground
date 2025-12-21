use crate::{ParticleRenderer, WINDOW_HEIGHT, WINDOW_WIDTH};
use glam::Vec2;
use pp_physics::{CircleCollider, Particle, World};
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
}

// funktion für nearest neighbour search
fn find_nearest_particle(world: &World, mouse_pos: Vec2) -> Option<usize> {
    let mut nearest: Option<usize> = None;
    let mut nearest_dist2 = f32::MAX;

    for (i, p) in world.particles.iter().enumerate() {
        let d = p.pos - mouse_pos;
        let dist2 = d.length_squared();

        if dist2 < nearest_dist2 {
            nearest_dist2 = dist2;
            nearest = Some(i);
        }
    }
    nearest
}

impl RenderWindow {
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut last_time = std::time::Instant::now();
        // creates event loop and window
        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(
            //Arc is needed because window needs to be owned by the buffer AND the surface
            WindowBuilder::new()
                .with_title("Particle Playground")
                .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_resizable(false)
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
                memory_hints: wgpu::MemoryHints::Performance,
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
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        //creates particle renderer
        let particle_renderer = ParticleRenderer::new(&device, &config);

        // creates render window state
        let mut render_window = Self {
            surface,
            device,
            queue,
            config,
            particle_renderer,
        };

        // physics world + mausposition
        let mut world = World::new();
        let mut mouse_pos = Vec2::ZERO;

        //collider setup
        world.add_circle_collider(CircleCollider {
            center: Vec2::new(WINDOW_WIDTH as f32 / 2.0, WINDOW_HEIGHT as f32 / 2.0),
            radius: 250.0,
        });

        // runs the event loop
        event_loop.run(move |event, elwt| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested, //closing the window
                    ..
                } => {
                    elwt.exit();
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { event, .. }, //closing the window when pressing ESC
                    ..
                } => {
                    if event.state == ElementState::Pressed {
                        if let Key::Named(NamedKey::Escape) = event.logical_key {
                            elwt.exit();
                        }
                    }
                }
                // mausposition ausgeben
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    mouse_pos = Vec2::new(position.x as f32, position.y as f32);
                    println!("Mouse at: x = {}, y = {}", position.x, position.y);
                }
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { state, button, .. },
                    ..
                } => {
                    if state == ElementState::Pressed && button == MouseButton::Left {
                        // Linksklick: Partikel spawnen
                        let id = world.add_particle(Particle::new(mouse_pos));
                        println!("Spawned particle #{id} at {:?}", mouse_pos);
                    }
                    if state == ElementState::Pressed && button == MouseButton::Right {
                        // Rechtsklick: nächsten Partikel finden
                        if let Some(nearest) = find_nearest_particle(&world, mouse_pos) {
                            let p = &world.particles[nearest];
                            println!(
                                "Nearest particle is #{nearest} at pos {:?} to mouse {:?}",
                                p.pos, mouse_pos
                            );
                        } else {
                            println!("No particle close to {:?}", mouse_pos);
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(physical_size), //resizing the window
                    ..
                } => {
                    render_window.resize(physical_size.width, physical_size.height);
                }
                Event::AboutToWait => {
                    //renders when all pending events are finished

                    //physic update
                    let current_time = std::time::Instant::now();
                    let dt = (current_time - last_time).as_secs_f32();
                    last_time = current_time;
                    let time_scale = 2.0; // 2x schneller, geschwindigkeit einstellen
                    world.step(dt * time_scale);

                    //copy to GPU
                    render_window
                        .particle_renderer
                        .update_particles(&world.particles, &render_window.queue);

                    // renders frame
                    match render_window.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            render_window.resize(WINDOW_WIDTH, WINDOW_HEIGHT)
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            eprintln!("Out of memory!");
                            elwt.exit();
                        }
                        Err(e) => eprintln!("Render error: {:?}", e),
                    }

                    // requests redraw
                    window.request_redraw();
                }
                _ => {}
            }

            elwt.set_control_flow(ControlFlow::Poll); //sets loop to run as fast as possible and constantly fire AboutToWait -> renders every frame
        })?;

        Ok(())
    }

    fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            self.config.width = new_width;
            self.config.height = new_height;
            self.surface.configure(&self.device, &self.config);

            //new renderer incase window gets resized
            self.particle_renderer = ParticleRenderer::new(&self.device, &self.config);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // gets the current frame
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // creates command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // creates render pass with black clear color to remove artifacts
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
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

            //draw particle
            self.particle_renderer.render(&mut render_pass);

            // render pass ends here automatically when dropped
        }

        // submit commands
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
