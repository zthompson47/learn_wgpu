use cgmath::prelude::*;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    pub _padding: u32,
    pub color: [f32; 3],
    pub _padding2: u32,
}

pub struct LightBundle {
    pub uniform: LightUniform,
    pub buffer: wgpu::Buffer,
    pub layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl LightBundle {
    pub fn new(device: &wgpu::Device, position: [f32; 3], color: [f32; 3]) -> Self {
        let uniform = LightUniform {
            position,
            _padding: 0,
            color,
            _padding2: 0,
        };
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: None,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
        });

        LightBundle {
            uniform,
            buffer,
            layout,
            bind_group,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        let old_position: cgmath::Vector3<_> = self.uniform.position.into();
        self.uniform.position =
            (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
                * old_position)
                .into();
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}
