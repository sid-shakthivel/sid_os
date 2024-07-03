use super::{psf::Font, rect::Rect};
use crate::ds::queue::Queue;

pub const WINDOW_BACKGROUND_COLOUR: u32 = 0xFFBBBBBB;
const WINDOW_BORDER_COLOUR: u32 = 0xFF000000;
const WINDOW_TITLE_COLOUR: u32 = 0x7092be;
const WINDOW_TITLE_HEIGHT: u16 = 20;
const OUTLINE_SIZE: u16 = 3;

#[derive(Clone, Copy, Debug)]
pub struct Window {
    pub title: &'static str,
    pub wid: i16,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Window {
    pub const fn new(title: &'static str, x: u16, y: u16, width: u16, height: u16) -> Window {
        /*
           Must constrain areas which are updated to certain regions
           Windows consist of: title bar, main area, outline, text
        */
        Window {
            title,
            wid: 0,
            x,
            y,
            width,
            height,
        }
    }

    pub fn generate_rect(&self) -> Rect {
        Rect::new(self.y, self.y + self.height, self.x + self.width, self.x)
    }
}
