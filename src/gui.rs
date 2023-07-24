use egui::RichText;

use crate::{
    app::Timing,
    storage::{Camera, Globals},
};

pub struct Window {
    info: InfoPane,
    camera: CameraPane,
    environment: EnvironmentPane,
}

impl Window {
    pub fn new() -> Self {
        Self {
            info: InfoPane,
            camera: CameraPane,
            environment: EnvironmentPane,
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context, globals: &mut Globals, timing: &Timing) {
        egui::Window::new("Info")
            .default_open(true)
            .show(ctx, |ui: &mut egui::Ui| {
                self.info.ui(ui, globals, timing);
            });

        egui::Window::new("Camera")
            .default_open(true)
            .show(ctx, |ui: &mut egui::Ui| {
                self.camera.ui(ui, &mut globals.camera);
            });

        egui::Window::new("Environment")
            .default_open(true)
            .show(ctx, |ui: &mut egui::Ui| {
                self.environment.ui(ui, globals);
            });
    }
}

struct InfoPane;

impl InfoPane {
    fn ui(&mut self, ui: &mut egui::Ui, globals: &mut Globals, timing: &Timing) {
        draw_section(ui, "Frame", |ui| {
            ui.label("FPS");
            ui.label(RichText::new(format!("{:.2}", timing.avs_fps)).monospace());

            ui.end_row();

            ui.label("Frames");
            ui.label(RichText::new(format!("{}", globals.frame)).monospace());
        });
    }
}

struct CameraPane;

impl CameraPane {
    fn ui(&mut self, ui: &mut egui::Ui, camera: &mut Camera) {
        draw_section(ui, "Transform", |ui| {
            let position = camera.world_space_position;
            let rotation = {
                let (_, rotation, _) = camera.local_to_world_matrix.to_scale_rotation_translation();
                let (x, y, z) = rotation.to_euler(glam::EulerRot::XYZ);
                (x.to_degrees(), y.to_degrees(), z.to_degrees())
            };

            let mut transform = crate::scene::Transform {
                position,
                rotation: glam::vec3(rotation.0, rotation.1, rotation.2),
                scale: glam::vec3(1.0, 1.0, 1.0),
            };

            let transform_original = transform.clone();
            draw_transform(ui, &mut transform);
            if transform != transform_original {
                let (x, y, z) = {
                    let (x, y, z) = transform.rotation.into();
                    (x.to_radians(), y.to_radians(), z.to_radians())
                };

                camera.world_space_position = transform.position;
                camera.local_to_world_matrix = glam::Mat4::from_euler(glam::EulerRot::XYZ, x, y, z);
            }
        });
    }
}

struct EnvironmentPane;

impl EnvironmentPane {
    fn ui(&mut self, ui: &mut egui::Ui, globals: &mut Globals) {
        draw_section(ui, "Skybox", |ui| {
            let mut color = egui::Color32::from_rgba_premultiplied(
                (globals.skybox_color[0] * 255.0) as u8,
                (globals.skybox_color[1] * 255.0) as u8,
                (globals.skybox_color[2] * 255.0) as u8,
                255,
            );

            ui.label("Skybox color");
            ui.color_edit_button_srgba(&mut color);
        });

        draw_section(ui, "Lighting", |ui| {
            let mut color = egui::Color32::from_rgba_premultiplied(
                (globals.ambient_lighting_color[0] * 255.0) as u8,
                (globals.ambient_lighting_color[1] * 255.0) as u8,
                (globals.ambient_lighting_color[2] * 255.0) as u8,
                255,
            );

            ui.label("Ambient lighting color");
            ui.color_edit_button_srgba(&mut color);

            ui.end_row();

            ui.label("Ambient lighting strength");
            ui.add(egui::DragValue::new(&mut globals.ambient_lighting_strength).speed(0.1));
        });
    }
}

fn draw_section<F>(ui: &mut egui::Ui, name: &'static str, builder: F)
where
    F: FnOnce(&mut egui::Ui),
{
    egui::CollapsingHeader::new(name)
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new(name)
                .striped(true)
                .spacing([10.0, 10.0])
                .show(ui, |ui| {
                    builder(ui);
                });
        });
}

fn draw_transform(ui: &mut egui::Ui, transform: &mut crate::scene::Transform) {
    ui.label("Position");
    ui.add(egui::DragValue::new(&mut transform.position[0]).speed(0.1));
    ui.add(egui::DragValue::new(&mut transform.position[1]).speed(0.1));
    ui.add(egui::DragValue::new(&mut transform.position[2]).speed(0.1));

    ui.end_row();

    ui.label("Rotation");
    ui.add(egui::DragValue::new(&mut transform.rotation[0]).speed(0.1));
    ui.add(egui::DragValue::new(&mut transform.rotation[1]).speed(0.1));
    ui.add(egui::DragValue::new(&mut transform.rotation[2]).speed(0.1));

    ui.end_row();

    ui.label("Scale");
    ui.add(egui::DragValue::new(&mut transform.scale[0]).speed(0.1));
    ui.add(egui::DragValue::new(&mut transform.scale[1]).speed(0.1));
    ui.add(egui::DragValue::new(&mut transform.scale[2]).speed(0.1));
}
