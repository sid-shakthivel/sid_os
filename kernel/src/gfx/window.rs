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

// impl Window {
//     pub const fn new(title: &'static str, x: u16, y: u16, width: u16, height: u16) -> Window {
//         /*
//            Must constrain areas which are updated to certain regions
//            Windows consist of: title bar, main area, outline, text
//         */
//         Window {
//             title,
//             wid: 0,
//             x,
//             y,
//             width,
//             height,
//         }
//     }

//     pub fn generate_rect(&self) -> Rect {
//         Rect::new(self.y, self.y + self.height, self.x + self.width, self.x)
//     }

//     pub fn paint_dirty(&self, dirty_rects: &mut Queue<Rect>, fb_addr: usize, font: &Font) {
//         let mut rect = self.generate_rect();

//         // Title bar
//         let constrained_area_1 = Rect::new(
//             self.y + OUTLINE_SIZE,
//             self.y + WINDOW_TITLE_HEIGHT + OUTLINE_SIZE,
//             rect.right - OUTLINE_SIZE,
//             self.x + OUTLINE_SIZE,
//         );

//         // Main area
//         let constrained_area_2 = Rect::new(
//             self.y + OUTLINE_SIZE + WINDOW_TITLE_HEIGHT,
//             rect.bottom - OUTLINE_SIZE,
//             rect.right - OUTLINE_SIZE,
//             self.x + OUTLINE_SIZE,
//         );

//         let x_start = self.x + ((self.width / 2) - (self.title.as_bytes().len() as u16 * 8) / 2);
//         let y_start = self.y + (WINDOW_TITLE_HEIGHT + OUTLINE_SIZE - 10) / 2;

//         for rect in dirty_rects.list.into_iter() {
//             let rect = rect.unwrap().payload;

//             // Paint the outline
//             rect.paint_rect_outline(WINDOW_BORDER_COLOUR, fb_addr, &rect);

//             // Paint top title bar
//             rect.paint_special(WINDOW_TITLE_COLOUR, fb_addr, &constrained_area_1);

//             // Paint main area
//             rect.paint_special(WINDOW_BACKGROUND_COLOUR, fb_addr, &constrained_area_2);

//             rect.paint_text(self.title, x_start, y_start, font, fb_addr);
//         }
//     }

//     pub fn paint(&self, above_windows: &mut Queue<Window>, fb_addr: usize, font: &Font) {
//         let mut rect = self.generate_rect();

//         // Title bar
//         let constrained_area_1 = Rect::new(
//             self.y + OUTLINE_SIZE,
//             self.y + WINDOW_TITLE_HEIGHT + OUTLINE_SIZE,
//             rect.right - OUTLINE_SIZE,
//             self.x + OUTLINE_SIZE,
//         );

//         // Main area
//         let constrained_area_2 = Rect::new(
//             self.y + OUTLINE_SIZE + WINDOW_TITLE_HEIGHT,
//             rect.bottom - OUTLINE_SIZE,
//             rect.right - OUTLINE_SIZE,
//             self.x + OUTLINE_SIZE,
//         );

//         let x_start = self.x + ((self.width / 2) - (self.title.as_bytes().len() as u16 * 8) / 2);
//         let y_start = self.y + (WINDOW_TITLE_HEIGHT + OUTLINE_SIZE - 10) / 2;

//         let mut clipped_rects = Queue::<Rect>::new();
//         clipped_rects.enqueue(rect);

//         for window in above_windows.list.into_iter() {
//             let clipping_rect = window.unwrap().payload.clone().generate_rect();
//             Rect::split_rects(&mut clipped_rects, &clipping_rect);
//         }

//         for rect in clipped_rects.list.into_iter() {
//             let rect = rect.unwrap().payload;
//             rect.paint(WINDOW_BACKGROUND_COLOUR, fb_addr);

//             // Paint the outline
//             rect.paint_rect_outline(WINDOW_BORDER_COLOUR, fb_addr, &rect);

//             // Paint top title bar
//             rect.paint_special(WINDOW_TITLE_COLOUR, fb_addr, &constrained_area_1);

//             // Paint main area
//             rect.paint_special(WINDOW_BACKGROUND_COLOUR, fb_addr, &constrained_area_2);

//             rect.paint_text(self.title, x_start, y_start, font, fb_addr);
//         }

//         clipped_rects.empty();
//     }
// }
