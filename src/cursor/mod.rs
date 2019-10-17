use crate::{msg::Direction, point::Point, render::RenderFrame};
use ropey::Rope;

pub struct Cursor {
    position: Point,
    saved_x: u16,
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            position: Point::default(),
            saved_x: 0,
        }
    }

    pub fn index(&self, rope: &Rope) -> usize {
        self.position.index(rope)
    }

    pub fn step(&mut self, direction: Direction, rope: &Rope) {
        self.position.step(direction, rope);
        match direction {
            Direction::Left | Direction::Right => {
                self.saved_x = self.position.x;
            }
            Direction::Up | Direction::Down => {
                if self.saved_x > self.position.x {
                    self.position.x = self.saved_x;
                }
                self.position.prevent_runoff(rope);
            }
        }
    }

    pub fn render(&self, render_frame: &mut RenderFrame) {
        render_frame.queue_quad(self.position.x as f32 * 15. + 10., self.position.y as f32 * 25. + 10., 14., 30.);
    }
}
