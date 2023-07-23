mod app;
mod cli;
mod loader;
mod pipeline;
mod scene;
mod storage;
mod traits;

use std::{fs, time::Instant};

use app::State;
use clap::Parser;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

async fn run() -> Result<(), String> {
    let args = cli::Cli::parse();
    match args.command {
        cli::Commands::Convert {
            scene,
            scene_format,
        } => {
            let scene = match scene_format {
                cli::SceneFormat::Gltf => loader::gltf::load(scene)
                    .map_err(|e| format!("Unable to parse scene file:\n  {}", e))?,
            };

            let scene_str = {
                let config = ron::ser::PrettyConfig::default().struct_names(true);
                ron::ser::to_string_pretty(&scene, config)
                    .map_err(|e| format!("Unable to serialize scene:\n  {}", e))?
            };

            println!("{}", &scene_str);

            Ok(())
        }
        cli::Commands::Render {
            scene,
            skybox_color,
            ambient_lighting_color,
            ambient_lighting_strength,
            max_ray_bounces_per_ray,
            max_samples_per_pixel,
            focal_blur_strength,
        } => {
            let scene = {
                let scene = fs::read_to_string(scene.as_path())
                    .map_err(|_| format!("Unable to read file: {}", scene.as_path().display()))?;

                ron::from_str::<scene::Scene>(&scene)
                    .map_err(|e| format!("Unable to parse scene file:\n  {}", e))?
            };

            let parameters = app::Parameters {
                frame: 0,
                random_seed: rand::random(),
                max_ray_bounces: max_ray_bounces_per_ray,
                max_samples_per_pixel,
                skybox_color: skybox_color.into(),
                ambient_lighting_color: ambient_lighting_color.into(),
                ambient_lighting_strength,
                focal_blur_strength,
            };

            let event_loop = EventLoop::new();
            let window = WindowBuilder::new()
                .with_title("Raybaby")
                .with_inner_size(LogicalSize {
                    width: 960,
                    height: 540,
                })
                .build(&event_loop)
                .map_err(|e| e.to_string())?;

            let mut state = State::new(&window, &scene, &parameters).await;

            let mut last_frame_inst = Instant::now();
            let (mut frame_count, mut accum_time) = (0, 0.0);

            event_loop.run(move |event, _, control_flow| match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    if !state.input(event) {
                        match event {
                            WindowEvent::CloseRequested
                            | WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode: Some(VirtualKeyCode::Escape),
                                        ..
                                    },
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            WindowEvent::Resized(physical_size) => {
                                state.resize(*physical_size);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                state.resize(**new_inner_size);
                            }
                            _ => {}
                        }
                    }
                }
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    {
                        accum_time += last_frame_inst.elapsed().as_secs_f32();
                        last_frame_inst = Instant::now();
                        frame_count += 1;
                        if frame_count == 100 {
                            println!(
                                "Avg frame time {}ms",
                                accum_time * 1000.0 / frame_count as f32
                            );
                            accum_time = 0.0;
                            frame_count = 0;
                        }
                    }

                    state.update();
                    match state.render(&window) {
                        Ok(_) => {}
                        // Reconfigure the surface if lost
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size()),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                _ => {}
            });
        }
    }
}

fn main() {
    env_logger::init();
    match pollster::block_on(run()) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
