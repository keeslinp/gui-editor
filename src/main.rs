use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

use crossbeam_channel::unbounded;
use pathfinder_canvas::{CanvasRenderingContext2D, FillStyle, TextAlign};
use pathfinder_content::color::ColorU;
use pathfinder_geometry::vector::{Vector2F, Vector2I};
use pathfinder_renderer::concurrent::rayon::RayonExecutor;
use pathfinder_renderer::concurrent::scene_proxy::SceneProxy;
use pathfinder_renderer::options::BuildOptions;

mod window;

use window::{build_context, RenderCtx};

#[derive(Default)]
struct State {
    text: String,
}

enum Msg {
    CharPressed(char),
    KeyPressed(VirtualKeyCode),
}

fn update_state(state: &mut State, msg: Msg) -> bool {
    match msg {
        Msg::CharPressed(c) => {
            state.text.push(c);
            true
        }
        Msg::KeyPressed(key) => match key {
            VirtualKeyCode::Back => {
                state.text.pop();
                true
            }
            VirtualKeyCode::Return => {
                state.text.clear();
                true
            }
            _ => false,
        },
    }
}

fn main_loop(ctx: RenderCtx) {
    let mut state = State::default();

    let (msg_sender, msg_receiver) = unbounded();

    let RenderCtx {
        event_loop,
        window,
        mut renderer,
        font_context,
        ..
    } = ctx;
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::EventsCleared => {
                // Application update code.
                let mut should_render = false;
                for msg in msg_receiver.try_iter() {
                    should_render = should_render || update_state(&mut state, msg);
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
                // Make a canvas.
                let mut canvas =
                    CanvasRenderingContext2D::new(font_context.clone(), window_size.to_f32());

                // Draw the text.
                canvas.set_font_by_postscript_name("FiraMono-Regular");
                canvas.set_font_size(14.0);
                canvas.set_fill_style(FillStyle::Color(ColorU::from_u32(0xffffffff)));
                canvas.fill_text(&state.text, Vector2F::new(32.0, 48.0));
                canvas.set_text_align(TextAlign::Right);
                canvas.stroke_text("G", Vector2F::new(608.0, 464.0));

                // Render the canvas to screen.
                let scene = SceneProxy::from_scene(canvas.into_scene(), RayonExecutor);
                scene.build_and_render(&mut renderer, BuildOptions::default());
                renderer.device.present_drawable();
                // Redraw the application.
                //
                // It's preferrable to render in this event rather than in EventsCleared, since
                // rendering in here allows the program to gracefully handle redraws requested
                // by the OS.
            }
            Event::WindowEvent {
                event: WindowEvent::ReceivedCharacter(c),
                ..
            } => {
                msg_sender
                    .send(Msg::CharPressed(c))
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
                    .send(Msg::KeyPressed(keycode))
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
