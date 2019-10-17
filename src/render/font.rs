use wgpu::TextureView;
use wgpu_glyph::Section;
use winit::dpi::PhysicalSize;

pub struct Font {
    glyphs: wgpu_glyph::GlyphBrush<'static, ()>,
}

impl Font {
    pub fn from_bytes(device: &mut wgpu::Device, bytes: &'static [u8]) -> Font {
        Font {
            glyphs: wgpu_glyph::GlyphBrushBuilder::using_font_bytes(bytes)
                .texture_filter_method(wgpu::FilterMode::Nearest)
                .build(device, wgpu::TextureFormat::Bgra8UnormSrgb),
        }
    }

    pub fn queue(&mut self, section: Section<'_>) {
        self.glyphs.queue(section);
    }

    pub fn draw(
        &mut self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &TextureView,
        size: &PhysicalSize,
    ) {
        self.glyphs
            .draw_queued(
                device,
                encoder,
                target,
                size.width.round() as u32,
                size.height.round() as u32,
            )
            .expect("Draw font");
    }
}
