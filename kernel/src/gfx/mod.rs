mod framebuffer;

use crate::memory::allocator::kmalloc;
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

    let address = kmalloc(size_in_bytes) as usize;

    print_serial!("0x{:x}\n", address);

    // Map the address to video memory
    paging::map_pages(number_of_pages, fb_tag.addr as usize, address);

    for y in 0..fb_tag.height {
        for x in 0..fb_tag.width {
            let offset = ((address as u32) + (y * 4096) + ((x * 32) / 8)) as *mut u32;
            unsafe {
                *offset = 0x3499fe;
            }
        }
    }
}
