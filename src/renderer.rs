use wgpu::{CommandEncoder, Device, Surface, SwapChain, SwapChainOutput, Queue};
use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, Section};
use winit::dpi::PhysicalSize;
// use winit::window::Window;

// pub struct Renderer<'a> {
//     glyph_brush: GlyphBrush<'a, ()>,
//     swap_chain: SwapChain,
//     device: Device,
//     surface: Surface,
// }

// Lifetime hell
// 'a is for the glyph_brush lasting for Renderer's lifetime
// 'b is to hold a reference to glyph_brush inside of RenderFrame
// impl<'a:'b, 'b> Renderer<'a> {
//     pub fn new(window: &Window) -> Renderer<'a> {
//         let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
//             power_preference: wgpu::PowerPreference::HighPerformance,
//             backends: wgpu::BackendBit::all(),
//         })
//         .expect("Request adapter");

//         let (mut device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
//             extensions: wgpu::Extensions {
//                 anisotropic_filtering: false,
//             },
//             limits: wgpu::Limits { max_bind_groups: 1 },
//         });

//         let render_format = wgpu::TextureFormat::Bgra8UnormSrgb;
//         let mut size = window.inner_size().to_physical(window.hidpi_factor());

//         let surface = wgpu::Surface::create(window);

//         // Prepare glyph_brush
//         let inconsolata: &'a[u8] = include_bytes!("FiraMono-Regular.otf");
//         let mut glyph_brush =
//             GlyphBrushBuilder::using_font_bytes(inconsolata).build(&mut device, render_format);
//         let mut swap_chain = device.create_swap_chain(
//             &surface,
//             &wgpu::SwapChainDescriptor {
//                 usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
//                 format: render_format,
//                 width: size.width.round() as u32,
//                 height: size.height.round() as u32,
//                 present_mode: wgpu::PresentMode::Vsync,
//             },
//         );
//         Renderer {
//             glyph_brush,
//             swap_chain,
//             device,
//             surface,
//         }
//     }
//     // pub fn start_frame(&mut self) {
//     //     println!("test");
//     // }
//     pub fn start_frame(&mut self) -> RenderFrame<'a, 'b> {

//         let frame: SwapChainOutput<'b> = self.swap_chain.get_next_texture();
//         let glyph_brush: &'b mut GlyphBrush<'a, ()> = &mut self.glyph_brush;
//         RenderFrame {
//             frame,
//             glyph_brush,
//             encoder,
//         }
//     }

//     pub fn update_swapchain(&mut self, window: &Window, new_size: winit::dpi::LogicalSize) {
//         let size = new_size.to_physical(window.hidpi_factor());
//         let render_format = wgpu::TextureFormat::Bgra8UnormSrgb;
//         self.swap_chain = self.device.create_swap_chain(
//             &self.surface,
//             &wgpu::SwapChainDescriptor {
//                 usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
//                 format: render_format,
//                 width: size.width.round() as u32,
//                 height: size.height.round() as u32,
//                 present_mode: wgpu::PresentMode::Vsync,
//             },
//         );
//     }
// }

pub struct RenderFrame<'a, 'b> {
    frame: SwapChainOutput<'b>,
    encoder: CommandEncoder,
    glyph_brush: &'b mut GlyphBrush<'a, ()>,
    queue: &'b mut Queue,
    device: &'b mut Device,
}

impl<'a, 'b> RenderFrame<'a, 'b> {
    pub fn start_frame(swap_chain: &'b mut SwapChain, device: &'b mut Device, glyph_brush: &'b mut GlyphBrush<'a, ()>, queue: &'b mut Queue) -> Self {
        let frame: SwapChainOutput<'b> = swap_chain.get_next_texture();
        let mut encoder = device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { todo: 0 },
        );
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color {
                    r: 0.,
                    g: 0.169,
                    b: 0.2117,
                    a: 1.0,
                },
            }],
            depth_stencil_attachment: None,
        });
        RenderFrame {
            frame,
            encoder,
            glyph_brush,
            queue,
            device,
        }
    }
    pub fn queue_text(&mut self, section: Section) {
        self.glyph_brush.queue(section);
    }

    pub fn submit(mut self, size: &PhysicalSize) {
        self.glyph_brush
            .draw_queued(
                &mut self.device,
                &mut self.encoder,
                &self.frame.view,
                size.width.round() as u32,
                size.height.round() as u32,
            )
            .expect("Draw queued");
        self.queue.submit(&[self.encoder.finish()]);
    }
}
