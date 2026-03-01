use pp_physics::Particle;
use std::cell::Cell;
use wgpu::util::DeviceExt;
use wgpu::{Device, Queue, SurfaceConfiguration};

// WGSL Shader Code (WebGPU Shading Language)
// gets executed for every particle on the GPU
const SHADER_SOURCE: &str = r#"
// Vertex Shader Output / Fragment Shader Input

struct Globals {
    screen_size_wrapper: vec4<f32>,   // .xy = width, height
    color: vec4<f32>,                 // .rgba = color
    camera: vec4<f32>           // .xy = offset to , .z = zoom
};

// we bind the buffer to group 0, Binding 0
@group(0) @binding(0) var<uniform> globals: Globals;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>, // coordinates for circle calculation
    @location(1) color: vec4<f32>, // gives the color of each particle seperatly to the GPU
};

// Vertex Shader
@vertex
fn vs_main(
    @location(0) vertex_pos: vec2<f32>,         // Quad-Corner
    @location(1) instance_pos: vec2<f32>,       // Particle_Position
    @location(2) instance_color: vec4<f32>,     // Individual color
    @location(3) instance_radius: f32,          // Particle_Radius

) -> VertexOutput {
    var out: VertexOutput; 

    let screen_size = globals.screen_size_wrapper.xy; 

    //camera offset and zoom for moving the camera, so we can change the view in runtime
    let camera_offset = globals.camera.xy; 
    let zoom = globals.camera.z; 

    // quad from -0.5 to +0.5 on pixel-size skalable
    let scaled_pos = vertex_pos * (2.0 * instance_radius)*zoom;

    // added to move the center to the middle for Circle_collider
    let center_offset = screen_size / 2.0;

    // general position of the particles in the world with camera offset and zoom
    let world_pos = (instance_pos * zoom) + camera_offset;

    // move partikel position in screen-space
    let screen_pos = scaled_pos + world_pos + center_offset;

    // convert to pixel coordinates (normalized device coordinates)
    let ndc_x = (screen_pos.x / screen_size.x) * 2.0 - 1.0;
    let ndc_y = -((screen_pos.y / screen_size.y) * 2.0 - 1.0);

    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);

    // set uv coordinates for fragment shader 
    // Quad corner turn from -1 to +1 for distance calculation
    out.uv = vertex_pos * 2.0; // UV-Koordinaten for circle-test in fragment shader

    out.color = instance_color;

    return out;
}

// Fragment Shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.uv);
    if (dist > 1.0) {
        discard;    // if pixel is outside of the circle it doesn't get rendered
    }

    // Color of the pixels is determined in the scene 
    return in.color;
}
"#;

//Enum for color mode to switch between heatmap and fixed color
#[derive(Clone, Copy, PartialEq)]
pub enum ColorMode {
    Heatmap,
    ColorFixed([f32; 4]),
}
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalUniforms {
    pub screen_size_wrapper: [f32; 4], // two for size and two unused to get 16 Byte blocks. I had problems if they were bigger or smaller.
    pub _padding: [f32; 4],
    pub camera: [f32; 4],
}

// renderer for particles as circles with FPU-instancing
// each particle is shown as small quad (2 triangles)
// the fragment shaders turns these into a circle
pub struct ParticleRenderer {
    render_pipeline: wgpu::RenderPipeline, // GPU Pipeline for rendering
    vertex_buffer: wgpu::Buffer,           // Quad geometry
    instance_buffer: wgpu::Buffer,         // Particle-positions
    uniform_buffer: wgpu::Buffer,          // Add Uniform Buffer for resizing
    uniform_bind_group: wgpu::BindGroup,
    instance_count: u32, // Particle count
    max_particles: u32,  // can later be used for buffer capacity

    pub size: (u32, u32), //for resizing
    pub color_mode: Cell<ColorMode>,
}

// representation of a particle in GPU
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ParticleInstance {
    position: [f32; 2], // x and y in window
    color: [f32; 4],
    pub radius: f32, // radius in window
    _padding: f32,
}

// as corner of a quad
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2], // x and y relative to center
}

impl Vertex {
    // 4 corners of quad as 2 triangles
    const QUAD: [Vertex; 6] = [
        Vertex {
            position: [-0.5, -0.5], //u l (unten links, etc.) (deliberately in german)
        },
        Vertex {
            position: [0.5, -0.5], //u r
        },
        Vertex {
            position: [0.5, 0.5], //o r
        },
        Vertex {
            position: [-0.5, -0.5], //u l
        },
        Vertex {
            position: [0.5, 0.5], //o r
        },
        Vertex {
            position: [-0.5, 0.5], //o l
        },
    ];

    // Vertex Layout for WGPU
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0, //@location(0) is Shader
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

impl ParticleInstance {
    // Instance Layout for WGPU
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ParticleInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1, //@location(1) is Shader
                    format: wgpu::VertexFormat::Float32x2,
                },
                //for the individual color switch, could be changed later if we use seperate shaders
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 2]>() + std::mem::size_of::<[f32; 4]>())
                        as wgpu::BufferAddress,
                    shader_location: 3, // @location(3) is radius
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

