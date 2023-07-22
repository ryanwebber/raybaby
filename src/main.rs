mod app;
mod cli;
mod pipeline;
mod storage;
mod traits;
mod types;

use std::{fs, time::Instant};

use app::State;
use clap::Parser;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

async fn run() {
    let args = cli::Cli::parse();
    match args.command {
        cli::Commands::Render {
            scene,
            skybox_color,
            ambient_lighting_color,
            ambient_lighting_strength,
            max_ray_bounces_per_ray,
            max_samples_per_pixel,
        } => {
            let scene = {
                let scene = fs::read_to_string(scene.as_path()).expect(&format!(
                    "Unable to read file: {}",
                    scene.as_path().display()
                ));

                ron::from_str::<types::Scene>(&scene).expect("Unable to parse scene")
            };

            let parameters = app::Parameters {
                frame: 0,
                random_seed: rand::random(),
                max_ray_bounces: max_ray_bounces_per_ray,
                max_samples_per_pixel,
                skybox_color: skybox_color.into(),
                ambient_lighting_color: ambient_lighting_color.into(),
                ambient_lighting_strength,
            };

            let event_loop = EventLoop::new();
            let window = WindowBuilder::new()
                .with_title("Raybaby")
                .with_inner_size(LogicalSize {
                    width: 960,
                    height: 540,
                })
                .build(&event_loop)
                .unwrap();

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
                    match state.render() {
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
    pollster::block_on(run());
}
