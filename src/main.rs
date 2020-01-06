use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
};

use wgpu_glyph::{Scale, Section};
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
mod render;
mod skim_buffer;
mod state;
mod text_buffer;

use render::{RenderFrame, Renderer};

use state::State;

use handle_command::handle_command;

use msg::{Cmd, InputMsg, Msg};


fn update_state(
    state: &mut State,
    msg: Msg,
    msg_sender: EventLoopProxy<Msg>,
    window_size: PhysicalSize,
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
            match handle_command(state, cmd_msg, msg_sender.clone(), window_size) {
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
            }
        }
    }
}

fn render(render_frame: &mut RenderFrame, state: &State, window_size: PhysicalSize) {
    render_frame.clear();
    use mode::Mode::*;
    match state.mode {
        Normal | Insert | Command | Jump => {
            state.buffers[state.current_buffer].render(render_frame, window_size)
        }
        Skim => state.skim_buffer.render(render_frame, window_size),
    }
    state.mode.render(render_frame, window_size);
    if state.mode == mode::Mode::Command {
        state.command_buffer.render(render_frame, window_size);
    }
    if let Some(ref status) = state.status {
        render_frame.queue_text(Section {
            text: status.as_str(),
            screen_position: (10., window_size.height as f32 - 30.),
            color: [1., 0., 0., 1.],
            scale: Scale { x: 30., y: 30. },
            ..Section::default()
        });
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: Option<std::path::PathBuf>,
}

fn main() {
    let opt = Opt::from_args();

    let event_loop: EventLoop<Msg> = EventLoop::with_user_event();

    let window = winit::window::WindowBuilder::new()
        .with_title("Editor".to_string())
        .build(&event_loop)
        .unwrap();

    let mut renderer = Renderer::on_window(&window);

    let mut size = window.inner_size().to_physical(window.hidpi_factor());

    // END OF SETUP
    let mut state = State::new();

    let msg_sender = event_loop.create_proxy();

    if let Some(file_path) = opt.input {
        msg_sender.send_event(Msg::Cmd(Cmd::LoadFile(file_path))).expect("Sending file open event");
    }

    let mut dirty = false;

    window.request_redraw();
    // TODO (perf): Do some performance improvements on this main loop
    // If there are any serious problems it is better to find out now
    event_loop.run(move |event, _, control_flow| match event {
        Event::EventsCleared => {
            if dirty {
                window.request_redraw();
            }
        }
        Event::UserEvent(msg) => {
            if msg == Msg::Cmd(Cmd::Quit) {
                *control_flow = ControlFlow::Exit;
            } else {
                dirty = update_state(&mut state, msg, msg_sender.clone(), size) || dirty;
            }
        }
        winit::event::Event::WindowEvent {
            event: winit::event::WindowEvent::Resized(new_size),
            ..
        } => {
            size = new_size.to_physical(window.hidpi_factor());
            renderer.update_size(size);
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
    });
}
