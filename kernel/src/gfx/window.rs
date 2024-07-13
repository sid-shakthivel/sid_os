use super::{psf::Font, rect::Rect};
use crate::{ds::queue::Queue, print_serial};

pub const WINDOW_BACKGROUND_COLOUR: u32 = 0xFFBBBBBB;
const WINDOW_BORDER_COLOUR: u32 = 0xFF000000;
const WINDOW_TITLE_COLOUR: u32 = 0x232422;
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
    pub colour: u32,
}

impl Window {
    pub const fn new(
        title: &'static str,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        colour: u32,
    ) -> Window {
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
            colour,
        }
    }

    pub fn generate_rect(&self) -> Rect {
        Rect::new(self.y, self.y + self.height, self.x + self.width, self.x)
    }

    pub fn paint(&self, dr: &Queue<Rect>, fb_addr: usize, font: &Font) {
        let mut rect = self.generate_rect();

        // Title bar
        let title_bar = Rect::new(self.y, self.y + WINDOW_TITLE_HEIGHT, rect.right, self.x);

        // Main area
        let main_area = Rect::new(
            self.y + WINDOW_TITLE_HEIGHT,
            rect.bottom,
            rect.right,
            self.x,
        );

        let text_start_x =
            self.x + ((self.width / 2) - (self.title.as_bytes().len() as u16 * 8) / 2);
        let text_start_y = self.y + (WINDOW_TITLE_HEIGHT - 16) / 2;

        // print_serial!("{:?}\n", title_bar);

        for rect_node in dr.list.into_iter() {
            let current_rect = rect_node.payload;

            // Paint main area
            current_rect.paint_against_region(&main_area, self.colour, fb_addr);

            // Paint top title bar
            current_rect.paint_against_region(&title_bar, WINDOW_TITLE_COLOUR, fb_addr);

            current_rect.paint_text(
                self.title,
                text_start_x,
                text_start_y,
                font,
                fb_addr,
                0xffffff,
            );
        }
    }

    pub fn paint_rect(&self, dr: &Rect, fb_addr: usize, font: &Font) {
        let mut rect = self.generate_rect();

        // Title bar
        let title_bar = Rect::new(self.y, self.y + WINDOW_TITLE_HEIGHT, rect.right, self.x);

        // Main area
        let main_area = Rect::new(
            self.y + WINDOW_TITLE_HEIGHT,
            rect.bottom,
            rect.right,
            self.x,
        );

        // print_serial!("{:?} {:?}\n", title_bar, dr);

        let text_start_x =
            self.x + ((self.width / 2) - (self.title.as_bytes().len() as u16 * 8) / 2);
        let text_start_y = self.y + (WINDOW_TITLE_HEIGHT - 16) / 2;

        // Paint main area
        dr.paint_against_region(&main_area, self.colour, fb_addr);

        // Paint top title bar
        dr.paint_against_region(&title_bar, WINDOW_TITLE_COLOUR, fb_addr);

        dr.paint_text(
            self.title,
            text_start_x,
            text_start_y,
            font,
            fb_addr,
            0xffffff,
        );
    }
}
