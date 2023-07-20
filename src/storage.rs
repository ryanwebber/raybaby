use encase::ShaderType;
use glam::f32;

#[derive(ShaderType)]
pub struct Globals {
    pub camera_view_projection: f32::Mat4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: f32::Vec3,
    pub uvs: f32::Vec2,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
