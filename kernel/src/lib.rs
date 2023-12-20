#![no_std] // Don't link with Rust standard library
#![feature(const_option)]
#![feature(core_intrinsics)]
#![feature(naked_functions)]
#![allow(warnings)]
#![feature(asm_const)]
#![feature(exclusive_range_pattern)]
#![feature(const_mut_refs)]

mod interrupts;
mod memory;
mod multitask;
mod output;
mod utils;
mod ds;

use crate::memory::page_frame_allocator::PAGE_FRAME_ALLOCATOR;
use crate::output::output::Output;
use crate::output::uart::CONSOLE;
use crate::output::vga_text::Screen;
use crate::utils::multiboot2::MultibootInfo;

use core::arch::asm;
use core::mem;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn rust_main(multiboot_information_address: usize) {
    interrupts::disable();

    let mut new_screen = Screen::new();
    new_screen.clear();

    CONSOLE.lock().init();
    CONSOLE.free();

    let multiboot_end = multiboot_information_address + mem::size_of::<MultibootInfo>();
    let memory_end = 0x100000000;
    PAGE_FRAME_ALLOCATOR.lock().init(multiboot_end, memory_end);
    PAGE_FRAME_ALLOCATOR.free();

    interrupts::pit::PIT.lock().init();
    interrupts::pit::PIT.free();

    interrupts::pic::PICS.lock().init();
    interrupts::pic::PICS.free();

    interrupts::init();

    // interrupts::enable();

    memory::paging::map_page(0x5000000, 0x5000000, false);

    // print_serial!("Start of multiboot = {:x}\n", multiboot_information_address);
    // print_serial!("End of multiboot = {:x}\n", multiboot_end);

    print_serial!("Initalised everything!\n");

    loop {}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    print_serial!("Error: {}", info);
    loop {}
}
