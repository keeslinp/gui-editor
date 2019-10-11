use pathfinder_canvas::CanvasRenderingContext2D;
use pathfinder_geometry::vector::Vector2F;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    // Select,
}

impl Mode {
    fn as_str(&self) -> &'static str {
        match self {
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
            Mode::Command => "Command",
        }
    }
    pub fn render(&self, canvas: &mut CanvasRenderingContext2D, bounds: Vector2F) {
        let value = self.as_str();
        canvas.fill_text(value, bounds - Vector2F::new(value.len() as f32 * 9., 15.));
    }
}
