use std::{fs::File, io::Write};

use cgmath::prelude::*;
use wgpu::util::DeviceExt;

use crate::{
    buffer, camera,
    data::{INDICES, NUM_INSTANCES_PER_ROW, VERTICES},
    depth::{self, RenderPass},
    light,
    model::{self, DrawLight, DrawModel, Vertex},
    render, resources, texture,
    vertex::{self, Instance, InstanceRaw},
};

#[derive(Copy, Clone, Debug, Default)]
pub struct KeyState {
    pub show_depth: bool,
    pub alt_shape: bool,
    pub alt_image: bool,
    pub tex_loop: bool,
    pub screenshot: bool,
    pub rotate: bool,
    pub tab: bool,
    pub tab_index: usize,
    pub background: bool,
}

#[rustfmt::skip]
pub struct State {
    pub clear_color: wgpu::Color,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub config: wgpu::SurfaceConfiguration,

    pub render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group: texture::TextureBindGroup,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,

    pub camera_bundle: camera::CameraBundle,
    rotation_bundle: vertex::RotationBundle,

    depth_pass: depth::DepthPass,
    obj_model: model::Model,
    pub keys: KeyState,

    light_bundle: light::LightBundle,
    light_render_pipeline: wgpu::RenderPipeline,
}

impl State {
    pub async fn new(window: &winit::window::Window) -> anyhow::Result<Self> {
        let clear_color = wgpu::Color::default();
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    //limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let texture_bind_group = texture::TextureBindGroup::from_files(
            &device,
            &queue,
            vec!["gari.png", "tree.png", "baba.png", "moon.png"],
        )
        .await?;

        let light_bundle = light::LightBundle::new(&device, [2.0, 2.0, 2.0], [1.0, 1.0, 1.0]);
        let camera_bundle = camera::CameraBundle::new(&device, &config);
        let rotation_bundle = vertex::RotationBundle::new(&device);

        // Create instances.
        const SPACE_BETWEEN: f32 = 3.0;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let position = cgmath::Vector3 { x, y: 0.0, z };

                    let rotation = if position.is_zero() {
                        // this is needed so an object at (0, 0, 0) won't get scaled to zero
                        // as Quaternions can effect scale if they're not created correctly
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };
                    /*let rotation = cgmath::Quaternion::from_axis_angle(
                        (0.0, 1.0, 0.0).into(),
                        cgmath::Deg(180.0),
                    );*/

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let depth_pass = depth::DepthPass::new(&device, &config);

