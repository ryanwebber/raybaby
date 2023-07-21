use encase::{
    private::WriteInto, ArrayLength, ShaderSize, ShaderType, StorageBuffer, UniformBuffer,
};
use glam::f32;

pub trait Storable {
    fn into_bytes(&self) -> Vec<u8>;
}

pub struct Uniform<'a, T>(pub &'a T)
where
    T: ShaderType + ShaderSize + WriteInto;

impl<T> Storable for Uniform<'_, T>
where
    T: ShaderType + ShaderSize + WriteInto,
{
    fn into_bytes(&self) -> Vec<u8> {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(self.0).expect("Unable to write uniform");
        buffer.into_inner()
    }
}

pub struct Buffer<'a, T>(pub &'a [T])
where
    T: ShaderSize;

#[derive(ShaderType)]
struct SizedBuffer<'a, T: ShaderSize + 'a> {
    length: ArrayLength,

    #[size(runtime)]
    buffer: &'a [T],
}

impl<'a, T> SizedBuffer<'a, T>
where
    T: ShaderSize + 'a,
{
    fn new(buffer: &'a [T]) -> Self {
        Self {
            length: ArrayLength,
            buffer,
        }
    }
}

impl<T> Storable for Buffer<'_, T>
where
    T: ShaderSize + WriteInto,
{
    fn into_bytes(&self) -> Vec<u8> {
        let data = SizedBuffer::new(self.0);
        let mut buffer = StorageBuffer::new(Vec::new());
        buffer.write(&data).expect("Unable to write buffer");

        buffer.into_inner()
    }
}

/*
    Types
*/

#[derive(ShaderType)]
pub struct Globals {
    pub camera: Camera,
    pub frame: u32,
    pub random_seed: u32,
    pub max_ray_bounces: u32,
    pub max_samples_per_pixel: u32,
}

#[derive(ShaderType)]
pub struct Camera {
    pub focal_plane: f32::Vec3,
    pub world_space_position: f32::Vec3,
    pub local_to_world_matrix: f32::Mat4,
    pub near_clip: f32,
    pub far_clip: f32,
}

#[derive(ShaderType)]
pub struct Material {
    pub color: f32::Vec4,
    pub luminosity: f32,
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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_buffer_serialize() {
        let spheres: &[Sphere] = &[
            Sphere {
                position: f32::vec3(0.0, 1.0, 2.0),
                radius: 23.0,
                material_id: 0x55,
            },
            Sphere {
                position: f32::vec3(3.0, 4.0, 5.0),
                radius: 24.0,
                material_id: 0x77,
            },
        ];

        let bytes = Buffer(spheres).into_bytes();
        assert_eq!(bytes[0], 2);
        assert_eq!(bytes.len() % 16, 0);
    }
}
