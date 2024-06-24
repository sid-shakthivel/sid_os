#![no_std] // Don't link with Rust standard library
#![feature(const_option)]
#![feature(core_intrinsics)]
#![feature(naked_functions)]
#![allow(warnings)]
#![feature(asm_const)]
#![feature(exclusive_range_pattern)]
#![feature(const_mut_refs)]
#![feature(ptr_metadata)]

mod dev;
mod ds;
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
use crate::utils::{grub, multiboot2_test};

extern crate multiboot2;
use multiboot2::load;

use core::arch::asm;
use core::mem;
use core::panic::PanicInfo;
use core::prelude::v1;

use spin::Lazy;
use spinning_top::Spinlock;

use ps2_mouse::{Mouse, MouseState};
pub static MOUSE2: Lazy<Spinlock<Mouse>> = Lazy::new(|| Spinlock::new(Mouse::new()));

extern "C" {
    static __kernel_end: u8;
    static __kernel_start: u8;
}

fn init_mouse() {
    MOUSE2.lock().init().unwrap();
    MOUSE2.lock().set_on_complete(on_complete);
}

fn on_complete(mouse_state: MouseState) {
    use crate::gfx::wm::WM;

    WM.lock().handle_mouse_event(
        (mouse_state.get_x(), mouse_state.get_y() * -1),
        mouse_state.left_button_down(),
    );
    WM.free();
}

#[no_mangle]
pub extern "C" fn rust_main(multiboot_info_addr: usize, magic: usize) {
    interrupts::disable();

    use multiboot2::{BootInformation, MemoryArea};

    let boot_info = unsafe { load(multiboot_info_addr as usize).unwrap() };

    print_serial!(
        "Start Address of multiboot header is {:#X}\n",
        boot_info.start_address()
    );

    print_serial!(
        "End Address of multiboot header is {:#X}\n",
        boot_info.end_address()
    );

    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    for (i, entry) in memory_map_tag.memory_areas().enumerate() {
        print_serial!(
            "Memory Map Entry {}: Start Address: {:#X}, End Address: {:#X}\n",
            i,
            entry.start_address(),
            entry.end_address(),
        );
    }

    for module in boot_info.module_tags() {
        print_serial!(
            "MODULE ADDRESS = 0x{:x}, SIZE =  0x{:x} END ADDRESS = 0x{:x}\n",
            module.start_address(),
            module.module_size(),
            module.end_address(),
        );
    }

    // let mut new_screen = Screen::new();
    // new_screen.clear();

    // CONSOLE.lock().init();
    // CONSOLE.free();

    // memory::gdt::init();
    // // dev::init();

    // init_mouse();

    print_serial!("CUSTOM ONE NOW:\n");

    unsafe {
        let kernel_start_address = &__kernel_start as *const u8 as usize;
        print_serial!("Kernel start address: 0x{:x}\n", kernel_start_address);

        let kernel_end_address = &__kernel_end as *const u8 as usize;
        print_serial!("Kernel end address: 0x{:x}\n", kernel_end_address);
    }

    let multiboot_info = multiboot2_test::load(multiboot_info_addr, magic);

    // print_serial!(
    //     "Custom multiboot start address 0x{:x}\n",
    //     multiboot_info.start_address()
    // );

    print_serial!(
        "Custom multiboot end address 0x{:x}\n",
        multiboot_info.end_address()
    );

    grub::get_end_of_memory(multiboot_info);

    for tag in multiboot_info.get_module_tags() {
        // All modules are programs (so far)

        let module_addr = tag.mod_start as usize;

        print_serial!(
            "This is the module start address 0x{:x} and this is the end address 0x{:x}\n",
            tag.mod_start,
            tag.mod_end,
        );
    }

    // PAGE_FRAME_ALLOCATOR.lock().init(
    //     multiboot_info.end_address(),
    //     grub::get_end_of_memory(multiboot_info),
    // );
    // PAGE_FRAME_ALLOCATOR.free();

    // grub::bga_set_video_mode();
    // gfx::init(multiboot_info.get_framebuffer_tag().expect("Expected FB"));

    // grub::initalise_userland(multiboot_info);

    // interrupts::init();

    // interrupts::pit::PIT.lock().init();
    // interrupts::pit::PIT.free();

    // interrupts::pic::PICS.lock().init();
    // interrupts::pic::PICS.free();

    // interrupts::enable();

    print_serial!("Finished Execution\n");

    loop {}
}

#[panic_handler] // This function is called on panic.
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    print_serial!("Error: {}", info);
    loop {}
}
