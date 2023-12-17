#![no_std] // Don't link with Rust standard library
#![feature(const_option)]

mod output;
mod ports;
mod uart;
mod vga_text;

use crate::output::Output;
use core::panic::PanicInfo;
use uart::Console;
use vga_text::Screen;

#[no_mangle]
pub extern "C" fn rust_main() {
    let mut new_screen = Screen::new();
    new_screen.clear();
    new_screen.write_string("Hello World!");

    let mut console = Console::new();
    console.write_string("Hello World");

    loop {}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
