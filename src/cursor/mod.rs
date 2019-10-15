use crate::{msg::Direction, point::Point};
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

    // pub fn render(&self, canvas: &mut CanvasRenderingContext2D) {
    //     canvas.fill_rect(RectF::new(
    //         Vector2F::new(
    //             self.position.x as f32 * 8.4 + 10.,
    //             self.position.y as f32 * 20.,
    //         ),
    //         Vector2F::new(8., 14.),
    //     ));
    // }
}
