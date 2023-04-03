use super::{
    transform::Transform,
};

use super::super::renderer::Renderer;

use crate::util::cast_slice;
use specs::{Component, VecStorage};

#[derive(Component)]
#[storage(VecStorage)]
pub struct Renderable {
    pub transform_buffer: wgpu::Buffer,
    pub transform_bind_group: wgpu::BindGroup,
}

impl Renderable {
    pub fn new(device: &wgpu::Device, renderer: &Renderer) -> Self {
        let transform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<cg::Matrix4<f32>>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &renderer.transform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding()
                }
            ],
            label: None,
        });

        Self {
            transform_buffer,
            transform_bind_group,
        }
    } 

    pub fn update_transform_buffer(&mut self, queue: &wgpu::Queue, transform: &Transform) {
        queue.write_buffer(&self.transform_buffer, 0, cast_slice(&[transform.get_matrix()]));
    }
}