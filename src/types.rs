use glam::f32;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Object>,
}

#[derive(Debug, Deserialize)]
pub struct Camera {
    pub transform: Transform,
    pub lens: Lens,
    pub clipping: Clipping,
}

#[derive(Debug, Deserialize)]
pub enum Lens {
    Perspective { fov: f32 },
}

#[derive(Debug, Deserialize)]
pub struct Clipping {
    pub near: f32,
    pub far: f32,
}

#[derive(Debug, Deserialize)]
pub struct Object {
    pub surface: Surface,
    pub transform: Transform,
    pub material: Material,
}

#[derive(Debug, Deserialize)]
pub enum Surface {
    Sphere { radius: f32 },
}

#[derive(Debug, Deserialize)]
pub struct Transform {
    pub position: f32::Vec3,
    pub rotation: f32::Vec3,
}

#[derive(Debug, Deserialize)]
pub struct Material {
    pub color: f32::Vec4,
    pub luminosity: f32,
    pub smoothness: f32,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize() {
        let source = include_str!("../examples/01-spheres.ron");
        let scene = ron::from_str::<Scene>(source).expect("Unable to parse scene");
        assert_eq!(
            scene.objects[0].material.color,
            f32::vec4(1.0, 0.0, 0.0, 1.0)
        );
    }
}
