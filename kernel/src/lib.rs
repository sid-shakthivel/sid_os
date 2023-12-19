#![no_std] // Don't link with Rust standard library
#![feature(const_option)]
#![feature(core_intrinsics)]
#![feature(naked_functions)]

mod output {
    pub mod output;
    pub mod uart;
    pub mod vga_text;
}

mod memory {
    pub mod gdt;
    pub mod page_frame_allocator;
}

mod utils {
    pub mod multiboot2;
    pub mod ports;
    pub mod spinlock;
}

mod interrupts;

use crate::output::output::Output;
use crate::output::uart::CONSOLE;
use crate::output::vga_text::Screen;
use crate::utils::multiboot2::MultibootInfo;
use core::mem;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn rust_main(multiboot_information_address: usize) {
    let mut new_screen = Screen::new();
    new_screen.clear();

    CONSOLE.lock().init();
    CONSOLE.free();

    interrupts::init();

    // TODO: Fix this because literally adding a page purely for safety
    let multiboot_end = multiboot_information_address + mem::size_of::<MultibootInfo>() + 0x1000;

    print_serial!("Start of multiboot = {:x}\n", multiboot_information_address);
    print_serial!("End of multiboot = {:x}\n", multiboot_end);

    // generate_gdt_values();

    loop {}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
