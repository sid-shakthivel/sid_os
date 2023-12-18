#![no_std] // Don't link with Rust standard library
#![feature(const_option)]

mod gdt;
mod multiboot2;
mod output;
mod page_frame_allocator;
mod ports;
mod spinlock;
mod uart;
mod vga_text;

use crate::{gdt::generate_gdt_values, output::Output};
use core::mem;
use core::panic::PanicInfo;
use multiboot2::MultibootInfo;
use uart::CONSOLE;
use vga_text::Screen;

#[no_mangle]
pub extern "C" fn rust_main(multiboot_information_address: usize) {
    let mut new_screen = Screen::new();
    new_screen.clear();

    CONSOLE.lock().init();
    CONSOLE.free();

    // TODO: Fix this because literally adding a page purely for safety
    let multiboot_end = multiboot_information_address + mem::size_of::<MultibootInfo>() + 0x1000;

    print_serial!("Start of multiboot = {:x}\n", multiboot_information_address);
    print_serial!("End of multiboot = {:x}\n", multiboot_end);

    generate_gdt_values();

    loop {}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
