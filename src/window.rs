use cocoa::appkit::{NSView, NSWindow};
use cocoa::base::id as cocoa_id;

use winit::{event_loop::EventLoop, platform::macos::WindowExtMacOS};

use font_kit::handle::Handle;
use pathfinder_canvas::CanvasFontContext;
use pathfinder_content::color::ColorF;
use pathfinder_geometry::vector::Vector2I;
use pathfinder_gpu::resources::{FilesystemResourceLoader, ResourceLoader};
use pathfinder_renderer::gpu::options::{DestFramebuffer, RendererOptions};
use pathfinder_renderer::gpu::renderer::Renderer;

use metal::*;
use pathfinder_metal::MetalDevice;

use objc::runtime::YES;

use std::mem;

pub struct RenderCtx {
    pub window: winit::window::Window,
    pub cocoa_window: cocoa_id,
    pub animation_layer: CoreAnimationLayer,
    pub event_loop: EventLoop<()>,
    pub renderer: Renderer<MetalDevice>,
    pub font_context: CanvasFontContext,
}

pub fn build_context() -> RenderCtx {
    let event_loop = EventLoop::new();

    let window = winit::window::WindowBuilder::new()
        .with_title("Pathfinder Metal".to_string())
        .build(&event_loop)
        .unwrap();

    let cocoa_window: cocoa_id = unsafe { mem::transmute(window.ns_window()) };
    let device = Device::system_default(); //.expect("no device found");
    let inner_size = window.inner_size();
    let window_size = Vector2I::new(inner_size.width as i32, inner_size.height as i32);

    let animation_layer = CoreAnimationLayer::new();
    animation_layer.set_device(&device);
    animation_layer.set_pixel_format(MTLPixelFormat::BGRA8Unorm);
    animation_layer.set_presents_with_transaction(false);

    unsafe {
        let view = cocoa_window.contentView();
        view.setWantsBestResolutionOpenGLSurface_(YES);
        view.setWantsLayer(YES);
        view.setLayer(mem::transmute(animation_layer.as_ref()));
    }

    let resource_loader = FilesystemResourceLoader::locate();
    let renderer = Renderer::new(
        MetalDevice::new(animation_layer.as_ref()),
        &resource_loader,
        DestFramebuffer::full_window(window_size),
        RendererOptions {
            background_color: Some(ColorF::transparent_black()),
        },
    );

    let font_data =
        std::sync::Arc::new(resource_loader.slurp("fonts/FiraMono-Regular.otf").unwrap());
    let font = Handle::from_memory(font_data, 0);
    let font_context = CanvasFontContext::from_fonts(std::iter::once(font));

    RenderCtx {
        window,
        cocoa_window,
        animation_layer,
        event_loop,
        renderer,
        font_context,
    }
}
