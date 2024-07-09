/*
    System calls are used to call a kernel service from userland as certain actions must be done with privilege
    Syscalls can be used for process management, file management, communication, and information maintainence
    They are invoked with software interrupts and the design is inspired by postfix
*/

use core::panic;

use crate::fs::vfs::{Vfs, VFS};
use crate::gfx::window::Window;
use crate::interrupts::InterruptStackFrame;
use crate::memory::page_frame_allocator::PAGE_FRAME_ALLOCATOR;
use crate::utils::{bitwise, string};
use crate::{either, print_serial};

use super::PROCESS_MANAGER;

pub static mut FILE_TABLE_COUNTER: usize = 0;

#[repr(usize)]
enum OpenFlags {
    ReadOnly = 0b00000000,   // 0x00
    WriteOnly = 0b00000001,  // 0x01
    ReadWrite = 0b00000010,  // 0x02
    Create = 0b01000000,     // 0x40
    Exclusive = 0b10000000,  // 0x80
    Truncate = 0b1000000000, // 0x200
    Append = 0b10000000000,  // 0x400
}

pub fn syscall_handler(registers: &InterruptStackFrame) -> i64 {
    let syscall_id = registers.rax;

    print_serial!("New Syscall: Id: {} Rbx: {}\n", syscall_id, registers.rbx);

    // WARNING: lseek should be 8
    return match syscall_id {
        0 => read(registers.rbx, registers.rcx as *mut u8, registers.rdx),
        1 => write(registers.rbx, registers.rcx as *mut u8, registers.rdx),
        2 => open(registers.rbx as *mut u8, registers.rcx),
        3 => close(registers.rbx),
        8 => allocate_pages(registers.rbx),
        9 => lseek(registers.rbx, registers.rcx as isize, registers.rdx),
        19 => free_pages(registers.rbx, registers.rcx),
        56 => exit(),
        350 => getpid(),
        351 => isatty(registers.rbx),
        _ => {
            panic!("Unknown syscall? {}\n", syscall_id);
            return 0;
        }
    };
}

fn read(file: usize, buffer: *mut u8, length: usize) -> i64 {
    if length == 0 {
        return -1;
    }

    if (file == 0) {
        panic!("Error: No implementation for stdin");
    }

    let current_proc = PROCESS_MANAGER.lock().get_current_process();
    PROCESS_MANAGER.free();

    let file = current_proc.fdt.get(file);

    match file {
        Some(file) => {
            // May not work
            VFS.lock().read_file(&file, buffer, length);
            VFS.free();

            return either!(length == file.size => 0; length as i64);
        }
        None => panic!("Error: File not found"),
    }
    0
}

/*
    Writes given length of bytes from buffer to the file specified
    Length must be above 0 and under max value
*/
fn write(file: usize, buffer: *mut u8, length: usize) -> i64 {
    if length == 0 {
        return 0;
    }

    if length > usize::max_value() {
        return -1;
    }

    match file {
        1..=2 => {
            // 1 refers to stdout and writes to the console
            // 2 refers to stderr and writes to the console
            for i in 0..(length) {
                let character = unsafe { *buffer.offset(i as isize) };
                print_serial!("{}", character as char);
            }
        }
        _ => {
            let current_proc = PROCESS_MANAGER.lock().get_current_process();
            PROCESS_MANAGER.free();

            let file = current_proc.fdt.get(file);

            match file {
                // Need references rather then files need to fix
                Some(mut file) => {
                    VFS.lock().write_file(&mut file, buffer, length);
                    VFS.free();

                    return either!(length == file.size => 0; length as i64);
                }
                None => panic!("Error: File not found"),
            }
        }
    }

    length as i64
}

fn open(file: *const u8, flags: usize) -> i64 {
    let filepath = string::get_string_from_ptr(file);

    if (bitwise::contains_bit(flags as u8, OpenFlags::Create as u8)) {
        panic!("Not implemented")
    }

    let file = VFS.lock().open(filepath);
    VFS.free();

    let current_proc = PROCESS_MANAGER.lock().get_current_process();
    PROCESS_MANAGER.free();

    unsafe {
        FILE_TABLE_COUNTER += 1;
        current_proc.fdt.set(FILE_TABLE_COUNTER, file);
        return FILE_TABLE_COUNTER as i64;
    }
}

fn close(file: usize) -> i64 {
    let current_proc = PROCESS_MANAGER.lock().get_current_process();
    PROCESS_MANAGER.free();

    current_proc.fdt.delete(file);
    0
}

fn lseek(file: usize, offset: isize, whence: usize) -> i64 {
    0
}

fn exit() -> i64 {
    PROCESS_MANAGER.lock().remove_process();
    PROCESS_MANAGER.free();

    0
}

fn isatty(file: usize) -> i64 {
    if file == 0 || file == 1 || file == 2 {
        return 1;
    }
    return -1;
}

fn getpid() -> i64 {
    let pid = PROCESS_MANAGER.lock().get_current_process().pid as i64;
    PROCESS_MANAGER.free();
    pid
}

fn allocate_pages(pages_required: usize) -> i64 {
    let address = PAGE_FRAME_ALLOCATOR
        .lock()
        .alloc_page_frames(pages_required);
    PAGE_FRAME_ALLOCATOR.free();
    address as i64
}

fn free_pages(memory_address: usize, pages_required: usize) -> i64 {
    PAGE_FRAME_ALLOCATOR
        .lock()
        .free_page_frames(memory_address as *mut usize, pages_required);
    PAGE_FRAME_ALLOCATOR.free();
    0
}
