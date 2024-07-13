use crate::{
    ds::{list::List, queue::Queue},
    print_serial,
};

use super::{psf::Font, BPP, PITCH};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rect {
    pub top: u16,
    pub bottom: u16,
    pub left: u16,
    pub right: u16,
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

    pub fn coord_does_intersect(&self, coords: (u16, u16)) -> bool {
        let (x, y) = coords;

        return x > self.left && x < self.right && y > self.top && y < self.bottom;
    }

    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let intersect_left = self.left.max(other.left);
        let intersect_right = self.right.min(other.right);
        let intersect_top = self.top.max(other.top);
        let intersect_bottom = self.bottom.min(other.bottom);

        // Check if there is an overlap
        if intersect_left < intersect_right && intersect_top < intersect_bottom {
            Some(Rect::new(
                intersect_top,
                intersect_bottom,
                intersect_right,
                intersect_left,
            ))
        } else {
            None
        }
    }

    pub fn paint_text(
        &self,
        text: &'static str,
        mut base_x: u16,
        base_y: u16,
        font: &Font,
        fb_addr: usize,
        colour: u32,
    ) {
        for byte in text.as_bytes() {
            self.draw_clipped_character(*byte as char, base_x, base_y, font, fb_addr, colour);
            base_x += 8;
        }
    }

    fn draw_clipped_character(
        &self,
        character: char,
        x: u16,
        y: u16,
        font: &Font,
        fb_addr: usize,
        colour: u32,
    ) {
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
                        *fb_offset = colour;
                    }
                }
                index -= 1;
            }
        }
    }

    pub fn paint_against_region(&self, region: &Rect, colour: u32, fb_addr: usize) {
        // Clamp writeable area to both the clipped region and the contrained area in which it should be
        let x_base = core::cmp::max(region.left, self.left);
        let y_base = core::cmp::max(region.top, self.top);
        let x_limit = core::cmp::min(region.right, self.right);
        let y_limit = core::cmp::min(region.bottom, self.bottom);

        let clamped_rect = Rect::new(y_base, y_limit, x_limit, x_base);

        clamped_rect.paint(colour, fb_addr);
    }

    pub fn paint(&self, colour: u32, fb_addr: usize) {
        for y in self.top..self.bottom {
            for x in self.left..self.right {
                let offset =
                    ((fb_addr as u32) + (y as u32 * PITCH) + ((x as u32 * BPP) / 8)) as *mut u32;
                unsafe {
                    *offset = colour;
                }
            }
        }
    }

    // Split rectangle against list of rectangles
    pub fn split_rect_list(splitting_rect: &mut Rect, rects: &mut Queue<Rect>) {
        // Loop through the clipping rects
        let mut splitted_rects = Queue::<Rect>::new();

        for mut i in 0..rects.list.length() {
            let mut working_rect = rects.get_element(i);

            // Check for intersection
            if (!splitting_rect.does_intersect(&working_rect)) {
                splitted_rects.enqueue(*working_rect);
                continue;
            }

            Self::split_rect(working_rect, splitting_rect, &mut splitted_rects);
        }

        rects.empty();

        for rect in splitted_rects.list.into_iter() {
            rects.enqueue(rect.payload);
        }
    }

    // Split rectangle against another rectangle
    pub fn split_rect(rect: &mut Rect, split_rect: &Rect, rects: &mut Queue<Rect>) {
        // top left, bottom, right

        // Split by left edge
        if (split_rect.left > rect.left && split_rect.left < rect.right) {
            rects.enqueue(Rect::new(rect.top, rect.bottom, split_rect.left, rect.left));
            rect.left = split_rect.left;
        }

        // Split by top edge
        if (split_rect.top > rect.top && split_rect.top < rect.bottom) {
            rects.enqueue(Rect::new(rect.top, split_rect.top, rect.right, rect.left));
            rect.top = split_rect.top;
        }

        // Split by right edge
        if (split_rect.right > rect.left && split_rect.right < rect.right) {
            rects.enqueue(Rect::new(
                rect.top,
                rect.bottom,
                rect.right,
                split_rect.right,
            ));
            rect.right = split_rect.right;
        }

        // Split by bottom edge
        if (split_rect.bottom > rect.top && split_rect.bottom < rect.bottom) {
            rects.enqueue(Rect::new(
                split_rect.bottom,
                rect.bottom,
                rect.right,
                rect.left,
            ));
            rect.bottom = split_rect.bottom;
        }
    }
}
