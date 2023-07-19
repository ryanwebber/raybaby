use encase::ShaderType;
use glam::f32::Mat4;

#[derive(ShaderType)]
pub struct Globals {
    pub camera_view_projection: Mat4,
}
