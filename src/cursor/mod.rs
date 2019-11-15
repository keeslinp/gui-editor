use crate::{
    msg::{Direction, JumpType},
    point::Point,
    render::RenderFrame,
};
use ropey::RopeSlice;

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

    pub fn index(&self, rope: &RopeSlice) -> usize {
        self.position.index(rope)
    }

    pub fn step(&mut self, direction: Direction, rope: &RopeSlice) {
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

    pub fn render(
        &self,
        render_frame: &mut RenderFrame,
        horizontal_offset: usize,
        vertical_offset: usize,
    ) {
        render_frame.queue_quad(
            // horizontal is added because it is to make room for line numbers
            (f32::from(self.position.x) + horizontal_offset as f32) * 15. + 30.,
            // vertical is subbed because it adjusts for scrolling
            (f32::from(self.position.y) - vertical_offset as f32) * 25. + 10.,
            14.,
            30.,
        );
    }

    pub fn row(&self) -> usize {
        self.position.y as usize
    }
    pub fn jump(&mut self, jump_type: JumpType, rope: &RopeSlice) {
        self.position.jump(jump_type, rope);
        self.saved_x = self.position.x;
    }
}
