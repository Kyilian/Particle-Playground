use pp_physics::Particle;
use wgpu::util::DeviceExt;
use wgpu::{Device, Queue, SurfaceConfiguration};

const SHADER_SOURCE: &str = r#"
// Vertex Shader Output / Fragment Shader Input
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>, // Für Kreis berechnung
};

// Vertex Shader
@vertex
fn vs_main(
    @location(0) vertex_pos: vec2<f32>,     // Quad-Ecke
    @location(1) instance_pos: vec2<f32>,    // Partikel_Position
) -> VertexOutput {
    var out: VertexOutput; 

    let particle_size: f32 = 12.0;      //      ----------!!als Parameter setzen später!!----------
    let screen_width: f32 = 800.0;      // man kann hier nicht auf die Rust-konstanten zurückgreifen
    let screen_height: f32 = 600.0;     // Funktion die sich wie fenster an resize anpasst?

    let scaled_pos = vertex_pos * particle_size;

    let screen_pos = scaled_pos + instance_pos;

    //zu Pixel koordinaten konvertieren
    let ndc_x = (screen_pos.x / screen_width) * 2.0 - 1.0;
    let ndc_y = -((screen_pos.y / screen_height) * 2.0 - 1.0);

    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);

    out.uv = vertex_pos * 2.0; //UV-Koordinaten für Kreis-Test im Fragment Shader

    return out;
}

//Fragment Shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.uv);
    if (dist > 1.0) {
        discard;    //wenn Pixel außerhalb des Kreises wird er nicht gerendert
    }

    return vec4<f32>(1.0, 1.0, 1.0, 1.0); //Pixel im Kreis werden weiß gezeichnet
}

"#;

pub struct ParticleRenderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_count: u32, //how many particles
    max_particles: u32,  //can be used later to give out how many particles there are
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ParticleInstance {
    position: [f32; 2],
}

//jeder Partikel wird als kleines Quad bzw Rechteck dargestellt
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    //4 ecken des Quadrats als 2 dreiecke
    const QUAD: [Vertex; 6] = [
        Vertex {
            position: [-0.5, -0.5],
        }, //u l
        Vertex {
            position: [0.5, -0.5],
        }, //u r
        Vertex {
            position: [0.5, 0.5],
        }, //o r
        Vertex {
            position: [-0.5, -0.5],
        }, //u l
        Vertex {
            position: [0.5, 0.5],
        }, //o r
        Vertex {
            position: [-0.5, 0.5],
        }, //o l
    ];

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
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ParticleInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 1, //@location(1) is Shader
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

impl ParticleRenderer {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        // Shader Modul wird erstellt
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle  Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        // Render Pipeline wird erstellt
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particle Render Pipeline"),
            layout: None, //automatisches layout
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Vertex::desc(),           //@location(0) für quad geometrie
                    ParticleInstance::desc(), //@location(1) für partikel position
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            cache: None,
        });

        // Vertex-Buffer wird erstellt für die Quad Geometrie
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Vertex Buffer"),
            contents: bytemuck::cast_slice(&Vertex::QUAD),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let max_particles = 10000; //max 10k particles
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
            instance_count: 0,
            max_particles,
        }
    }

    pub fn update_particles(&mut self, particles: &[Particle], queue: &Queue) {
        //Particle -> ParticleInstance
        let instances: Vec<ParticleInstance> = particles
            .iter()
            .map(|p| ParticleInstance {
                position: [p.pos.x, p.pos.y],
            })
            .collect();

        //speichert anzahl
        self.instance_count = instances.len() as u32;

        //kopiert zu GPU
        if !instances.is_empty() {
            queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.instance_count == 0 {
            return;
        }
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw(0..6, 0..self.instance_count); //mind. 6 vertices
    }
}
