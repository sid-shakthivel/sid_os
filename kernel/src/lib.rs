#![no_std] // Don't link with Rust standard library
#![allow(warnings)]
#![feature(core_intrinsics)]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(exclusive_range_pattern)]
#![feature(const_mut_refs)]

mod dev;
mod ds;
mod fs;
mod gfx;
mod interrupts;
mod memory;
mod multitask;
mod output;
mod utils;

use crate::dev::mouse;
use crate::gfx::init;
use crate::memory::allocator::{kfree, kmalloc, print_memory_list};
use crate::memory::page_frame_allocator::PAGE_FRAME_ALLOCATOR;
use crate::memory::paging;
use crate::multitask::PROCESS_MANAGER;
use crate::output::output::Output;
use crate::output::uart::CONSOLE;
use crate::output::vga_text::Screen;
use crate::utils::{grub, multiboot2};

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

    PAGE_FRAME_ALLOCATOR.lock().init(&multiboot_info);
    PAGE_FRAME_ALLOCATOR.free();

    PROCESS_MANAGER.lock().init();
    PROCESS_MANAGER.free();

    interrupts::pit::PIT.lock().init();
    interrupts::pit::PIT.free();

    dev::init();

    interrupts::init();

    interrupts::pic::PICS.lock().init();
    interrupts::pic::PICS.free();

    // grub::bga_set_video_mode();
    // gfx::init(multiboot_info.get_framebuffer_tag().expect("Expected FB"));

    utils::rtc::read_rtc();

    // grub::initalise_userland(multiboot_info);

    let current_datetime = utils::rtc::get_current_datetime();
    current_datetime.print();

    interrupts::enable();

    loop {}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    print_serial!("Error: {}", info);
    loop {}
}
