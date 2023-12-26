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
use crate::utils::{grub, multiboot2};

use core::arch::asm;
use core::mem;
use core::panic::PanicInfo;
use core::prelude::v1;

use spin::Lazy;
use spinning_top::Spinlock;

use ps2_mouse::{Mouse, MouseState};
pub static MOUSE2: Lazy<Spinlock<Mouse>> = Lazy::new(|| Spinlock::new(Mouse::new()));

fn init_mouse() {
    MOUSE2.lock().init().unwrap();
    MOUSE2.lock().set_on_complete(on_complete);
}

fn on_complete(mouse_state: MouseState) {
    use crate::gfx::wm::WM;

    WM.lock().handle_mouse_event(
        (mouse_state.get_y() * -1, mouse_state.get_x()),
        mouse_state.left_button_down(),
    );
    WM.free();
}

#[no_mangle]
pub extern "C" fn rust_main(multiboot_info_addr: usize, magic: usize) {
    interrupts::disable();

    let mut new_screen = Screen::new();
    new_screen.clear();

    CONSOLE.lock().init();
    CONSOLE.free();

    memory::gdt::init();
    // dev::init();

    init_mouse();

    let multiboot_info = multiboot2::load(multiboot_info_addr, magic);

    let mut end_memory: usize = 0;
    let mmap_tag = multiboot_info.get_memory_map_tag().expect("Expected mmap");
    for tag in mmap_tag.get_available_mmap_entries() {
        end_memory = tag.end_address()
    }

    PAGE_FRAME_ALLOCATOR
        .lock()
        .init(multiboot_info.end_address(), end_memory);
    PAGE_FRAME_ALLOCATOR.free();

    // grub::bga_set_video_mode();
    // gfx::init(multiboot_info.get_framebuffer_tag().expect("Expected FB"));

    // for tag in multiboot_info.get_module_tags() {
    //     // All modules are programs (so far)
    //     let module_addr = tag.mod_start as usize;
    //     let module_len = (tag.mod_end - tag.mod_start) as usize;

    //     PROCESS_MANAGER.lock().add_process(
    //         multitask::ProcessPriority::High,
    //         0,
    //         (module_addr, module_len),
    //     );
    //     PROCESS_MANAGER.free();
    // }

    let test = kmalloc(4);
    print_serial!("The kmalloc'd address is 0x{:x}\n", test as usize);
    kfree(test);
    print_memory_list();
    let best = kmalloc(4);
    print_serial!("The kmalloc'd address is 0x{:x}\n", best as usize);
    print_memory_list();

    interrupts::init();

    interrupts::pit::PIT.lock().init();
    interrupts::pit::PIT.free();

    interrupts::pic::PICS.lock().init();
    interrupts::pic::PICS.free();

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
