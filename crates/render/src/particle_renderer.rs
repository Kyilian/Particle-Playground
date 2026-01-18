use pp_physics::Particle;
use wgpu::util::DeviceExt;
use wgpu::{Device, Queue, SurfaceConfiguration};

// WGSL Shader Code (WebGPU Shading Language)
// Wird auf der GPU für jeden Partikel ausgeführt
const SHADER_SOURCE: &str = r#"
// Vertex Shader Output / Fragment Shader Input

struct Globals {
    screen_size_wrapper: vec4<f32>,   // .xy = width, height
    color: vec4<f32>,                 // .rgba = color
    particle_size_wrapper: vec4<f32>, // .x = size
};

// Wir binden den Buffer an Gruppe 0, Binding 0
@group(0) @binding(0) var<uniform> globals: Globals;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>, // Koordinate für Kreis berechnung
};

// Vertex Shader
@vertex
fn vs_main(
    @location(0) vertex_pos: vec2<f32>,         // Quad-Ecke
    @location(1) instance_pos: vec2<f32>,       // Partikel_Position
) -> VertexOutput {
    var out: VertexOutput; 

    let screen_size = globals.screen_size_wrapper.xy; 
    let particle_size = globals.particle_size_wrapper.x;

    // quad von -0.5 bis +0.5 auf pixel-größe skalieren
    let scaled_pos = vertex_pos * particle_size;

    //hinzugefügt um den Center in die Mitte zu verschieben für Circle_collider
    let center_offset = screen_size / 2.0;

    // partikel position im screen-space verschieben
    let screen_pos = scaled_pos + instance_pos + center_offset;

    // zu Pixel koordinaten konvertieren (normalized device coordinates)
    let ndc_x = (screen_pos.x / screen_size.x) * 2.0 - 1.0;
    let ndc_y = -((screen_pos.y / screen_size.y) * 2.0 - 1.0);

    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);

    // uv koordinate für fragment shader setzen
    // Quad Ecken werden zu -1 bis +1 für Distanz berechnung
    out.uv = vertex_pos * 2.0; //UV-Koordinaten für Kreis-Test im Fragment Shader

    return out;
}

//Fragment Shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.uv);
    if (dist > 1.0) {
        discard;    // wenn Pixel außerhalb des Kreises wird er nicht gerendert
    }

    // Color of the pixels is determined in the scene 
    return globals.color;
}
"#;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalUniforms {
    pub screen_size_wrapper: [f32; 4], //two for size and two unused to get 16 Byte blocks. I had problems if they were bigger or smaller.
    pub color: [f32; 4],
    pub particle_size_wrapper: [f32; 4],
}

// Renderer für Partikel als Kreise mit GPU-Instancing
// jeder partikel wird als kleines Quad (2 Dreiecke) dargestellt,
// der Fragment Shader macht daraus einen Kreis.
pub struct ParticleRenderer {
    render_pipeline: wgpu::RenderPipeline, // GPU Pipeline für Rendering
    vertex_buffer: wgpu::Buffer,           // Quad Geometrie
    instance_buffer: wgpu::Buffer,         // Partikel-positionen
    uniform_buffer: wgpu::Buffer,          //Add Uniform Buffer for resizing
    uniform_bind_group: wgpu::BindGroup,
    instance_count: u32, // Partikelanzahl
    max_particles: u32,  // kann später für Buffer Kapazität benutzt werden

    pub size: (u32, u32), //For resizing
}

// repräsentation eines Partikels in GPU
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ParticleInstance {
    position: [f32; 2], // x und y in Fenster
}

// ein Eckpunkt eines Quads
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2], // x und y relativ zu Zentrum
}

impl Vertex {
    // 4 ecken des Quadrats als 2 dreiecke
    const QUAD: [Vertex; 6] = [
        Vertex {
            position: [-0.5, -0.5], //u l
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

    // Vertex Layout für WGPU
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
    // Instance Layout für WGPU
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
        //  Creating Uniform Buffer
        let uniforms = GlobalUniforms {
            screen_size_wrapper: [config.width as f32, config.height as f32, 0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            particle_size_wrapper: [12.0, 0.0, 0.0, 0.0],
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Bind Group Layout erstellen
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

        // Bind Group erstellen
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Uniform Bind Group"),
        });

        // Pipeline Layout erstellen
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Particle Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        // Shader Modul wird erstellt
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle  Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        // Render Pipeline wird erstellt
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particle Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            // Vertex Shader
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),           //@location(0) für quad geometrie
                    ParticleInstance::desc(), //@location(1) für partikel position
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

        // Vertex-Buffer wird erstellt für die Quad Geometrie
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Vertex Buffer"),
            contents: bytemuck::cast_slice(&Vertex::QUAD),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Instance Buffer, wird für jeden Frame aktualisiert
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

            uniform_buffer,
            uniform_bind_group,
            instance_count: 0,
            max_particles,
            size: (config.width, config.height),
        }
    }

    //To call the scene
    pub fn update_render_settings(&self, queue: &Queue, color: [f32; 4], particle_radius: f32) {
        let width = self.size.0 as f32;
        let height = self.size.1 as f32;

        let uniforms = GlobalUniforms {
            screen_size_wrapper: [width, height, 0.0, 0.0],
            color: color,
            particle_size_wrapper: [particle_radius, 0.0, 0.0, 0.0], // these zeros are placeholders because I had problems if i did not used 16 byte blocks
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }
    // if the changes of the renderer and scene works as I intended, we wouldnt need this function anymore

    //pub fn update_window_size(&self, queue: &Queue, width: u32, height: u32) {
    //    let uniforms = GlobalUniforms {
    //        screen_size: [width as f32, height as f32],
    //        _padding: [0.0, 0.0],
    //    };
    //    queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    //}

    pub fn update_window_size(&mut self, _queue: &Queue, width: u32, height: u32) {
        self.size = (width, height);
        //queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    // Aktualisiert die Partikel-Positionen auf der GPU
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

    // Rendert alle Partikel in einem Draw-Call
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.instance_count == 0 {
            return; // macht nichts wenn keine Artikel vorhanden
        }
        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]); //Bind group für shader

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw(0..6, 0..self.instance_count); //mind. 6 vertices
    }
    // passt die größe der spawnenden Partikel an die geänderten Parameter an
    pub fn update_particle_size(&self, queue: &Queue, size: f32) {
        let uniforms = GlobalUniforms {
            screen_size_wrapper: [self.size.0 as f32, self.size.1 as f32, 0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            particle_size_wrapper: [size, 0.0, 0.0, 0.0],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
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
