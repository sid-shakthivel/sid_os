/*
    System calls are used to call a kernel service from userland as certain actions must be done with privilege
    Syscalls can be used for process management, file management, communication, and information maintainence
    They are invoked with software interrupts and the design is inspired by postfix
*/

use core::panic;

use crate::gfx::window::Window;
use crate::interrupts::InterruptStackFrame;
use crate::memory::page_frame_allocator::PAGE_FRAME_ALLOCATOR;
use crate::print_serial;

pub fn syscall_handler(registers: &InterruptStackFrame) -> i64 {
    let syscall_id = registers.rax;

    print_serial!("New Syscall: Id: {} Rbx: {}\n", syscall_id, registers.rbx);

    // WARNING: lseek should be 8
    return match syscall_id {
        0 => read(registers.rbx, registers.rcx as *mut u8, registers.rdx),
        1 => write(registers.rbx, registers.rcx as *mut u8, registers.rdx),
        2 => open(registers.rbx, registers.rcx),
        3 => close(registers.rbx),
        8 => allocate_pages(registers.rbx),
        9 => lseek(registers.rbx, registers.rcx as isize, registers.rdx),
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
        1 => {
            // 1 refers to stdout and writes to the console
            for i in 0..(length) {
                let character = unsafe { *buffer.offset(i as isize) };
                print_serial!("{}", character as char);
            }
        }
        2 => {
            // 2 refers to stderr and writes to the console
            for i in 0..(length) {
                let character = unsafe { *buffer.offset(i as isize) };
                print_serial!("{}", character as char);
            }
        }
        _ => panic!("Not implemented"),
    }

    length as i64
}

fn open(file: usize, flags: usize) -> i64 {
    0
}

fn close(file: usize) -> i64 {
    0
}

fn lseek(file: usize, offset: isize, whence: usize) -> i64 {
    0
}

fn exit() -> i64 {
    0
}

fn isatty(file: usize) -> i64 {
    if file == 0 || file == 1 || file == 2 {
        return 1;
    }
    return -1;
}

fn getpid() -> i64 {
    0
}

fn allocate_pages(pages_required: usize) -> i64 {
    let address = PAGE_FRAME_ALLOCATOR
        .lock()
        .alloc_page_frames(pages_required);
    PAGE_FRAME_ALLOCATOR.free();
    address as i64
}