        let render_pipeline = {
            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &texture_bind_group.layout,
                        &camera_bundle.layout,
                        &rotation_bundle.layout,
                        &light_bundle.layout,
                    ],
                    push_constant_ranges: &[],
                });

            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("normal shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            };
            // ^-OR: let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

            render::create_render_pipeline(
                &device,
                &render_pipeline_layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), InstanceRaw::desc()],
                shader,
                Some("Main Render Pipeline"),
            )
        };

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group.layout,
                    &camera_bundle.layout,
                    &rotation_bundle.layout,
                    &light_bundle.layout,
                ],

                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
            };

            render::create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader,
                Some("Light Render Pipeline"),
            )
        };

        // Buffers.
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = INDICES.len() as u32;

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // View
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    // Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: None,
            });

        let obj_model =
            resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout)
                .await
                .unwrap();

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            clear_color,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            texture_bind_group,
            camera_bundle,
            rotation_bundle,
            instances,
            instance_buffer,
            depth_pass,
            obj_model,
            keys: KeyState::default(),
            light_bundle,
            light_render_pipeline,
        })
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_pass.texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_bind_group(1, &self.camera_bundle.bind_group, &[]);
            render_pass.set_bind_group(2, &self.rotation_bundle.bind_group, &[]);
            render_pass.set_bind_group(3, &self.light_bundle.bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            let labels = ["tree.png", "gari.png", "baba.png", "moon.png", "stone"];
            if self.keys.tab {
                self.keys.tab = false;
                self.keys.tab_index = (self.keys.tab_index + 1) % labels.len();
            }
            let mesh = &self.obj_model.meshes[0];
            let material = &self.obj_model.materials[mesh.material];
            if labels[self.keys.tab_index] == "stone" {
                render_pass.set_bind_group(0, &material.bind_group, &[]);
            } else {
                render_pass.set_bind_group(
                    0,
                    self.texture_bind_group.get(labels[self.keys.tab_index]),
                    &[],
                );
            };

            render_pass.set_pipeline(&self.light_render_pipeline);
            render_pass.draw_light_model(
                &self.obj_model,
                &self.camera_bundle.bind_group,
                &self.light_bundle.bind_group,
            );

            render_pass.set_pipeline(&self.render_pipeline);
            if self.keys.alt_shape {
                render_pass.draw_indexed(9..self.num_indices, 5, 0..self.instances.len() as u32);
            } else {
                render_pass.draw_model_instanced(
                    &self.obj_model,
                    0..self.instances.len() as u32,
                    &self.camera_bundle.bind_group,
                    &self.light_bundle.bind_group,
                );
            }
        }

        // Show depth mask in corner of screen.
        if self.keys.show_depth {
            self.depth_pass.render(&view, &mut encoder);
        }

        // Screenshot.  FIXME: too slow and need to convert colorspace
        if self.keys.screenshot {
            self.keys.screenshot = false;
            self.create_screenshot(encoder, &output);
        } else {
            let command_buffer = encoder.finish();
            self.queue.submit(Some(command_buffer));
        }

        output.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
        self.depth_pass.resize(&self.device, &self.config);
    }

    pub fn update(&mut self) {
        self.camera_bundle.update(&self.queue);
        if self.keys.rotate {
            self.rotation_bundle.update(&self.queue);
        }
        self.depth_pass.update(&self.queue);
        self.light_bundle.update(&self.queue);
    }

    fn create_screenshot(
        &mut self,
        mut encoder: wgpu::CommandEncoder,
        output: &wgpu::SurfaceTexture,
    ) {
        // Create output buffer for image with dimensions of surface.
        let dimensions =
            buffer::BufferDimensions::new(self.size.width as usize, self.size.height as usize);
        let texture_extent = wgpu::Extent3d {
            width: dimensions.width as u32,
            height: dimensions.height as u32,
            depth_or_array_layers: 1,
        };
        let image_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (dimensions.padded_bytes_per_row * dimensions.height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            output.texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &image_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        std::num::NonZeroU32::new(dimensions.padded_bytes_per_row as u32).unwrap(),
                    ),
                    rows_per_image: None,
                },
            },
            texture_extent,
        );

        // Create buffer for to store a new texture for a bind group.
        let new_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("new_texture_label"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        encoder.copy_texture_to_texture(
            output.texture.as_image_copy(),
            wgpu::ImageCopyTexture {
                texture: &new_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            texture_extent,
        );

        let bg_layout = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // View
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    // Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: None,
            });

        let new_texture_view = new_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let new_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let bg_new_texture = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bg_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&new_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&new_sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });
        self.texture_bind_group
            .groups
            .insert("loop".to_string(), bg_new_texture);

        // Submit to gpu command queue.
        let command_buffer = encoder.finish();
        let index = self.queue.submit(Some(command_buffer));

        // Save image to file.  TODO blocks..  not working..
        pollster::block_on(create_png(
            "out.png",
            &self.device,
            image_buffer,
            &dimensions,
            index,
        ));
    }
}

async fn create_png(
    png_output_path: &str,
    device: &wgpu::Device,
    output_buffer: wgpu::Buffer,
    buffer_dimensions: &buffer::BufferDimensions,
    submission_index: wgpu::SubmissionIndex,
) {
    // Note that we're not calling `.await` here.
    let buffer_slice = output_buffer.slice(..);
    // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
    let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    // Poll the device in a blocking manner so that our future resolves.
    // In an actual application, `device.poll(...)` should
    // be called in an event loop or on another thread.
    //
    // We pass our submission index so we don't need to wait for any other possible submissions.
    device.poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));
    // If a file system is available, write the buffer as a PNG
    let has_file_system_available = cfg!(not(target_arch = "wasm32"));
    if !has_file_system_available {
        return;
    }

    if let Some(Ok(())) = receiver.receive().await {
        let padded_buffer = buffer_slice.get_mapped_range();

        let mut png_encoder = png::Encoder::new(
            File::create(png_output_path).unwrap(),
            buffer_dimensions.width as u32,
            buffer_dimensions.height as u32,
        );
        png_encoder.set_depth(png::BitDepth::Eight);
        png_encoder.set_color(png::ColorType::Rgba);
        let mut png_writer = png_encoder
            .write_header()
            .unwrap()
            .into_stream_writer_with_size(buffer_dimensions.unpadded_bytes_per_row)
            .unwrap();

        // from the padded_buffer we write just the unpadded bytes into the image
        for chunk in padded_buffer.chunks(buffer_dimensions.padded_bytes_per_row) {
            png_writer
                .write_all(&chunk[..buffer_dimensions.unpadded_bytes_per_row])
                .unwrap();
        }
        png_writer.finish().unwrap();

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(padded_buffer);

        output_buffer.unmap();
    }
}
