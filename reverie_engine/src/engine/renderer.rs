use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;
use specs::prelude::*;
use wgpu::BindGroupLayout;

use super::{
    components::{
        mesh::Mesh, 
        material::MaterialComponent, transform::TransformComponent,
    },
    context::create_render_pipeline, 
    camera::Camera,
    asset::model::{DrawModel, Vertex, ModelVertex}, scene::Scene,
};

use super::asset::texture::Texture;

pub struct Renderer {
    pub clear_color: wgpu::Color,
    pub texture_view: wgpu::TextureView,
    pub depth_texture: Texture,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub render_pipeline: wgpu::RenderPipeline,
    pub light_pipeline: wgpu::RenderPipeline,
    pub skybox_pipeline: wgpu::RenderPipeline,
}

static MATERIAL_LAYOUT: Lazy<Mutex<Option<Arc<wgpu::BindGroupLayout>>>> = Lazy::new(|| Mutex::new(None));
static TRANSFORM_LAYOUT: Lazy<Mutex<Option<Arc<wgpu::BindGroupLayout>>>> = Lazy::new(|| Mutex::new(None));
static LIGHT_LAYOUT: Lazy<Mutex<Option<Arc<wgpu::BindGroupLayout>>>> = Lazy::new(|| Mutex::new(None));
static SHADOW_LAYOUT: Lazy<Mutex<Option<Arc<wgpu::BindGroupLayout>>>> = Lazy::new(|| Mutex::new(None));
static SKYBOX_LAYOUT: Lazy<Mutex<Option<Arc<wgpu::BindGroupLayout>>>> = Lazy::new(|| Mutex::new(None));

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        extent: &wgpu::Extent3d,
    ) -> Self {
        let clear_color = wgpu::Color::BLACK;

        let (texture_view, depth_texture) = create_depth_texture(device, extent);

        let mut transform_layout = TRANSFORM_LAYOUT.lock().unwrap();
        *transform_layout = Some(Arc::new(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: None,
        })));

        let mut material_layout = MATERIAL_LAYOUT.lock().unwrap();
        *material_layout = Some(Arc::new(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
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
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 10,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 11,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("material_bind_group_layout"),
        })));

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });
        
        let mut light_layout = LIGHT_LAYOUT.lock().unwrap();
        *light_layout = Some(Arc::new(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::CubeArray,
                        sample_type: wgpu::TextureSampleType::Depth,
                    },
                    count: Some(std::num::NonZeroU32::new(16).unwrap()),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("light_bind_group_layout")
        })));

        let mut shadow_layout = SHADOW_LAYOUT.lock().unwrap();
        *shadow_layout = Some(Arc::new(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("shadow_layout"),
        })));

        let mut skybox_layout = SKYBOX_LAYOUT.lock().unwrap();
        *skybox_layout = Some(Arc::new(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("shadow_layout"),
        })));

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                transform_layout.as_ref().unwrap(),
                &camera_bind_group_layout,
                material_layout.as_ref().unwrap(),
                light_layout.as_ref().unwrap(),
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/pbr.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                config.format,
                Some(wgpu::TextureFormat::Depth32Float),
                &[ModelVertex::desc()],
                shader,
            )
        };

        let light_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                shadow_layout.as_ref().unwrap(),
                transform_layout.as_ref().unwrap(),
            ],
            push_constant_ranges: &[],
        });

        let light_pipeline = {
            let shader_desc = wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/shadow.wgsl").into()),
            };
            let shader = device.create_shader_module(shader_desc);
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("light pass"),
                layout: Some(&light_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[ModelVertex::desc()],
                },
                fragment: None,
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Front),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false
                },
                depth_stencil: Some(wgpu::TextureFormat::Depth32Float).map(|format| wgpu::DepthStencilState {
                    format,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState  {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None
            })
        };

        let skybox_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                skybox_layout.as_ref().unwrap(),
            ],
            push_constant_ranges: &[],
        });

        let skybox_pipeline = {
            let shader_desc = wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/skybox.wgsl").into()),
            };
            let shader = device.create_shader_module(shader_desc);
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("skybox pass"),
                layout: Some(&skybox_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x3,
                            }
                        ]
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false
                },
                depth_stencil: Some(wgpu::TextureFormat::Depth32Float).map(|format| wgpu::DepthStencilState {
                    format,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState  {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None
            })
        };

        Self {
            clear_color,
            texture_view,
            depth_texture,
            camera_bind_group_layout,
            render_pipeline,
            light_pipeline,
            skybox_pipeline,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, extent: &wgpu::Extent3d) {
        (self.texture_view, self.depth_texture) = create_depth_texture(device, extent);
    }

    pub fn get_material_layout() -> Arc<BindGroupLayout> {
        MATERIAL_LAYOUT.lock().unwrap().as_ref().unwrap().clone()
    }

    pub fn get_transform_layout() -> Arc<BindGroupLayout> {
        TRANSFORM_LAYOUT.lock().unwrap().as_ref().unwrap().clone()
    }

    pub fn get_light_layout() -> Arc<BindGroupLayout> {
        LIGHT_LAYOUT.lock().unwrap().as_ref().unwrap().clone()
    }

    pub fn get_shadow_layout() -> Arc<BindGroupLayout> {
        SHADOW_LAYOUT.lock().unwrap().as_ref().unwrap().clone()
    }

    pub fn get_skybox_layout() -> Arc<BindGroupLayout> {
        SKYBOX_LAYOUT.lock().unwrap().as_ref().unwrap().clone()
    }
}

