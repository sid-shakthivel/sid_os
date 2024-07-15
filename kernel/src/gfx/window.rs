use super::{
    psf::{Font, FONT},
    rect::Rect,
};
use crate::{ds::queue::Queue, memory::allocator::kmalloc, print_serial};

pub const WINDOW_BACKGROUND_COLOUR: u32 = 0xFFBBBBBB;
const WINDOW_BORDER_COLOUR: u32 = 0xFF000000;
const WINDOW_TITLE_COLOUR: u32 = 0x232422;
const WINDOW_TITLE_HEIGHT: u16 = 20;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct SimpleWindow {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    colour: u32,
    pub name: *const u8,
}

#[derive(Clone, Copy, Debug)]
pub struct Window {
    pub title: &'static str,
    pub wid: usize,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub colour: u32,
    buffer_addr: usize,
}

impl Window {
    pub fn from(simple_window: &SimpleWindow, name: &'static str) -> Window {
        let size = simple_window.width as u32 * simple_window.height as u32;
        let bytes_required = (size * 32) / 8;
        let buffer_addr = kmalloc(bytes_required as usize) as usize;

        let width = simple_window.width;
        let height = simple_window.height;

        let size_of_bar = (width * WINDOW_TITLE_HEIGHT) as usize;
        let size_of_main =
            (width as usize * (height as usize - WINDOW_TITLE_HEIGHT as usize)) as usize;

        let mut new_window = Window {
            title: name,
            wid: 0,
            x: simple_window.x,
            y: simple_window.y,
            width: simple_window.width,
            height: simple_window.height,
            colour: simple_window.colour,
            buffer_addr,
        };

        let base_x = ((width / 2) - (name.as_bytes().len() as u16 * 8) / 2);
        let base_y = (WINDOW_TITLE_HEIGHT - 16) / 2;

        new_window.copy_colour_to_buffer(0, WINDOW_TITLE_COLOUR, size_of_bar);
        new_window.copy_colour_to_buffer(size_of_bar, simple_window.colour, size_of_main);
        new_window.copy_string_to_buffer(&name, base_x, base_y, 0xFFFFFF);

        return new_window;
    }

    pub fn new(
        title: &'static str,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        colour: u32,
    ) -> Window {
        let size = width as u32 * height as u32;
        let bytes_required = (size * 32) / 8;
        let buffer_addr = kmalloc(bytes_required as usize) as usize;

        let size_of_bar = (width * WINDOW_TITLE_HEIGHT) as usize;
        let size_of_main =
            (width as usize * (height as usize - WINDOW_TITLE_HEIGHT as usize)) as usize;

        let mut new_window = Window {
            title,
            wid: 0,
            x,
            y,
            width,
            height,
            colour,
            buffer_addr,
        };

        let base_x = ((width / 2) - (title.as_bytes().len() as u16 * 8) / 2);
        let base_y = (WINDOW_TITLE_HEIGHT - 16) / 2;

        new_window.copy_colour_to_buffer(0, WINDOW_TITLE_COLOUR, size_of_bar);
        new_window.copy_colour_to_buffer(size_of_bar, colour, size_of_main);
        new_window.copy_string_to_buffer(&title, base_x, base_y, 0xFFFFFF);

        return new_window;
    }

    fn copy_colour_to_buffer(&mut self, offset: usize, colour: u32, length: usize) {
        unsafe {
            let mut current_addr = unsafe { (self.buffer_addr as *mut u32).add(offset) };
            for _ in 0..length {
                core::ptr::write(current_addr, colour);
                current_addr = current_addr.add(1);
            }
        }
    }

    pub fn copy_buffer_to_buffer(&mut self, buffer_addr: *const u32) {
        // Wont work because of the offset to get to the actual main content bit
        let count = (self.width * (self.height - WINDOW_TITLE_HEIGHT)) as usize;

        unsafe {
            let buffer_offset = buffer_addr.add(self.width as usize * WINDOW_TITLE_HEIGHT as usize);
            core::ptr::copy(buffer_addr, self.buffer_addr as *mut u32, count)
        }
    }

    pub fn copy_string_to_buffer(&mut self, text: &str, mut base_x: u16, base_y: u16, colour: u32) {
        for byte in text.as_bytes() {
            self.copy_character_to_buffer(*byte as char, base_x, base_y, colour);
            base_x += 8;
        }
    }

    fn copy_character_to_buffer(&mut self, character: char, x: u16, y: u16, colour: u32) {
        let buffer_ptr = self.buffer_addr as *mut u32;

        let font = FONT.lock();
        FONT.free();

        let font_metadata = unsafe { &*font.metadata_ptr };
        let glyph_address = (font.start_addr
            + font_metadata.header_size
            + (font_metadata.bytes_per_glyph * (character as u32)))
            as *const u8;

        for (cy, line) in (0..16).zip(unsafe { core::slice::from_raw_parts(glyph_address, 16) }) {
            for cx in 0..8 {
                let adjusted_x = x + cx;
                let adjusted_y = y + cy;

                if (line & (1 << (7 - cx))) != 0 {
                    unsafe {
                        *buffer_ptr.offset((adjusted_y * self.width + adjusted_x) as isize) =
                            colour;
                    }
                }
            }
        }
    }

    pub fn generate_rect(&self) -> Rect {
        Rect::new(self.y, self.y + self.height, self.x + self.width, self.x)
    }

    pub fn paint(&self, dr: &Queue<Rect>, fb_addr: usize) {
        let mut rect = self.generate_rect();

        for rect_node in dr.list.into_iter() {
            let current_rect = rect_node.payload;

            current_rect.paint_against_region(
                &rect,
                self.buffer_addr,
                fb_addr,
                self.x,
                self.y,
                self.width,
            );
        }
    }

    pub fn paint_rect(&self, dr: &Rect, fb_addr: usize) {
        let mut rect = self.generate_rect();

        dr.paint_against_region(&rect, self.buffer_addr, fb_addr, self.x, self.y, self.width);
    }
}
