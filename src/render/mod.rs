use wgpu::{CommandEncoder, Device, Queue, Surface, SwapChain, SwapChainOutput};
use wgpu_glyph::Section;
use winit::{dpi::PhysicalSize, window::Window};

mod font;
mod quad;

use font::Font;

pub struct Renderer {
    font: Font,
    swap_chain: SwapChain,
    queue: Queue,
    device: Device,
    surface: Surface,
}

const RENDER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

impl Renderer {
    pub fn on_window(window: &Window) -> Self {
        let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            backends: wgpu::BackendBit::all(),
        })
        .expect("Request adapter");

        let (mut device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits { max_bind_groups: 1 },
        });

        let surface = wgpu::Surface::create(window);
        let inconsolata: &[u8] = include_bytes!("FiraMono-Regular.ttf");
        let font = Font::from_bytes(&mut device, &inconsolata);

        let size = window.inner_size().to_physical(window.hidpi_factor());
        let swap_chain = device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: RENDER_FORMAT,
                width: size.width.round() as u32,
                height: size.height.round() as u32,
                present_mode: wgpu::PresentMode::Vsync,
            },
        );
        Renderer {
            font,
            swap_chain,
            queue,
            device,
            surface,
        }
    }

    pub fn update_size(&mut self, size: PhysicalSize) {
        self.swap_chain = self.device.create_swap_chain(
            &self.surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: RENDER_FORMAT,
                width: size.width.round() as u32,
                height: size.height.round() as u32,
                present_mode: wgpu::PresentMode::Vsync,
            },
        );
    }

    pub fn start_frame<'a>(&'a mut self) -> RenderFrame<'a> {
        let frame: SwapChainOutput<'a> = self.swap_chain.get_next_texture();
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        RenderFrame {
            frame,
            font: &mut self.font,
            queue: &mut self.queue,
            device: &mut self.device,
            encoder,
        }
    }
}

pub struct RenderFrame<'a> {
    frame: SwapChainOutput<'a>,
    font: &'a mut Font,
    queue: &'a mut Queue,
    device: &'a mut Device,
    encoder: CommandEncoder,
}

impl<'a> RenderFrame<'a> {
    pub fn clear(&mut self) {
        let _ = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &self.frame.view,
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
    }
    pub fn queue_text(&mut self, section: Section) {
        self.font.queue(section);
    }

    pub fn submit(mut self, size: &PhysicalSize) {
        self.font
            .draw(self.device, &mut self.encoder, &self.frame.view, size);
        self.queue.submit(&[self.encoder.finish()]);
    }
}