pub fn create_depth_texture(device: &wgpu::Device, extent: &wgpu::Extent3d) -> (wgpu::TextureView, Texture){
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("texture"),
        size: *extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_texture = Texture::create_depth_texture(&device, extent, "depth_texture");

    (texture_view, depth_texture)
}

pub trait Pass {
    fn draw(&mut self, view: &wgpu::TextureView, scene: &mut Scene, camera: &Camera, encoder: &mut wgpu::CommandEncoder) -> Result<(), wgpu::SurfaceError>;
}

impl Pass for Renderer {
    fn draw(&mut self, view: &wgpu::TextureView, scene: &mut Scene, camera: &Camera, encoder: &mut wgpu::CommandEncoder) -> Result<(), wgpu::SurfaceError> {
        
        for shadow in &scene.light_manager.point_shadows {
            for (i, depth_texture_view) in shadow.views.iter().enumerate() {
                let meshes = scene.world.read_storage::<Mesh>();
                let transforms = scene.world.read_storage::<TransformComponent>();
                let materials_c = scene.world.read_storage::<MaterialComponent>();

                let light_pass_descriptor = wgpu::RenderPassDescriptor {
                    label: Some("light_pass_desc"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: depth_texture_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                };
                let mut light_pass = encoder.begin_render_pass(&light_pass_descriptor);
                light_pass.set_pipeline(&self.light_pipeline);
                light_pass.set_bind_group(0, &shadow.bind_groups[i], &[]);

                for (transform, mesh, _) in (&transforms, &meshes, &materials_c).join() {
                    light_pass.set_bind_group(1, &transform.bind_group, &[]);
                    for m in (*mesh.mesh).iter() {
                        light_pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                        light_pass.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        light_pass.draw_indexed(0..m.element_count, 0, 0..1);
                    }
                }
            }
        }

        let meshes = scene.world.read_storage::<Mesh>();
        let transforms = scene.world.read_storage::<TransformComponent>();
        let materials_c = scene.world.read_storage::<MaterialComponent>();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(self.clear_color),
                            store: true,
                        }
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            }); 

            render_pass.set_pipeline(&self.skybox_pipeline);
            render_pass.set_bind_group(0, &scene.skybox.as_ref().unwrap().bind_group, &[]);
            
            render_pass.set_vertex_buffer(0, scene.skybox.as_ref().unwrap().vertex_buffer.slice(..));
            render_pass.draw(0..36, 0..1);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(1, &camera.bind_group, &[]);
            render_pass.set_bind_group(3, &scene.light_manager.bind_group, &[]);
            
            for (mesh, transform, material) in (&meshes, &transforms, &materials_c).join()  {
                render_pass.set_bind_group(0, &transform.bind_group, &[]);
                for m in (*mesh.mesh).iter() {
                    render_pass.draw_mesh(&m, &material.material);
                }
            }
        }

        Ok(())
    }
}