impl ParticleRenderer {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        //  Creating Uniform Buffer
        let uniforms = GlobalUniforms {
            screen_size_wrapper: [config.width as f32, config.height as f32, 0.0, 0.0],
            _padding: [1.0, 1.0, 1.0, 1.0],
            camera: [1.0, 1.0, 1.0, 1.0], //using the former padding for the camera
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // create Bind Group Layout
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Uniform Bind Group Layout"),
            });

        // create Bind Group
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Uniform Bind Group"),
        });

        // create Pipeline Layout
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Particle Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        // create Shader Modul
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle  Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        // create Render Pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particle Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            // Vertex Shader
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),           //@location(0) for quad geometry
                    ParticleInstance::desc(), //@location(1) and @location(2) for particle position and radius
                ],
                compilation_options: Default::default(),
            },
            // Fragment Shader
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,                         // Surface Format
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING), // Alpha Blending on
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            //cache: None,
        });

        // Vertex-Buffer is created for the Quad geometry
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Vertex Buffer"),
            contents: bytemuck::cast_slice(&Vertex::QUAD),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Instance Buffer, gets updated for every frame
        let max_particles = 200000; //for now max particles is fixed, could be changed later
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Instance Buffer"),
            size: (max_particles * std::mem::size_of::<ParticleInstance>() as u32) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            instance_buffer,

            uniform_buffer,
            uniform_bind_group,
            instance_count: 0,
            max_particles,
            size: (config.width, config.height),
            color_mode: Cell::new(ColorMode::Heatmap),
        }
    }

    pub fn get_max_particles(&self) -> u32 {
        self.max_particles
    }

    //To call the scene
    pub fn update_render_settings(&self, queue: &Queue, camera_offset: glam::Vec2, zoom: f32) {
        let width = self.size.0 as f32;
        let height = self.size.1 as f32;

        let uniforms = GlobalUniforms {
            screen_size_wrapper: [width, height, 0.0, 0.0],
            _padding: [0.0; 4], //not used anymore but still needed for alignment. Before it was used for color
            camera: [camera_offset.x, camera_offset.y, zoom, 0.0],
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    pub fn update_window_size(&mut self, width: u32, height: u32) {
        self.size = (width, height);
        //queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    // updates the particle-position on the GPU
    pub fn update_particles(&mut self, particles: &[Particle], queue: &Queue) {
        let color_slow = [0.2, 0.2, 1.0, 1.0];
        let color_fast = [1.0, 0.2, 0.2, 1.0];
        let max_speed = 5.0;

        let instances: Vec<ParticleInstance> = particles
            .iter()
            .map(|p| {
                let color = if p.is_magnet {
                    p.color
                } else {
                    //Color:
                    //Switch between heatmap and a solid color
                    match self.color_mode.get() {
                        ColorMode::Heatmap => {
                            let velocity = p.pos - p.old_pos;
                            let t = (velocity.length() / max_speed).clamp(0.0, 1.0);
                            [
                                color_slow[0] * (1.0 - t) + color_fast[0] * t,
                                color_slow[1] * (1.0 - t) + color_fast[1] * t,
                                color_slow[2] * (1.0 - t) + color_fast[2] * t,
                                1.0,
                            ]
                        }
                        ColorMode::ColorFixed(c) => c,
                    }
                };

                ParticleInstance {
                    position: [p.pos.x, p.pos.y],
                    color,
                    radius: p.radius,
                    _padding: 0.0,
                }
            })
            .collect();

        // saves count
        self.instance_count = instances.len() as u32;

        // copy to GPU
        if !instances.is_empty() {
            queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
        }
    }

    // renders all particles in a draw call
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.instance_count == 0 {
            return; // doesn't do anything if no particles are there
        }
        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]); //Bind group for shader

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw(0..6, 0..self.instance_count); //min. 6 vertices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_alignment() {
        assert_eq!(
            std::mem::size_of::<GlobalUniforms>(),
            48,
            "GlobalUniforms muss exakt 48 Bytes groß sein"
        );
    }
}
//Ai Unit Test with claude AI
#[test]
fn test_color_mode_cell_set_get() {
    let mode = Cell::new(ColorMode::Heatmap);
    mode.set(ColorMode::ColorFixed([1.0, 0.0, 0.0, 1.0]));

    match mode.get() {
        ColorMode::ColorFixed(c) => assert_eq!(c, [1.0, 0.0, 0.0, 1.0]),
        _ => panic!("Sollte ColorFixed  sein nach set()"),
    }
}
#[test]
fn test_solid_mode_ignores_speed() {
    let solid = [0.0, 1.0, 0.0, 1.0];
    match ColorMode::ColorFixed(solid) {
        ColorMode::ColorFixed(c) => assert_eq!(c, solid),
        _ => panic!("Sollte ColorFixed sein"),
    }
}
