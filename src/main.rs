#![allow(unused_imports, dead_code)]

use engine::engine::Engine;
use winit::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, MouseScrollDelta};

mod engine;

fn main() {
    println!("hello world");

    
    let (mut engine, event_loop) = Engine::new();

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    control_flow.set_exit();
                    println!("goodbye world");
                }

                WindowEvent::Resized(_) => {
                    engine.resize();
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    handle_input(&mut engine, input);
                }

                WindowEvent::MouseWheel { delta, ..} => {
                    let delta = match delta {
                        MouseScrollDelta::LineDelta(_x, y) => y,
                        MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    };

                    engine.zoom(delta);
                }

                _ => ()
            }
            Event::MainEventsCleared => {

            }
            Event::RedrawEventsCleared => {
                engine.render();
            }

            _ => (),
        }
    });

    fn handle_input(engine: &mut Engine, input: KeyboardInput) {
        let keycode = input.virtual_keycode;

        match keycode {
            Some(code) => {
                match code {
                    VirtualKeyCode::W => {
                        engine.camera_up();
                    }
                    VirtualKeyCode::A => {
                        engine.camera_left();
                    }
                    VirtualKeyCode::S => {
                        engine.camera_down();
                    }
                    VirtualKeyCode::D => {
                        engine.camera_right();
                    }

                    VirtualKeyCode::E => {
                        engine.resolution_up();
                    }
                    
                    VirtualKeyCode::Q => {
                        engine.resolution_down();
                    }

                    VirtualKeyCode::R => {
                        engine.zoom(0.5);
                    }

                    VirtualKeyCode::F => {
                        engine.zoom(-0.5);
                    }

                    VirtualKeyCode::T => {
                        engine.reset_camera();
                    }


                    _ => ()
                }

                // engine.render();
            }

            None => {
                return;
            }
        }
    }
}