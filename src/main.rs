use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crossbeam_channel::{unbounded, Sender};

mod buffer;
mod command;
mod cursor;
mod handle_command;
mod input;
mod mode;
mod msg;
mod point;
mod render;
mod state;

use render::{RenderFrame, Renderer};

use state::State;

use handle_command::handle_command;

use msg::{Cmd, InputMsg, Msg};

fn update_state(state: &mut State, msg: Msg, msg_sender: Sender<Msg>) -> bool {
    match msg {
        Msg::Input(input_msg) => {
            if let Some(cmd) = input::build_cmd_from_input(input_msg, state.mode) {
                msg_sender.send(Msg::Cmd(cmd)).expect("sending command");
            }
            false // input does not alter state
        }
        Msg::Cmd(cmd_msg) => handle_command(state, cmd_msg, msg_sender),
    }
}

fn render(render_frame: &mut RenderFrame, state: &State, window_size: PhysicalSize) {
    render_frame.clear();
    state.buffers[state.current_buffer].render(render_frame);
    state.mode.render(render_frame, window_size);
    if state.mode == mode::Mode::Command {
        state.command_buffer.render(render_frame, window_size);
    }
}

fn main() {
    // All this is here because of lifetime bullshit
    let event_loop = EventLoop::new();

    let window = winit::window::WindowBuilder::new()
        .with_title("Pathfinder Metal".to_string())
        .build(&event_loop)
        .unwrap();

    let mut renderer = Renderer::on_window(&window);

    let mut size = window.inner_size().to_physical(window.hidpi_factor());

    // END OF SETUP
    let mut state = State::new();

    let (msg_sender, msg_receiver) = unbounded();

    window.request_redraw();
    // TODO (perf): Do some performance improvements on this main loop
    // If there are any serious problems it is better to find out now
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::EventsCleared => {
                let mut should_render = false;
                for msg in msg_receiver.try_iter() {
                    if msg == Msg::Cmd(Cmd::Quit) {
                        *control_flow = ControlFlow::Exit;
                    } else {
                        should_render =
                            update_state(&mut state, msg, msg_sender.clone()) || should_render;
                    }
                }
                // Queue a RedrawRequested event.
                if should_render {
                    window.request_redraw();
                }
            }
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(new_size),
                ..
            } => {
                size = new_size.to_physical(window.hidpi_factor());
                renderer.update_size(size);
                // swap_chain = device.create_swap_chain(
                //     &surface,
                //     &wgpu::SwapChainDescriptor {
                //         usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                //         format: render_format,
                //         width: size.width.round() as u32,
                //         height: size.height.round() as u32,
                //         present_mode: wgpu::PresentMode::Vsync,
                //     },
                // );
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let mut render_frame = renderer.start_frame();
                render(&mut render_frame, &state, size);
                render_frame.submit(&size);
            }
            Event::WindowEvent {
                event: WindowEvent::ReceivedCharacter(c),
                ..
            } => {
                msg_sender
                    .send(Msg::Input(InputMsg::CharPressed(c)))
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
                    .send(Msg::Input(InputMsg::KeyPressed(keycode)))
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
