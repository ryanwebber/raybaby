use crate::storage::{Camera, Globals};

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

    pub fn ui(&mut self, ctx: &egui::Context, globals: &mut Globals) {
        egui::Window::new("Info")
            .min_width(900.0)
            .show(ctx, |ui: &mut egui::Ui| {
                self.info.ui(ui, globals);
            });

        egui::Window::new("Camera").show(ctx, |ui: &mut egui::Ui| {
            self.camera.ui(ui, &mut globals.camera);
        });

        egui::Window::new("Environment").show(ctx, |ui: &mut egui::Ui| {
            self.environment.ui(ui, globals);
        });
    }
}

struct InfoPane;

impl InfoPane {
    fn ui(&mut self, ui: &mut egui::Ui, globals: &mut Globals) {
        egui::Grid::new("info")
            .striped(true)
            .spacing([10.0, 10.0])
            .show(ui, |ui| {
                ui.label("Frame");
                ui.add(egui::DragValue::new(&mut globals.frame).speed(1.0));
            });
    }
}

struct CameraPane;

impl CameraPane {
    fn ui(&mut self, ui: &mut egui::Ui, camera: &mut Camera) {
        // Camera transform
        {
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
                camera.local_to_world_matrix = glam::Mat4::from_scale_rotation_translation(
                    transform.scale,
                    glam::Quat::from_euler(glam::EulerRot::XYZ, x, y, z),
                    transform.position,
                );
            }
        }
    }
}

struct EnvironmentPane;

impl EnvironmentPane {
    fn ui(&mut self, ui: &mut egui::Ui, globals: &mut Globals) {
        egui::CollapsingHeader::new("Skybox")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("skybox")
                    .striped(true)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        let mut color = egui::Color32::from_rgba_premultiplied(
                            (globals.skybox_color[0] * 255.0) as u8,
                            (globals.skybox_color[1] * 255.0) as u8,
                            (globals.skybox_color[2] * 255.0) as u8,
                            255,
                        );

                        ui.label("Skybox color");
                        ui.color_edit_button_srgba(&mut color);
                    })
            });

        egui::CollapsingHeader::new("Lighting")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("lighting")
                    .striped(true)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
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
                        ui.add(
                            egui::DragValue::new(&mut globals.ambient_lighting_strength).speed(0.1),
                        );
                    })
            });
    }
}

fn draw_transform(ui: &mut egui::Ui, transform: &mut crate::scene::Transform) {
    egui::CollapsingHeader::new("Transform")
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new("transform")
                .striped(true)
                .spacing([10.0, 10.0])
                .show(ui, |ui| {
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
                });
        });
}
