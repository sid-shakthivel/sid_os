use super::rect::Rect;

#[derive(Clone)]
pub struct Window {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl Window {
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Window {
        Window {
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
