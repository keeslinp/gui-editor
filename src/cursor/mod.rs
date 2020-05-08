use crate::{
    msg::{Direction, JumpType},
    point::Point,
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
        ui: &imgui::Ui,
        horizontal_offset: f32,
    ) {
        let line_height = ui.text_line_height_with_spacing();
        let left = ((self.position.x as f32 + 1.2) * 6.5) + horizontal_offset;
        let top = (self.position.y + 1) as f32 * line_height - ui.scroll_y();
        let bottom = top + line_height;
        let right = left + 7.;
        ui.get_window_draw_list().add_rect([left, top], [right, bottom], [1., 1., 1., 0.2]).filled(true).build();
        let window_height = ui.window_size()[1];
        if bottom > window_height {
            ui.set_scroll_from_pos_y_with_ratio(bottom + 5., 1.);
        }
        if top < 0. {
            ui.set_scroll_from_pos_y_with_ratio(top - 5., 0.);
        }
    }

    pub fn row(&self) -> usize {
        self.position.y as usize
    }
    pub fn jump(&mut self, jump_type: JumpType, rope: &RopeSlice) {
        self.position.jump(jump_type, rope);
        self.saved_x = self.position.x;
    }
}
