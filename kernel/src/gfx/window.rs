use super::rect::Rect;

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

    pub fn paint(&self) {}
}
