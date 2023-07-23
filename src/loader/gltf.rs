use std::path::PathBuf;

use gltf::{camera::Projection, Gltf};

use crate::scene;

pub fn load_scene(path: PathBuf) -> Result<scene::Scene, String> {
    let (gltf, buffers, _) =
        ::gltf::import(path).map_err(|e| format!("Error while parsing GLTF file: {}", e))?;

    let nodes: Vec<(gltf::Node, glam::Mat4)> = gltf
        .nodes()
        .flat_map(|n| flatten_transforms(n, glam::Mat4::IDENTITY))
        .collect();

    let objects = nodes.iter().filter_map(|node| {
        let mesh = node.0.mesh()?;
        let vertices = mesh
            .primitives()
            .flat_map(|p| {
                let reader = p.reader(|b| Some(&buffers[b.index()]));
                let positions = reader.read_positions().unwrap();
                positions.map(|p| {
                    let p = glam::f32::Vec3::from(p);
                    node.1.transform_point3(p)
                })
            })
            .collect();

        let indices = mesh
            .primitives()
            .flat_map(|p| {
                let reader = p.reader(|b| Some(&buffers[b.index()]));
                let indices: Vec<u32> = reader.read_indices().unwrap().into_u32().collect();

                indices[..]
                    .windows(3)
                    .step_by(3)
                    .map(|w| glam::u32::UVec3 {
                        x: w[0],
                        y: w[1],
                        z: w[2],
                    })
                    .collect::<Vec<glam::u32::UVec3>>()
            })
            .collect();

        let obj = scene::Object {
            transform: node.1.into(),
            surface: scene::Surface::MeshData {
                vertices: vertices,
                indices,
            },
            material: scene::Material {
                color: glam::vec4(1.0, 1.0, 1.0, 1.0),
                luminosity: 0.0,
                smoothness: 0.0,
            },
        };

        Some(obj)
    });

    let camera = {
        let cam: Option<scene::Camera> =
            nodes
                .iter()
                .find_map(|node| match node.0.camera()?.projection() {
                    Projection::Perspective(perspective) => Some(scene::Camera {
                        transform: node.1.into(),
                        lens: scene::Lens::Perspective {
                            fov: perspective.yfov().to_degrees(),
                            focal_distance: 0.0,
                        },
                        clipping: scene::Clipping {
                            near: perspective.znear(),
                            far: perspective.zfar().unwrap_or(2000.0),
                        },
                    }),
                    _ => None,
                });

        match cam {
            Some(cam) => cam,
            None => return Err("No camera found in scene".to_string()),
        }
    };

    let scene = scene::Scene {
        camera,
        objects: objects.collect(),
    };

    Ok(scene)
}

fn flatten_transforms(node: gltf::Node, transform: glam::Mat4) -> Vec<(gltf::Node, glam::Mat4)> {
    let transform = glam::f32::Mat4::from_cols_array_2d(&node.transform().matrix()) * transform;
    let mut nodes = vec![(node.clone(), transform)];
    for child in node.children().into_iter() {
        nodes.extend(flatten_transforms(child, transform));
    }

    nodes
}

impl Into<scene::Transform> for glam::Mat4 {
    fn into(self) -> scene::Transform {
        let (scale, rotation, position) = self.to_scale_rotation_translation();
        let rotation = rotation.to_euler(glam::EulerRot::XYZ);
        let rotation = glam::f32::Vec3::new(rotation.0, rotation.1, rotation.2);

        scene::Transform {
            position,
            rotation,
            scale,
        }
    }
}
