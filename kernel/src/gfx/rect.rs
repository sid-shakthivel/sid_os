use crate::{
    ds::{list::List, queue::Queue},
    print_serial,
};

use super::psf::Font;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rect {
    pub top: u16,
    pub bottom: u16,
    pub right: u16,
    pub left: u16,
}

impl Rect {
    pub const fn new(top: u16, bottom: u16, right: u16, left: u16) -> Rect {
        Rect {
            top,
            bottom,
            right,
            left,
        }
    }

    pub fn does_intersect(&self, rect: &Rect) -> bool {
        return self.left < rect.right
            && self.right > rect.left
            && self.top < rect.bottom
            && self.bottom > rect.top;
    }

    pub fn paint_text(
        &self,
        text: &'static str,
        mut base_x: u16,
        base_y: u16,
        font: &Font,
        fb_addr: usize,
    ) {
        // May break at some point but ...

        for byte in text.as_bytes() {
            self.draw_clipped_character(*byte as char, base_x, base_y, font, fb_addr);
            base_x += 8;
        }
    }

    fn draw_clipped_character(&self, character: char, x: u16, y: u16, font: &Font, fb_addr: usize) {
        let glyph_address = (font.start_addr
            + font.metadata.header_size
            + (font.metadata.bytes_per_glyph * (character as u32)))
            as *mut u8;

        for cy in 0..16 {
            let mut index = 8;
            for cx in 0..8 {
                let adjusted_x = x + cx;
                let adjusted_y = y + cy;

                // Load correct bitmap for glyph
                let glyph_offset: u16 =
                    unsafe { (*glyph_address.offset(cy as isize) as u16) & (1 << index) };
                if glyph_offset > 0 {
                    let fb_offset = ((fb_addr as u32)
                        + (adjusted_y as u32 * 4096)
                        + ((adjusted_x as u32 * 32) / 8))
                        as *mut u32;
                    unsafe {
                        *fb_offset = 0x00;
                    }
                }
                index -= 1;
            }
        }
    }

    pub fn paint_rect_outline(&self, colour: u32, fb_address: usize, constrained_area: &Rect) {
        // Need to constrict top, bottom, left and right

        let clip_top = Rect::new(
            constrained_area.top,
            constrained_area.top + 3,
            constrained_area.right,
            constrained_area.left,
        );

        let clip_bottom = Rect::new(
            constrained_area.bottom - 3,
            constrained_area.bottom,
            constrained_area.right,
            constrained_area.left,
        );

        let clip_left = Rect::new(
            constrained_area.top,
            constrained_area.bottom,
            constrained_area.left + 3,
            constrained_area.left,
        );

        let clip_right = Rect::new(
            constrained_area.top,
            constrained_area.bottom,
            constrained_area.right,
            constrained_area.right - 3,
        );

        self.paint_special(colour, fb_address, &clip_top);
        self.paint_special(colour, fb_address, &clip_bottom);
        self.paint_special(colour, fb_address, &clip_left);
        self.paint_special(colour, fb_address, &clip_right);
    }

    // Paints against a specific region
    pub fn paint_special(&self, colour: u32, fb_address: usize, contrained_area: &Rect) {
        // Clamp writeable area to both the clipped region and the contrained area in which it should be
        let x_base = core::cmp::max(contrained_area.left, self.left);
        let y_base = core::cmp::max(contrained_area.top, self.top);
        let x_limit = core::cmp::min(contrained_area.right, self.right);
        let y_limit = core::cmp::min(contrained_area.bottom, self.bottom);

        for x in x_base..x_limit {
            for y in y_base..y_limit {
                let offset =
                    ((fb_address as u32) + (y as u32 * 4096) + ((x as u32 * 32) / 8)) as *mut u32;
                unsafe {
                    *offset = colour;
                }
            }
        }
    }

    pub fn paint(&self, colour: u32, fb_address: usize) {
        for y in self.top..self.bottom {
            for x in self.left..self.right {
                let offset =
                    ((fb_address as u32) + (y as u32 * 4096) + ((x as u32 * 32) / 8)) as *mut u32;
                unsafe {
                    *offset = colour;
                }
            }
        }
    }

    /*
        Split a rectangle (which may be a window) by a splitting rectangle
    */
    fn split_rect(rect: &mut Rect, split_rect: &Rect) -> Queue<Rect> {
        let mut splitted_rects: Queue<Rect> = Queue::<Rect>::new();

        // top left, bottom, right

        // Split by left edge
        if (split_rect.left > rect.left && split_rect.left < rect.right) {
            splitted_rects.enqueue(Rect::new(rect.top, rect.bottom, split_rect.left, rect.left));
            rect.left = split_rect.left;
        }

        // Split by top edge
        if (split_rect.top > rect.top && split_rect.top < rect.bottom) {
            splitted_rects.enqueue(Rect::new(rect.top, split_rect.top, rect.right, rect.left));
            rect.top = split_rect.top;
        }

        // Split by right edge
        if (split_rect.right > rect.left && split_rect.right < rect.right) {
            splitted_rects.enqueue(Rect::new(
                rect.top,
                rect.bottom,
                rect.right,
                split_rect.right,
            ));
            rect.right = split_rect.right;
        }

        // Split by bottom edge
        if (split_rect.bottom > rect.top && split_rect.bottom < rect.bottom) {
            splitted_rects.enqueue(Rect::new(
                split_rect.bottom,
                rect.bottom,
                rect.right,
                rect.left,
            ));
            rect.bottom = split_rect.bottom;
        }

        splitted_rects
    }

    /*
        Splits a list of already splitted rectangles by another splitting rectangle
    */
    pub fn split_rects(rects: &mut Queue<Rect>, splitting_rect: &Rect) {
        let mut test_rects = Queue::<Rect>::new();

        for mut i in 0..rects.list.length() {
            let mut working_rect = rects.get_element(i);

            // Check for intersection
            if (!splitting_rect.does_intersect(&working_rect)) {
                test_rects.enqueue(*working_rect);
                continue;
            }

            let mut splitted_rects = Self::split_rect(working_rect, splitting_rect);

            for rect in splitted_rects.list.into_iter() {
                test_rects.enqueue(rect.payload);
            }
        }

        rects.empty();

        for rect in test_rects.list.into_iter() {
            rects.enqueue(rect.payload);
        }
    }
}
