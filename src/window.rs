use winit::{
    event_loop::EventLoop,
    window::Window,
};


// pub fn build_context() -> (EventLoop<()>, Window, Renderer<'static>) {
//     let event_loop = EventLoop::new();

//     let window = winit::window::WindowBuilder::new()
//         .with_title("Pathfinder Metal".to_string())
//         .build(&event_loop)
//         .unwrap();

//     let renderer = Renderer::new(&window);
//     (event_loop, window, renderer)
// }
