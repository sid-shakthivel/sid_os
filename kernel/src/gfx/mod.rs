mod rect;
mod window;
mod wm;

use crate::gfx::window::Window;
use crate::gfx::wm::{WindowManager, WM};
use crate::memory::allocator::kmalloc;
use crate::memory::page_frame_allocator::PAGE_FRAME_ALLOCATOR;
use crate::memory::{page_frame_allocator, paging};
use crate::multiboot2;
use crate::{print_serial, CONSOLE};

pub fn init(fb_tag: &multiboot2::FramebufferTag) {
    // Ensure the fb is of RBG
    assert!(fb_tag.fb_type == 1, "FB is not of type RBG");

    // Setup the front buffer
    let size_in_bytes =
        ((fb_tag.bpp as usize) * (fb_tag.width as usize) * (fb_tag.height as usize)) / 8;

    let size_in_mib = size_in_bytes / 1024 / 1024;
    let number_of_pages = page_frame_allocator::get_number_of_pages(size_in_bytes);

    assert!(size_in_mib == 3, "FB is not of expected size");

    // let address = PAGE_FRAME_ALLOCATOR
    //     .lock()
    //     .alloc_page_frames(number_of_pages) as usize;
    // PAGE_FRAME_ALLOCATOR.free();
    let address = kmalloc(size_in_bytes) as usize;
    print_serial!("FB at: 0x{:x}\n", address);

    // Map the address to video memory
    paging::map_pages(number_of_pages, address, fb_tag.addr as usize);

    // for y in 0..fb_tag.height {
    //     for x in 0..fb_tag.width {
    //         let offset = ((address as u32) + (y * 4096) + ((x * 32) / 8)) as *mut u32;
    //         unsafe {
    //             *offset = 0x3499fe;
    //         }
    //     }
    // }

    WM.lock().set_fb_address(address);
    WM.free();

    WM.lock().add_window(Window::new(200, 125, 300, 300));
    WM.free();

    WM.lock().add_window(Window::new(50, 50, 300, 200));
    WM.free();

    WM.lock().add_window(Window::new(25, 25, 100, 100));
    WM.free();

    WM.lock().paint();
    WM.free();
}
