use wgpu::util::DeviceExt;
use crate::{model, vertex};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RotationUniform {
    view_proj: [[f32; 4]; 4],
}

impl RotationUniform {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_angle(&mut self, angle: cgmath::Rad<f32>) {
        self.view_proj = cgmath::Matrix4::from_angle_z(angle).into();
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GradientUniform {
    color: [f32; 4],
}

pub struct GradientSource {
    pub uniform: GradientUniform,
    gradient: colorgrad::Gradient,
    index: f64,
    pub layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub gradient_buffer: wgpu::Buffer,
}

impl GradientSource {
    #[allow(clippy::new_without_default)]
    pub fn new(device: &wgpu::Device) -> Self {

        let uniform = vertex::GradientUniform::default();
        let gradient_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gradient Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("gradient_bind_group_layout"),
            });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: gradient_buffer.as_entire_binding(),
            }],
            label: Some("gradient_bind_group"),
        });

        Self {
            uniform,
            gradient: colorgrad::rainbow(),
            index: 0.0,
            bind_group,
            layout,
            gradient_buffer,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        let color = self.gradient.at(self.index);
        self.uniform.color = [
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        ];
        self.index = (self.index + 0.01) % 1.0;
        queue.write_buffer(
            &self.gradient_buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }
}

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation))
            .into(),
            normal: cgmath::Matrix3::from(self.rotation).into(),
        }
    }
}

impl model::Vertex for InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later
                    // tutorials we'll be using 2, 3, and 4, for Vertex. We'll start at slot 5
                    // not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s.
                // We need to define a slot for each vec4. We'll have to reassemble the mat4
                // in the shader.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
