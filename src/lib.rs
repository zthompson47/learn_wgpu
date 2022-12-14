mod buffer;
mod camera;
mod data;
mod depth;
mod light;
mod model;
mod render;
mod resources;
mod state;
mod texture;
mod vertex;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use state::State;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    //let window = winit::window::Window::new(&event_loop).unwrap();
    window.set_maximized(true);

    /*println!(
        "__++__ {:?} is_max:{}",
        window.inner_size(),
        window.is_maximized()
    );*/

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("learn_wgpu")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = State::new(&window).await.unwrap();
    let mut last_render_time = instant::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                //println!("..000000000000000000000000000000000");
                if !input(&mut state, event) {
                    match event {
                        #[cfg(not(target_arch = "wasm32"))]
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
                            /*println!(
                                "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!00000>{:?}",
                                physical_size
                            );*/
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so we have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let now = instant::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                state.update(dt);
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }

            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }

            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                if state.camera_bundle.mouse_pressed {
                    state
                        .camera_bundle
                        .controller
                        .process_mouse(delta.0, delta.1)
                }
            }

            _ => {}
        }
    });
}

fn input(state: &mut State, event: &WindowEvent) -> bool {
    state.clear_color = wgpu::Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    match event {
        WindowEvent::CursorMoved { position: pos, .. } => {
            if state.keys.background {
                state.clear_color = wgpu::Color {
                    r: (pos.x / state.size.width as f64),
                    g: (pos.y / state.size.height as f64),
                    b: (pos.y + pos.x) / (state.size.width as f64 + state.size.height as f64),
                    a: 1.0,
                };
            }
        }

        WindowEvent::MouseWheel { delta, .. } => {
            state.camera_bundle.controller.process_scroll(delta);
        }

        WindowEvent::MouseInput {
            button: MouseButton::Left,
            state: element_state,
            ..
        } => {
            state.camera_bundle.mouse_pressed = *element_state == ElementState::Pressed;
        }

        WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(key),
                    ..
                },
            ..
        } => match key {
            VirtualKeyCode::Y => {
                state.keys.alt_shape = !state.keys.alt_shape;
                log::info!("Y changed render2 to {}", state.keys.alt_shape);
            }
            VirtualKeyCode::Tab => {
                state.keys.tab = !state.keys.tab;
                log::info!("TAB changed tab to {}", state.keys.tab);
            }
            VirtualKeyCode::P => {
                state.keys.screenshot = !state.keys.screenshot;
                log::info!("P changed screenshot to {}", state.keys.alt_image);
            }
            VirtualKeyCode::L => {
                state.keys.tex_loop = !state.keys.tex_loop;
                log::info!("L changed tex_loop to {}", state.keys.tex_loop);
            }
            VirtualKeyCode::R => {
                state.keys.rotate = !state.keys.rotate;
                log::info!("L changed rotate to {}", state.keys.rotate);
            }
            VirtualKeyCode::Z => {
                state.keys.show_depth = !state.keys.show_depth;
                log::info!("Z changed show_depth to {}", state.keys.show_depth);
            }
            VirtualKeyCode::B => {
                state.keys.background = !state.keys.background;
                log::info!("B changed background to {}", state.keys.background);
            }
            _ => {
                return state
                    .camera_bundle
                    .controller
                    .process_keyboard(*key, ElementState::Pressed)
            }
        },

        WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    state: ElementState::Released,
                    virtual_keycode: Some(key),
                    ..
                },
            ..
        } => {
            return state
                .camera_bundle
                .controller
                .process_keyboard(*key, ElementState::Released)
        }

        _ => return false,
    }
    true
}
