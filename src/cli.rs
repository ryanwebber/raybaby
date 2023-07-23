use clap::{arg, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// A simple raytracing renderer
#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SceneFormat {
    Ron,
    Gltf,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Render {
        /// The scene to render
        #[arg(long, required = true)]
        scene: PathBuf,

        /// The type of scene file
        #[arg(long, required = true)]
        scene_format: SceneFormat,

        /// The skybox color
        #[arg(long, default_value_t = Color(0.0, 0.0, 0.0))]
        skybox_color: Color,

        /// The ambient lighting color
        #[arg(long, default_value_t = Color(1.0, 1.0, 1.0))]
        ambient_lighting_color: Color,

        /// The ambient lighting strength
        #[arg(long, default_value_t = 0.1)]
        ambient_lighting_strength: f32,

        /// The maximum number of ray bounces per ray
        #[arg(long, default_value_t = 30)]
        max_ray_bounces_per_ray: u32,

        /// The maximum number of rays per pixel per render pass
        #[arg(long, default_value_t = 4)]
        max_samples_per_pixel: u32,

        /// The strength of blur for objects out of focus
        #[arg(long, default_value_t = 200.0)]
        focal_blur_strength: f32,
    },
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Color(pub f32, pub f32, pub f32);

impl std::str::FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ron::from_str::<Color>(s).map_err(|e| format!("Invalid color (at: {})", e.position.col))
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (r, g, b) = (self.0, self.1, self.2);
        write!(f, "({:.2}, {:.2}, {:.2})", r, g, b)
    }
}

impl Into<glam::Vec3> for Color {
    fn into(self) -> glam::Vec3 {
        glam::vec3(self.0, self.1, self.2)
    }
}
