mod psf;
mod rect;
pub mod tga;
pub mod window;
pub mod wm;

use window::Window;
use wm::WM;

use crate::memory::allocator::{kmalloc, print_memory_list};
use crate::memory::page_frame_allocator::PAGE_FRAME_ALLOCATOR;
use crate::memory::{page_frame_allocator, paging};
use crate::multiboot2;
use crate::{print_serial, CONSOLE};

const SCREEN_WIDTH: u16 = 1024;
const SCREEN_HEIGHT: u16 = 768;

const PITCH: u32 = 4096;
const BPP: u32 = 32;

const BACKGROUND_COLOUR: u32 = 0x3499fe;

static mut FB_ADDR: usize = 0;

pub fn init(fb_tag: &multiboot2::FramebufferTag) {
    // Ensure the fb is of RBG
    assert!(fb_tag.fb_type == 1, "FB is not of type RBG");

    // Setup the front buffer
    let size_in_bytes =
        ((fb_tag.bpp as usize) * (fb_tag.width as usize) * (fb_tag.height as usize)) / 8;

    let size_in_mib = size_in_bytes / 1024 / 1024;
    let number_of_pages = page_frame_allocator::get_number_of_pages(size_in_bytes);

    assert!(size_in_mib == 3, "FB is not of expected size");

    // let address = kmalloc(size_in_bytes) as usize;

    let address = PAGE_FRAME_ALLOCATOR
        .lock()
        .alloc_page_frames(number_of_pages) as usize;
    PAGE_FRAME_ALLOCATOR.free();

    // Map the address to video memory
    paging::map_pages(number_of_pages, address, fb_tag.addr as usize);

    unsafe {
        FB_ADDR = address;
    }

    WM.lock().set_fb_address(address);
    WM.free();

    WM.lock().set_font();
    WM.free();

    WM.lock()
        .add_window(Window::new("Terminal", 100, 100, 300, 300, 0x5E5E5E));
    WM.free();

    WM.lock()
        .add_window(Window::new("Paint", 300, 200, 150, 300, 0x009E9E));
    WM.free();

    WM.lock().paint();
    WM.free();
}
