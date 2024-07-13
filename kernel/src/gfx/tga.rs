/*
    Format of TGA files (from Gimp):
    - Do not use RLE compression
    - Top left
*/

use super::FB_ADDR;
use crate::{memory::allocator::kmalloc, print_serial};

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

pub fn display_image(tga_ptr: *const u8, size: usize) {
    let ptr_ref = unsafe { &*tga_ptr };
    let header = unsafe { &*(tga_ptr as *const TgaHeader) };

    let w = header.w as usize;
    let h = header.h as usize;
    let o = header.y as usize;

    let mut m = if header.encoding == 2 && header.cmaporig != 0 {
        (header.pixeltype >> 3) as usize * header.cmaplen as usize
    } else {
        0
    } + 18;

    // let value = unsafe { *ptr.offset(2) };
    assert!(header.encoding == 2, "Only supports RBGA ");

    // TODO: Test this
    let data = kmalloc(size) as *mut u32;

    unsafe {
        if *tga_ptr.offset(5) != 0
            || *tga_ptr.offset(6) != 0
            || *tga_ptr.offset(1) != 0
            || (*tga_ptr.offset(16) != 24 && *tga_ptr.offset(16) != 32)
        {
            panic!("Error: Only supports 24 or 32 bit images\n");
        }

        let the_value: usize = *tga_ptr.offset(16) as usize >> 3;

        let mut y = 0;
        let mut i = 0;
        for y in 0..h {
            let mut j = ((!o != 0) as usize * (h - y - 1) + (o == 0) as usize * y) * w * the_value;
            for x in 0..w {
                let color = 0xFF << 24
                    | (*tga_ptr.offset(j as isize + 2) as u32) << 16
                    | (*tga_ptr.offset(j as isize + 1) as u32) << 8
                    | (*tga_ptr.offset(j as isize) as u32);

                *data.offset(2 + i) = color;
                i += 1;
                j += the_value;
            }
        }
    }

    let framebuffer_ptr = unsafe { FB_ADDR as *mut u32 };

    for x in 0..w {
        for y in (0..h) {
            let adjusted_y = ((h - y) as u32 * 4096) + ((x as u32 * 32) / 8);

            let color = unsafe { *data.offset((2 + (y * w) + x) as isize) };

            let correct_address = (framebuffer_ptr as u32 + adjusted_y) as *mut u32;

            unsafe {
                *framebuffer_ptr.offset((adjusted_y / 4) as isize) = color;
            }
        }
    }
}
