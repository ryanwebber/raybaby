use encase::{private::WriteInto, DynamicStorageBuffer, ShaderSize, ShaderType, UniformBuffer};
use glam::f32;

pub trait Storable {
    fn into_bytes(&self) -> Vec<u8>;
}

pub struct Uniform<T>(pub T)
where
    T: ShaderType + ShaderSize + WriteInto;

impl<T> Storable for Uniform<T>
where
    T: ShaderType + ShaderSize + WriteInto,
{
    fn into_bytes(&self) -> Vec<u8> {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(&self.0).expect("Unable to write uniform");
        buffer.into_inner()
    }
}

pub struct Buffer<'a, T>(pub &'a [T])
where
    T: ShaderType + ShaderSize + WriteInto;

impl<T> Storable for Buffer<'_, T>
where
    T: ShaderType + ShaderSize + WriteInto,
{
    fn into_bytes(&self) -> Vec<u8> {
        let len = self.0.len() as u32;
        let mut buffer = DynamicStorageBuffer::new_with_alignment(Vec::new(), 32);
        buffer.write(&len).unwrap();
        buffer
            .write(&self.0)
            .expect("Unable to write object buffer");

        buffer.into_inner()
    }
}

/*
    Types
*/

#[derive(ShaderType)]
pub struct Globals {
    pub camera: Camera,
}

#[derive(ShaderType)]
pub struct Camera {
    pub focal_plane: f32::Vec3,
    pub world_space_position: f32::Vec3,
    pub local_to_world_matrix: f32::Mat4,
}

#[derive(ShaderType)]
pub struct Material {
    pub color: f32::Vec4,
}

#[derive(ShaderType)]
pub struct Sphere {
    pub position: f32::Vec3,
    pub radius: f32,
    pub material_id: u32,
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
