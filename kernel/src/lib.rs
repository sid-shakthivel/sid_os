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

    let size = unsafe { *(multiboot_info_addr as *const u32) };
    print_serial!("end address = {:x}\n", size);

    let mut tag = unsafe { (multiboot_info_addr + 8) as *const multiboot2::MultibootTag };

    while unsafe { (*tag).typ } != (0 as u32) {
        let tag_type = unsafe { (*tag).typ };
        let tag_size = unsafe { (*tag).size };

        // print_serial!("Tag {}, Size 0x{:x}\n", tag_type, tag_size);

        // Check if the tag is a module tag
        if (tag_type == 3) {
            let tag_module = unsafe { &*(tag as *const multiboot2::MultibootTagModule) };

            print_serial!(
                "Module start = {:x}, Module end = {:x}\n",
                tag_module.mod_start,
                tag_module.mod_end
            );
        }

        if (tag_type == 6) {
            let tag_mmap = unsafe { &*(tag as *const multiboot2::MultibootTagMmap) };
            let tag_address = tag as usize;

            let mut mmap = ((tag_address + 16) as *const multiboot2::MultibootMmapEntry);

            print_serial!("Printing mmap\n");
            unsafe {
                let end_address = tag_address + (tag_mmap.size as usize);

                while (mmap as usize) < end_address {
                    let addr = ((*mmap).addr) as usize;
                    let len = ((*mmap).len) as usize;
                    let typ = (*mmap).typ as usize;

                    // If 1 then available
                    if (typ == 1) {
                        print_serial!("0x{:x} 0x{:x}\n", addr, len);
                    }

                    mmap = {
                        (mmap as *const u8)
                            .add(mem::size_of::<multiboot2::MultibootMmapEntry>())
                            .cast::<multiboot2::MultibootMmapEntry>()
                    };
                }
            }
        }

        tag = unsafe { (tag as *const u8).add(((unsafe { (*tag).size } + 7) & !7) as usize) }
            as *const multiboot2::MultibootTag;
    }

    print_serial!("CUSTOM\n");

    let multiboot_info = multiboot2::load(multiboot_info_addr, magic);
    print_serial!(
        "Start Address: {:x} End Address: {:x}\n",
        multiboot_info.start_address(),
        multiboot_info.end_address()
    );

    print_serial!("Multibool Module Tags:\n");
    for tag in multiboot_info.get_module_tags() {
        unsafe {
            print_serial!("0x{:x} 0x{:x}\n", (*tag).mod_start, (*tag).mod_end);
        }
    }

    print_serial!("Available memory areas:\n");
    let mmap_tag = multiboot_info.get_memory_map_tag().expect("Expected mmap");
    for tag in mmap_tag.get_available_mmap_entries() {
        unsafe {
            print_serial!("0x{:x} 0x{:x}\n", (*tag).addr, (*tag).addr + (*tag).len);
        }
    }

    // let multiboot_end = multiboot_information_address;
    // PAGE_FRAME_ALLOCATOR.lock().init(multiboot_end, 0x100000000);
    // PAGE_FRAME_ALLOCATOR.free();

    // interrupts::pit::PIT.lock().init();
    // interrupts::pit::PIT.free();

    // interrupts::pic::PICS.lock().init();
    // interrupts::pic::PICS.free();

    // interrupts::init();

    loop {}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    print_serial!("Error: {}", info);
    loop {}
}
