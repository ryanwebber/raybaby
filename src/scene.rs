use glam::{f32, u32};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Object>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Camera {
    pub transform: Transform,
    pub lens: Lens,
    pub clipping: Clipping,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Lens {
    Perspective { fov: f32, focal_distance: f32 },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Clipping {
    pub near: f32,
    pub far: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Object {
    pub surface: Surface,
    pub transform: Transform,
    pub material: Material,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Surface {
    Sphere {
        radius: f32,
    },
    MeshData {
        vertices: Vec<f32::Vec3>,
        indices: Vec<u32::UVec3>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Transform {
    pub position: f32::Vec3,
    pub rotation: f32::Vec3,
    pub scale: f32::Vec3,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Material {
    pub color: f32::Vec4,
    pub luminosity: f32,
    pub smoothness: f32,
}

impl Into<Transform> for glam::Mat4 {
    fn into(self) -> Transform {
        let (scale, rotation, position) = self.to_scale_rotation_translation();
        let rotation = rotation.to_euler(glam::EulerRot::XYZ);
        let rotation = glam::f32::Vec3::new(rotation.0, rotation.1, rotation.2);

        Transform {
            position,
            rotation,
            scale,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize() {
        let source = include_str!("../examples/01-spheres.ron");
        let scene = ron::from_str::<Scene>(source).expect("Unable to parse scene");
        assert!(scene.objects.len() > 0);
    }
}
