#![no_std] // Don't link with Rust standard library
#![feature(const_option)]

mod multiboot2;
mod output;
mod page_frame_allocator;
mod ports;
mod spinlock;
mod uart;
mod vga_text;

use crate::output::Output;
use core::mem;
use core::panic::PanicInfo;
use multiboot2::{MultibootInfo, MultibootMemoryMap, MultibootModule};
use uart::{Console, CONSOLE};
use vga_text::Screen;

#[no_mangle]
pub extern "C" fn rust_main(multiboot_information_address: usize) {
    let mut new_screen = Screen::new();
    new_screen.clear();

    CONSOLE.lock().init();
    CONSOLE.free();

    let multiboot_end = multiboot_information_address + mem::size_of::<MultibootInfo>() + 0x1000;
    let pointer_check: *const usize = multiboot_end as *const usize;

    print_serial!("Start of multiboot = {:x}\n", multiboot_information_address);
    print_serial!("End of multiboot = {:x}\n", multiboot_end);

    // // Calculate the address of the memory map
    // let memory_map_addr: *const c_void =
    //     (multiboot_info as *const Multiboot2Info).offset(1) as *const c_void;

    // // Iterate through the memory map entries
    // let mut current_entry_addr = memory_map_addr;
    // while (current_entry_addr as usize)
    //     < (memory_map_addr as usize + multiboot_info.total_size as usize
    //         - core::mem::size_of::<Multiboot2Info>())
    // {
    //     let entry: &MemoryMapEntry = unsafe { &*(current_entry_addr as *const MemoryMapEntry) };

    //     // Process the memory map entry as needed
    //     // (You may want to check the entry type to skip certain regions)

    //     // Move to the next memory map entry
    //     current_entry_addr = (current_entry_addr as usize + entry.size as usize) as *const c_void;
    // }

    loop {}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
