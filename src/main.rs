use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
};

use wgpu_glyph::{Scale, Section};

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
mod error;

use render::{RenderFrame, Renderer};

use state::State;

use handle_command::handle_command;

use msg::{Cmd, InputMsg, Msg};

fn update_state(state: &mut State, msg: Msg, msg_sender: EventLoopProxy<Msg>, window_size: PhysicalSize) -> bool {
    match msg {
        Msg::Input(input_msg) => {
            if let Some(cmd) = input::build_cmd_from_input(input_msg, state.mode) {
                msg_sender.send_event(Msg::Cmd(cmd)).expect("sending command");
            }
            false // input does not alter state
        },
        Msg::Cmd(Cmd::SetError(err)) => {
            state.error = Some(err);
            true
        },
        Msg::Cmd(cmd_msg) => {
            match handle_command(state, cmd_msg, msg_sender.clone(), window_size) {
                Ok(should_render) => {
                    state.error = None;
                    should_render
                }
                Err(err) => {
                    msg_sender.send_event(Msg::Cmd(Cmd::SetError(err))).expect("setting error");
                    true
                }
            }
        }
    }
}

fn render(render_frame: &mut RenderFrame, state: &State, window_size: PhysicalSize) {
    render_frame.clear();
    state.buffers[state.current_buffer].render(render_frame, window_size);
    state.mode.render(render_frame, window_size);
    if state.mode == mode::Mode::Command {
        state.command_buffer.render(render_frame, window_size);
    }
    if let Some(ref error) = state.error {
        render_frame.queue_text(Section {
            text: &error.as_string(),
            screen_position: (10., window_size.height as f32 - 30.),
            color: [1., 0., 0., 1.],
            scale: Scale { x: 30., y: 30. },
            ..Section::default()
        });
    }
}

fn main() {
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

    let mut dirty = false;

    window.request_redraw();
    // TODO (perf): Do some performance improvements on this main loop
    // If there are any serious problems it is better to find out now
    event_loop.run(move |event, _, control_flow| {
        match event {
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
        }
    });
}
