use winit::{
    event::{ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::ControlFlow,
};

use crossbeam_channel::{unbounded, Sender};
use pathfinder_canvas::{CanvasRenderingContext2D, FillStyle, TextAlign};
use pathfinder_content::color::ColorU;
use pathfinder_geometry::vector::{Vector2F, Vector2I};
use pathfinder_renderer::concurrent::rayon::RayonExecutor;
use pathfinder_renderer::concurrent::scene_proxy::SceneProxy;
use pathfinder_renderer::options::BuildOptions;

mod buffer;
mod command;
mod cursor;
mod handle_command;
mod input;
mod mode;
mod msg;
mod point;
mod state;
mod window;

use state::State;

use handle_command::handle_command;

use msg::{Cmd, InputMsg, Msg};
use window::{build_context, RenderCtx};

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

fn render(canvas: &mut CanvasRenderingContext2D, state: &State, window_size: Vector2F) {
    canvas.set_font_by_postscript_name("FiraMono-Regular");
    canvas.set_font_size(14.0);
    canvas.set_fill_style(FillStyle::Color(ColorU::from_u32(0xffffffff)));
    state.buffers[state.current_buffer].render(canvas);
    state.mode.render(canvas, window_size);
    if state.mode == mode::Mode::Command {
        state.command_buffer.render(canvas, window_size);
    }
    canvas.set_text_align(TextAlign::Right);
    canvas.stroke_text("G", Vector2F::new(608.0, 464.0));
}

fn main_loop(ctx: RenderCtx) {
    let mut state = State::new();

    let (msg_sender, msg_receiver) = unbounded();

    let RenderCtx {
        event_loop,
        window,
        mut renderer,
        font_context,
        ..
    } = ctx;
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
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let inner_size = window.inner_size();
                let window_size = Vector2I::new(inner_size.width as i32, inner_size.height as i32);
                let mut canvas =
                    CanvasRenderingContext2D::new(font_context.clone(), window_size.to_f32());

                render(&mut canvas, &state, window_size.to_f32());

                let scene = SceneProxy::from_scene(canvas.into_scene(), RayonExecutor);
                scene.build_and_render(&mut renderer, BuildOptions::default());
                renderer.device.present_drawable();
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

fn main() {
    let render_ctx = build_context();
    main_loop(render_ctx);
}
