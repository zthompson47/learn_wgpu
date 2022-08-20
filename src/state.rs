use std::{fs::File, io::Write, mem::size_of};

use wgpu::util::DeviceExt;

use crate::{texture::TextureBindGroup, Camera, Texture, Vertex, INDICES, VERTICES};

struct BufferDimensions {
    width: usize,
    height: usize,
    #[allow(dead_code)]
    unpadded_bytes_per_row: usize,
    padded_bytes_per_row: usize,
}

impl BufferDimensions {
    fn new(width: usize, height: usize) -> Self {
        let bytes_per_pixel = size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
        Self {
            width,
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }
}

pub struct State {
    pub clear_color: wgpu::Color,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub render_pipeline: wgpu::RenderPipeline,
    pub alt_shape: bool,
    pub alt_image: bool,
    pub tex_loop: bool,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub camera: Camera,
    pub bind_group: TextureBindGroup,
    pub screenshot: bool,
}

impl State {
    pub async fn new(window: &winit::window::Window) -> Self {
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
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();
        let config = wgpu::SurfaceConfiguration {
            //usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            //TODO format: surface.get_supported_formats(&adapter)[0],
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let texture_gari = Texture::from_bytes(
            &device,
            &queue,
            include_bytes!("../img/gari.png"),
            "gari.png",
        )
        .unwrap();

        let texture_tree = Texture::from_bytes(
            &device,
            &queue,
            include_bytes!("../img/tree.png"),
            "tree.png",
        )
        .unwrap();

        let mut bind_group = TextureBindGroup::new(&device, Some("bind_group"));
        bind_group.add(&device, texture_gari, "gari");
        bind_group.add(&device, texture_tree, "tree");

        // Load the shader.
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        // OR:
        // let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        // Pipeline.
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group.layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires
                // Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

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

        // Camera.
        let camera = Camera::new(config.width as f32, config.height as f32);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            clear_color,
            render_pipeline,
            alt_shape: false,
            alt_image: false,
            tex_loop: false,
            vertex_buffer,
            index_buffer,
            num_indices,
            //texture_gari,
            //bg_gari,
            //bg_tree,
            camera,
            //texture_bind_group_layout,
            bind_group,
            screenshot: false,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn update(&mut self) {}

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
                depth_stencil_attachment: None,
            });
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            if self.tex_loop {
                render_pass.set_bind_group(0, self.bind_group.get("loop"), &[]);
            } else if self.alt_image {
                render_pass.set_bind_group(0, self.bind_group.get("tree"), &[]);
            } else {
                render_pass.set_bind_group(0, self.bind_group.get("gari"), &[]);
            }

            render_pass.set_pipeline(&self.render_pipeline);

            if self.alt_shape {
                render_pass.draw_indexed(9..self.num_indices, 5, 0..1);
            } else {
                render_pass.draw_indexed(0..9, 0, 0..1);
            }
        }

        if self.screenshot {
            self.screenshot = false;

            // Create output buffer for image with dimensions of surface.
            let dimensions =
                BufferDimensions::new(self.size.width as usize, self.size.height as usize);
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
                            std::num::NonZeroU32::new(dimensions.padded_bytes_per_row as u32)
                                .unwrap(),
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

            let bg_layout =
                self.device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[
                            // View
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    multisampled: false,
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    sample_type: wgpu::TextureSampleType::Float {
                                        filterable: true,
                                    },
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
            self.bind_group.groups.insert("loop".to_string(), bg_new_texture);

            // Submit to gpu command queue.
            let command_buffer = encoder.finish();
            let index = self.queue.submit(Some(command_buffer));

            // Save image to file.
            pollster::block_on(create_png(
                "out.png",
                &self.device,
                image_buffer,
                &dimensions,
                index,
            ));
        } else {
            let command_buffer = encoder.finish();
            self.queue.submit(Some(command_buffer));
        }

        output.present();

        Ok(())
    }
}

async fn create_png(
    png_output_path: &str,
    device: &wgpu::Device,
    output_buffer: wgpu::Buffer,
    buffer_dimensions: &BufferDimensions,
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
