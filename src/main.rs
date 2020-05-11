use futures::executor::block_on;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
};

use structopt::StructOpt;

mod buffer;
mod command;
mod cursor;
mod error;
mod handle_command;
mod input;
mod mode;
mod msg;
mod point;
mod skim_buffer;
mod state;
mod text_buffer;
use imgui::*;
use imgui_wgpu::Renderer;
use imgui_winit_support;
use std::time::Instant;

use anyhow::Result;

use state::State;

use handle_command::handle_command;

use msg::{Cmd, InputMsg, Msg};

fn update_state(state: &mut State, msg: Msg, msg_sender: EventLoopProxy<Msg>) -> bool {
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
        Msg::Cmd(cmd_msg) => match handle_command(state, cmd_msg, msg_sender.clone()) {
            Ok(should_render) => {
                state.status = None;
                should_render
            }
            Err(err) => {
                msg_sender
                    .send_event(Msg::Cmd(Cmd::SetStatusText(err.to_string())))
                    .expect("setting error");
                true
            }
        },
    }
}

fn render(ui: &imgui::Ui, state: &State, size: &PhysicalSize<u32>) {
    use mode::Mode::*;
    let mut buffer_height = size.height as f32 / 2.;
    let status_window = imgui::Window::new(im_str!("Status"));
    status_window
        .size([size.width as f32 / 2., 20.], Condition::Always)
        .position([0., size.height as f32 / 2. - 20.], Condition::Always)
        .movable(false)
        .scrollable(false)
        .no_decoration()
        .draw_background(false)
        .build(&ui, || {
            state.mode.render(ui);
            match state.mode {
                Skim => state.skim_buffer.render_bar(ui),
                Command => state.command_buffer.render(ui),
                _ => {}
            }
            if let Some(ref status) = &state.status {
                ui.set_cursor_pos([10., 0.]);
                let im_string = imgui::ImString::new(status);
                ui.text(im_string);
            }
            buffer_height -= ui.window_size()[1];
        });
    let main_window = imgui::Window::new(im_str!("Main"));
    main_window
        .size([size.width as f32 / 2., buffer_height], Condition::Always)
        .position([0., 0.], Condition::Always)
        .movable(false)
        .no_decoration()
        .draw_background(false)
        .build(&ui, || match state.mode {
            Normal | Insert | Command | Jump => {
                state.buffers[state.current_buffer].render(ui, &state.theme, &state.syntax_set)
            }
            Skim => state.skim_buffer.render(ui),
        });
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "editor",
    about = "Simple Modal Editor with speed and efficiency as core goals"
)]
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

    imgui.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("./FiraMono-Regular.ttf"),
        size_pixels: font_size,
        config: None,
    }]);

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
        platform.handle_event(imgui.io_mut(), &window, &event);
        match event {
            Event::MainEventsCleared => {
                if dirty {
                    window.request_redraw();
                }
            }
            Event::UserEvent(msg) => {
                if msg == Msg::Cmd(Cmd::Quit) {
                    if perf {
                        flame::dump_html(&mut std::fs::File::create("flame-graph.html").unwrap())
                            .unwrap();
                    }
                    *control_flow = ControlFlow::Exit;
                } else {
                    dirty = update_state(&mut state, msg, msg_sender.clone()) || dirty;
                }
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
                window.request_redraw();
            }
            Event::RedrawEventsCleared { .. } => {
                let _delta_s = last_frame.elapsed();
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
                    render(&ui, &state, &size);
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
                msg_sender
                    .send_event(Msg::Input(InputMsg::KeyPressed(keycode)))
                    .expect("sending key event");
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
    });
}
