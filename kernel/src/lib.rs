#![no_std] // Don't link with Rust standard library
#![feature(const_option)]
#![feature(core_intrinsics)]
#![feature(naked_functions)]
#![allow(warnings)]
#![feature(asm_const)]
#![feature(exclusive_range_pattern)]
#![feature(const_mut_refs)]
#![feature(ptr_metadata)]

mod ds;
mod interrupts;
mod memory;
mod multitask;
mod output;
mod utils;

use crate::memory::page_frame_allocator::PAGE_FRAME_ALLOCATOR;
use crate::multitask::PROCESS_MANAGER;
use crate::output::output::Output;
use crate::output::uart::CONSOLE;
use crate::output::vga_text::Screen;
use crate::utils::multiboot2;

use core::arch::asm;
use core::mem;
use core::panic::PanicInfo;
use core::prelude::v1;

#[no_mangle]
pub extern "C" fn rust_main(multiboot_info_addr: usize, magic: usize) {
    interrupts::disable();

    let mut new_screen = Screen::new();
    new_screen.clear();

    CONSOLE.lock().init();
    CONSOLE.free();

    memory::gdt::init();

    let multiboot_info = multiboot2::load(multiboot_info_addr, magic);

    let mut end_memory: usize = 0;
    let mmap_tag = multiboot_info.get_memory_map_tag().expect("Expected mmap");
    for tag in mmap_tag.get_available_mmap_entries() {
        end_memory = unsafe { (*tag).end_address() };
    }

    PAGE_FRAME_ALLOCATOR
        .lock()
        .init(multiboot_info.end_address(), end_memory);
    PAGE_FRAME_ALLOCATOR.free();

    for tag in multiboot_info.get_module_tags() {
        // All modules are programs (so far)
        unsafe {
            let module_addr = (*tag).mod_start as usize;
            let module_len = (*tag).mod_start as usize;

            PROCESS_MANAGER.lock().add_process(
                multitask::ProcessPriority::High,
                0,
                (module_addr, module_len),
            );
            PROCESS_MANAGER.free();
        }
    }

    interrupts::pit::PIT.lock().init();
    interrupts::pit::PIT.free();

    interrupts::pic::PICS.lock().init();
    interrupts::pic::PICS.free();

    interrupts::init();

    interrupts::enable();

    loop {}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    print_serial!("Error: {}", info);
    loop {}
}
