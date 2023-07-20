use wgpu::util::DeviceExt;
use winit::{event::WindowEvent, window::Window};

use crate::{
    pipeline::{compute::ComputePipeline, render::RenderPipeline},
    storage::{self, Storable},
};

const QUAD_VERTICIES: &[storage::Vertex] = &[
    storage::Vertex {
        position: glam::f32::vec3(-1.0, 1.0, 0.0),
        uvs: glam::f32::vec2(0.0, 1.0),
    },
    storage::Vertex {
        position: glam::f32::vec3(-1.0, -1.0, 0.0),
        uvs: glam::f32::vec2(0.0, 0.0),
    },
    storage::Vertex {
        position: glam::f32::vec3(1.0, -1.0, 0.0),
        uvs: glam::f32::vec2(1.0, 0.0),
    },
    storage::Vertex {
        position: glam::f32::vec3(1.0, 1.0, 0.0),
        uvs: glam::f32::vec2(1.0, 1.0),
    },
];

const QUAD_INDICES: &[u32] = &[0, 1, 2, 2, 3, 0];

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    pipelines: Pipelines,
    pipeline_data: PipelineData,
}

pub struct Pipelines {
    compute: ComputePipeline,
    render: RenderPipeline,
}

pub struct PipelineData {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    globals_buffer: wgpu::Buffer,
    materials_buffer: wgpu::Buffer,
    spheres_buffer: wgpu::Buffer,
    render_texture: wgpu::TextureView,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

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
                    features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        // Shader code assumes an sRGB surface texture
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let pipelines = Pipelines {
            compute: ComputePipeline::new(&device),
            render: RenderPipeline::new(&device, surface_format),
        };

        let pipeline_data = PipelineData {
            vertex_buffer: {
                let bytes = bytemuck::cast_slice(QUAD_VERTICIES);
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: &bytes,
                    usage: wgpu::BufferUsages::VERTEX,
                })
            },
            index_buffer: {
                let bytes = bytemuck::cast_slice(QUAD_INDICES);
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: &bytes,
                    usage: wgpu::BufferUsages::INDEX,
                })
            },
            globals_buffer: {
                let globals_uniform = {
                    let fov: f32 = 0.87;
                    let focal_distance: f32 = 10.0;
                    let aspect_ratio = (size.width as f32) / (size.height as f32);
                    let plane_height = 2.0 * (fov / 2.0).tan() * focal_distance;
                    let plane_width = plane_height * aspect_ratio;
                    storage::Globals {
                        camera: storage::Camera {
                            focal_plane: glam::f32::vec3(plane_width, plane_height, focal_distance),
                            world_space_position: glam::f32::vec3(0.0, 0.0, 0.0),
                            local_to_world_matrix: glam::f32::Mat4::from_euler(
                                glam::EulerRot::XYZ,
                                0.0,
                                0.0,
                                0.0,
                            ),
                        },
                    }
                };

                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Globals buffer"),
                    contents: &storage::Uniform(globals_uniform).into_bytes(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                })
            },
            materials_buffer: {
                let materials: &[storage::Material] = &[
                    storage::Material {
                        color: glam::f32::vec4(1.0, 0.0, 0.0, 1.0),
                    },
                    storage::Material {
                        color: glam::f32::vec4(0.0, 1.0, 0.0, 1.0),
                    },
                    storage::Material {
                        color: glam::f32::vec4(0.0, 0.0, 1.0, 1.0),
                    },
                ];

                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Materials buffer"),
                    contents: &storage::Buffer(&materials).into_bytes(),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
                })
            },
            spheres_buffer: {
                let spheres: &[storage::Sphere] = &[storage::Sphere {
                    position: glam::f32::vec3(0.0, 0.0, 10.0),
                    radius: 5.0,
                    material_id: 0,
                }];

                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Spheres buffer"),
                    contents: &storage::Buffer(&spheres).into_bytes(),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
                })
            },
            render_texture: {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Output texture"),
                    size: wgpu::Extent3d {
                        width: size.width,
                        height: size.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba32Float,
                    usage: wgpu::TextureUsages::STORAGE_BINDING,
                    view_formats: &[wgpu::TextureFormat::Rgba32Float],
                });

                texture.create_view(&wgpu::TextureViewDescriptor::default())
            },
        };

        Self {
            surface,
            device,
            queue,
            config,
            size,
            pipelines,
            pipeline_data,
        }
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0
            && new_size.height > 0
            && new_size.width < u32::MAX
            && new_size.height < u32::MAX
        {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, _: &WindowEvent) -> bool {
        false
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

        // Compute pass
        {
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Compute bind group"),
                layout: &self.pipelines.compute.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.pipeline_data.globals_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &self.pipeline_data.render_texture,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.pipeline_data.materials_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.pipeline_data.spheres_buffer.as_entire_binding(),
                    },
                ],
            });

            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Compute pass"),
                });

                compute_pass.set_pipeline(&self.pipelines.compute.pipeline);
                compute_pass.set_bind_group(0, &bind_group, &[]);
                compute_pass.dispatch_workgroups(self.size.width, self.size.height, 1)
            }
        }

        // Render pass
        {
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Render bind group"),
                layout: &self.pipelines.render.bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &self.pipeline_data.render_texture,
                    ),
                }],
            });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[
                        // This is what @location(0) in the fragment shader targets
                        Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 1.0,
                                    g: 0.0,
                                    b: 0.5,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        }),
                    ],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&self.pipelines.render.pipeline);
                render_pass.set_bind_group(0, &bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.pipeline_data.vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    self.pipeline_data.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );

                render_pass.draw_indexed(0..6, 0, 0..1);
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
