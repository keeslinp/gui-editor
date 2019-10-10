use ropey::Rope;
use pathfinder_canvas::{CanvasRenderingContext2D};
use pathfinder_geometry::{
    vector::Vector2F,
    rect::RectF,
};
use crate::{
    msg::{Direction},
    point::Point,
};

pub struct Cursor {
    position: Point
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            position: Point::default(),
        }
    }

    pub fn index(&self, rope: &Rope) -> usize {
        self.position.index(rope)
    }

    pub fn step(&mut self, direction: Direction, rope: &Rope) {
        self.position.step(direction, rope);
    }

    pub fn render(&self, canvas: &mut CanvasRenderingContext2D) {
        canvas.fill_rect(RectF::new(Vector2F::new(self.position.x as f32 * 8.4 + 10., self.position.y as f32 * 20.), Vector2F::new(8., 14.)));
    }
}
