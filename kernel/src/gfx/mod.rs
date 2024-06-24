mod psf;
mod rect;
pub mod window;
pub mod wm;

use crate::gfx::window::Window;
use crate::gfx::wm::{WindowManager, WM};
use crate::memory::allocator::{kmalloc, print_memory_list};
use crate::memory::page_frame_allocator::PAGE_FRAME_ALLOCATOR;
use crate::memory::{page_frame_allocator, paging};
use crate::multiboot2;
use crate::{print_serial, CONSOLE};

pub fn init(fb_tag: &multiboot2::FramebufferTag) {
    // Ensure the fb is of RBG
    // assert!(fb_tag.fb_type == 1, "FB is not of type RBG");

    // // Setup the front buffer
    // let size_in_bytes =
    //     ((fb_tag.bpp as usize) * (fb_tag.width as usize) * (fb_tag.height as usize)) / 8;

    // let size_in_mib = size_in_bytes / 1024 / 1024;
    // let number_of_pages = page_frame_allocator::get_number_of_pages(size_in_bytes);

    // assert!(size_in_mib == 3, "FB is not of expected size");

    // let address = kmalloc(size_in_bytes) as usize;

    // // Map the address to video memory
    // paging::map_pages(number_of_pages, address, fb_tag.addr as usize);

    // WM.lock().set_fb_address(address);
    // WM.free();

    // WM.lock().set_font();
    // WM.free();

    // WM.lock()
    //     .add_window(Window::new("Terminal", 100, 100, 300, 300));
    // WM.free();

    // // WM.lock().add_window(Window::new("Paint", 50, 50, 300, 200));
    // // WM.free();

    // // WM.lock()
    // //     .add_window(Window::new("Text-edit", 500, 400, 400, 300));
    // // WM.free();

    // WM.lock().paint();
    // WM.free();

    // WM.lock().handle_mouse_event((1, 1), true);
    // WM.free();
}
