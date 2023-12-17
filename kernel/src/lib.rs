// src/lib.rs

#![no_std] // Don't link with Rust standard library
#![feature(const_option)]

mod output;
mod vga_text;

use crate::output::Output;
use core::panic::PanicInfo;
use vga_text::Screen;

#[no_mangle]
pub extern "C" fn rust_main() {
    // let vga_buffer = 0xb8000 as *mut u8;

    // for (i, &byte) in HELLO.iter().enumerate() {
    //     unsafe {
    //         *vga_buffer.offset(i as isize * 2) = byte;
    //         *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
    //     }
    // }

    let mut new_screen = Screen::new();
    new_screen.clear();
    new_screen.write_string("Hello World!");

    loop {}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
