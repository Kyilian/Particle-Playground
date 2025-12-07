use std::sync::Arc;     //Arc for dual ownership
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};

pub struct RenderWindow {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
}

impl RenderWindow {
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        // creates event loop and window
        let event_loop = EventLoop::new()?;
        let window = Arc::new(      //Arc is needed because window needs to be owned by the buffer AND the surface
            WindowBuilder::new()
                .with_title("Particle Playground")
                .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
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
            power_preference: wgpu::PowerPreference::default(),
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

        // creates render window state
        let mut render_window = Self {
            surface,
            device,
            queue,
            config,
        };

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
                    event: WindowEvent::Resized(physical_size),  //resizing the window
                    ..
                } => {
                    render_window.resize(physical_size.width, physical_size.height);
                }
                Event::AboutToWait => {  //renders when all pending events are finished
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

            elwt.set_control_flow(ControlFlow::Poll);       //sets loop to run as fast as possible and constantly fire AboutToWait -> renders every frame
        })?;

        Ok(())
    }

    fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            self.config.width = new_width;
            self.config.height = new_height;
            self.surface.configure(&self.device, &self.config);
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
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
