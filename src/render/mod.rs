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
    quad_pipeline: quad::Pipeline,
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

        let quad_pipeline = quad::Pipeline::new(&mut device);

        let size = window.inner_size();
        let swap_chain = device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: RENDER_FORMAT,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Vsync,
            },
        );
        Renderer {
            font,
            swap_chain,
            queue,
            device,
            surface,
            quad_pipeline,
        }
    }

    pub fn update_size(&mut self, size: PhysicalSize<u32>) {
        self.swap_chain = self.device.create_swap_chain(
            &self.surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: RENDER_FORMAT,
                width: size.width,
                height: size.height,
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
            quads: Vec::new(),
            quad_pipeline: &mut self.quad_pipeline,
        }
    }
}

pub struct RenderFrame<'a> {
    frame: SwapChainOutput<'a>,
    font: &'a mut Font,
    queue: &'a mut Queue,
    device: &'a mut Device,
    encoder: CommandEncoder,
    quads: Vec<quad::Quad>,
    quad_pipeline: &'a mut quad::Pipeline,
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

    pub fn queue_quad(&mut self, x_pos: f32, y_pos: f32, width: f32, height: f32) {
        self.quads
            .push(quad::Quad::new(x_pos, y_pos, width, height));
    }

    pub fn submit(mut self, size: &PhysicalSize<u32>) {
        self.font
            .draw(self.device, &mut self.encoder, &self.frame.view, size);
        use quad::{Mat4, Vec2, Vec3};
        let transformation = Mat4::scaling_3d(Vec3::new(
            2. / size.width as f32,
            2. / size.height as f32,
            1.,
        ))
        .translated_2d(Vec2::new(-1., -1.));
        self.quad_pipeline.draw(
            self.device,
            &mut self.encoder,
            &self.quads,
            &transformation,
            &self.frame.view,
        );
        self.queue.submit(&[self.encoder.finish()]);
    }
}
