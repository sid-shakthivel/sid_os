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

static mut TEST_ADDRESS: usize = 0;

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
        TEST_ADDRESS = address;
    }

    WM.lock().set_fb_address(address);
    WM.free();

    WM.lock().set_font();
    WM.free();

    WM.lock()
        .add_window(Window::new("Terminal", 100, 100, 300, 300));
    WM.free();

    WM.lock()
        .add_window(Window::new("Paint", 300, 200, 150, 300));
    WM.free();

    WM.lock().paint();
    WM.free();
}

#[repr(C, packed)]
pub struct TgaHeader {
    magic1: u8,    // must be zero
    colormap: u8,  // must be zero
    encoding: u8,  // must be 2
    cmaporig: u16, // must be zero
    cmaplen: u16,  // must be zero
    cmapent: u8,   // must be zero
    x: u16,        // must be zero
    y: u16,        // image's height
    h: u16,        // image's height
    w: u16,        // image's width
    bpp: u8,       // must be 32
    pixeltype: u8, // must be 40
}

pub fn display_image(ptr: *const u8, size: usize) {
    let ptr_ref = unsafe { ptr.as_ref().unwrap() };
    let header = unsafe { &*(ptr as *const TgaHeader) };

    let w = header.w as usize;
    let h = header.h as usize;
    let o = header.y as usize;

    print_serial!("size is {}\n", core::mem::size_of::<TgaHeader>());

    let test = header.x;
    let best = header.bpp;
    let hest = header.pixeltype;
    let encoding = header.encoding;

    print_serial!("encoding is {}\n", encoding);
    print_serial!("pixel type {}\n", hest);
    print_serial!("bpp {}\n", best);
    print_serial!("x {}\n", test);

    let mut m = if header.encoding == 2 && header.cmaporig != 0 {
        (header.pixeltype >> 3) as usize * header.cmaplen as usize
    } else {
        0
    } + 18;

    print_serial!("w: {}, h: {}, o: {}, m: {}\n", w, h, o, m);

    let value = unsafe { *ptr.offset(2) };

    print_serial!("value is {}\n", value);

    // assert!(value == 2, "Only supports RBGA ");

    let data = PAGE_FRAME_ALLOCATOR.lock().alloc_page_frames(20) as *mut u32;
    PAGE_FRAME_ALLOCATOR.free();

    unsafe {
        if *ptr.offset(5) != 0
            || *ptr.offset(6) != 0
            || *ptr.offset(1) != 0
            || (*ptr.offset(16) != 24 && *ptr.offset(16) != 32)
        {
            print_serial!("this is a problem");
        }

        let the_value: usize = *ptr.offset(16) as usize >> 3;

        print_serial!("its {}\n", the_value);

        let mut y = 0;
        let mut i = 0;
        for y in 0..h {
            let mut j = ((!o != 0) as usize * (h - y - 1) + (o == 0) as usize * y) * w * the_value;
            for x in 0..w {
                // let color = ((*ptr.offset(16) == 32)
                //     .then(|| *ptr.offset(j as isize + 3))
                //     .unwrap_or(0xFF) as u32)
                //     << 24
                //     | (*ptr.offset(j as isize + 2) as u32) << 16
                //     | (*ptr.offset(j as isize + 1) as u32) << 8
                //     | (*ptr.offset(j as isize) as u32);

                let color = 0xFF << 24
                    | (*ptr.offset(j as isize + 2) as u32) << 16
                    | (*ptr.offset(j as isize + 1) as u32) << 8
                    | (*ptr.offset(j as isize) as u32);

                *data.offset(2 + i) = color;
                i += 1;
                j += the_value;
            }
        }
    }

    let framebuffer_ptr = unsafe { TEST_ADDRESS as *mut u32 };

    print_serial!("fb address {:#X}\n", framebuffer_ptr as u32);

    for x in 0..w {
        for y in (0..h) {
            let test = ((h - y) as u32 * 4096) + ((x as u32 * 32) / 8);

            let color = unsafe { *data.offset((2 + (y * w) + x) as isize) };

            let correct_address = (framebuffer_ptr as u32 + test) as *mut u32;
            unsafe {
                // *correct_address = BACKGROUND_COLOUR;
                *framebuffer_ptr.offset((test / 4) as isize) = color;
            }
        }
    }
}
