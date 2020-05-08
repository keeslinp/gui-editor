use winit::{
    dpi::{PhysicalSize, LogicalSize},
    event::{ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
};
use futures::executor::block_on;

use structopt::StructOpt;

mod buffer;
mod color_scheme;
mod command;
mod cursor;
mod error;
mod handle_command;
mod input;
mod mode;
mod msg;
mod point;
// mod render;
mod skim_buffer;
mod state;
mod text_buffer;
use imgui::*;
use imgui_wgpu::Renderer;
use imgui_winit_support;
use std::time::Instant;

// use render::{RenderFrame, Renderer};

use anyhow::Result;

use state::State;

// use handle_command::handle_command;

use msg::{Cmd, InputMsg, Msg};

fn update_state(
    state: &mut State,
    msg: Msg,
    msg_sender: EventLoopProxy<Msg>,
    window_size: PhysicalSize<u32>,
) -> bool {
    match msg {
        Msg::Input(input_msg) => {
            input::process_input(input_msg, state.mode, |cmd| {
                msg_sender
                    .send_event(Msg::Cmd(cmd))
                    .expect("Failed to create command from input");
            });
            false
        }
        Msg::Cmd(Cmd::SetStatusText(status)) => {
            state.status = Some(status);
            true
        }
        Msg::Cmd(cmd_msg) => {
            true
            // match handle_command(state, cmd_msg, msg_sender.clone(), window_size) {
            //     Ok(should_render) => {
            //         state.status = None;
            //         should_render
            //     }
            //     Err(err) => {
            //         msg_sender
            //             .send_event(Msg::Cmd(Cmd::SetStatusText(err.to_string())))
            //             .expect("setting error");
            //         true
            //     }
            // }
        }
    }
}

// fn render(render_frame: &mut RenderFrame, state: &State, window_size: PhysicalSize<u32>) {
//     render_frame.clear();
//     use mode::Mode::*;
//     match state.mode {
//         Normal | Insert | Command | Jump => state.buffers[state.current_buffer].render(
//             render_frame,
//             window_size,
//             &state.color_scheme,
//         ),
//         Skim => state.skim_buffer.render(render_frame, window_size),
//     }
//     state.mode.render(render_frame, window_size);
//     if state.mode == mode::Mode::Command {
//         state.command_buffer.render(render_frame, window_size);
//     }
//     if let Some(ref status) = state.status {
//         render_frame.queue_text(Section {
//             text: status.as_str(),
//             screen_position: (10., window_size.height as f32 - 30.),
//             color: [1., 0., 0., 1.],
//             scale: Scale { x: 30., y: 30. },
//             ..Section::default()
//         });
//     }
// }

#[derive(Debug, StructOpt)]
#[structopt(name = "editor", about = "Simple Modal Editor with speed and efficiency as core goals")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: Option<std::path::PathBuf>,
    #[structopt(short, long)]
    perf: bool,

    #[structopt(long)]
    single_frame: bool,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let perf = opt.perf;
    let single_frame = opt.single_frame;

    flame::start("window setup");
    let event_loop: EventLoop<Msg> = EventLoop::with_user_event();
    let mut hidpi_factor = 1.0;

    // let window = winit::window::WindowBuilder::new()
    //     .with_title("Editor".to_string())
    //     .build(&event_loop)
    //     .unwrap();
    let window = winit::window::Window::new(&event_loop).unwrap();
    window.set_title("Editor");

    let size = window.inner_size();
    let surface = wgpu::Surface::create(&window);
    let adapter = block_on(wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
        },
        wgpu::BackendBit::PRIMARY,
    ))
    .unwrap();

    let (mut device, mut queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: wgpu::Limits::default(),
    }));

    // Set up swap chain
    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width: size.width as u32,
        height: size.height as u32,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    // Set up dear imgui
    let mut imgui = imgui::Context::create();
    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
    platform.attach_window(
        imgui.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Default,
    );
    imgui.set_ini_filename(None);

    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
    imgui.fonts().add_font(&[FontSource::DefaultFontData {
        config: Some(imgui::FontConfig {
            oversample_h: 1,
            pixel_snap_h: true,
            size_pixels: font_size,
            ..Default::default()
        }),
    }]);

    // imgui.fonts().add_font(&[FontSource::TtfData {
    //     data: include_bytes!("./render/FiraMono-Regular.ttf"),
    //     size_pixels: font_size,
    //     config: None,
    // }]);

    let clear_color = wgpu::Color {
        r: 0.1,
        g: 0.2,
        b: 0.3,
        a: 1.0,
    };

    let mut renderer = Renderer::new(
        &mut imgui,
        &device,
        &mut queue,
        sc_desc.format,
        Some(clear_color),
    );

    let mut last_frame = Instant::now();
    let mut last_cursor = None;
    // let mut renderer = Renderer::on_window(&window);

    let mut size = window.inner_size();
    flame::end("window setup");
    // END OF SETUP
    let mut state = State::new()?;

    let msg_sender = event_loop.create_proxy();

    if let Some(file_path) = opt.input {
        msg_sender.send_event(Msg::Cmd(Cmd::LoadFile(file_path)))?;
    }

    let mut dirty = false;

    window.request_redraw();
    // TODO (perf): Do some performance improvements on this main loop
    // If there are any serious problems it is better to find out now
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                if true || dirty {
                    window.request_redraw();
                }
            }
            Event::UserEvent(ref msg) => {
                // if msg == Msg::Cmd(Cmd::Quit) {
                //     if perf {
                //         flame::dump_html(&mut std::fs::File::create("flame-graph.html").unwrap()).unwrap();
                //     }
                //     *control_flow = ControlFlow::Exit;
                // } else {
                //     dirty = update_state(&mut state, msg, msg_sender.clone(), size) || dirty;
                // }
            }
            Event::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { scale_factor, .. },
                ..
            } => {
                hidpi_factor = scale_factor;
            }
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(new_size),
                ..
            } => {
                size = new_size;

                sc_desc = wgpu::SwapChainDescriptor {
                    usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    width: size.width as u32,
                    height: size.height as u32,
                    present_mode: wgpu::PresentMode::Mailbox,
                };

                swap_chain = device.create_swap_chain(&surface, &sc_desc);
                // renderer.update_size(size);
                window.request_redraw();
            }
            Event::RedrawEventsCleared { .. } => {
                let delta_s = last_frame.elapsed();
                last_frame = imgui.io_mut().update_delta_time(last_frame);

                let frame = match swap_chain.get_next_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        eprintln!("dropped frame: {:?}", e);
                        return;
                    }
                };
                platform
                    .prepare_frame(imgui.io_mut(), &window)
                    .expect("Failed to prepare frame");
                let ui = imgui.frame();

                {
                    let window = imgui::Window::new(im_str!("Hello world"));
                    window
                        .size([300.0, 100.0], Condition::FirstUseEver)
                        .build(&ui, || {
                            ui.text(im_str!("Hello world!"));
                            ui.text(im_str!("This...is...imgui-rs on WGPU!"));
                            ui.separator();
                            let mouse_pos = ui.io().mouse_pos;
                            ui.text(im_str!(
                                "Mouse Position: ({:.1},{:.1})",
                                mouse_pos[0],
                                mouse_pos[1]
                            ));
                        });

                    let window = imgui::Window::new(im_str!("Hello too"));
                    window
                        .size([400.0, 200.0], Condition::FirstUseEver)
                        .position([400.0, 200.0], Condition::FirstUseEver)
                        .build(&ui, || {
                            ui.text(im_str!("Frametime: {:?}", delta_s));
                        });

                    ui.show_demo_window(&mut true);
                }

                let mut encoder: wgpu::CommandEncoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                if last_cursor != Some(ui.mouse_cursor()) {
                    last_cursor = Some(ui.mouse_cursor());
                    platform.prepare_render(&ui, &window);
                }
                renderer
                    .render(ui.render(), &mut device, &mut encoder, &frame.view)
                    .expect("Rendering failed");

                queue.submit(&[encoder.finish()]);
                // let mut render_frame = renderer.start_frame();
                // render(&mut render_frame, &state, size);
                // render_frame.submit(&size);
                if single_frame {
                    msg_sender.send_event(Msg::Cmd(Cmd::Quit)).unwrap();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::ReceivedCharacter(c),
                ..
            } => {
                msg_sender
                    .send_event(Msg::Input(InputMsg::CharPressed(c)))
                    .expect("sending char event");
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                // msg_sender
                //     .send_event(Msg::Input(InputMsg::KeyPressed(keycode)))
                //     .expect("sending key event");
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            }
            _ => *control_flow = ControlFlow::Wait,
        }
        platform.handle_event(imgui.io_mut(), &window, &event);
    });
}
