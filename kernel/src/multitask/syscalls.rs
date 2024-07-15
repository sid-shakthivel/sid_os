/*
    System calls are used to call a kernel service from userland as certain actions must be done with privilege
    Syscalls can be used for process management, file management, communication, and information maintainence
    They are invoked with software interrupts and the design is inspired by postfix
*/

use core::panic;

use crate::fs::vfs::{Vfs, VFS};
use crate::gfx::window::{self, SimpleWindow, Window};
use crate::gfx::wm::WM;
use crate::gfx::FB_ADDR;
use crate::interrupts::{InterruptStackFrame, SyscallStackFrame};
use crate::memory::allocator::kmalloc;
use crate::memory::page_frame_allocator::PAGE_FRAME_ALLOCATOR;
use crate::utils::event::EVENT_MANAGER;
use crate::utils::{bitwise, string};
use crate::{either, print_serial};

use super::process::Message;
use super::PROCESS_MANAGER;

pub static mut FILE_TABLE_COUNTER: usize = 0;

#[repr(usize)]
enum MemoryProtectionAttributes {
    None = 0x00,
    Read = 0x01,
    Write = 0x02,
}

#[repr(usize)]
enum MemoryMappingFlags {
    MapPrivate = 0x02,
    MapAnonymous = 0x20,
}

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

#[repr(C)]
pub struct Iovec {
    pub base: *mut u8,
    pub len: usize,
}

pub fn syscall_handler(registers: &SyscallStackFrame) -> i64 {
    let syscall_id = registers.rax;

    // print_serial!("syscall id: {}\n", syscall_id);
    // print_serial!("id: {} registers: {:?}\n", syscall_id, registers);

    // This set of syscalls are for the current implementation of newlib
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
        352 => send_message(registers.rbx as *mut Message),
        353 => receive_message(),
        354 => create_window(registers.rbx as *mut SimpleWindow),
        355 => get_event(),
        356 => paint_string(
            registers.rbx as *mut u8,
            registers.rcx,
            registers.rsi,
            registers.rdi,
        ),
        357 => copy_to_win_buffer(registers.rbx, registers.rcx as *const u32),
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

    assert!(file == 0, "Error: No implementation for stdin");

    let current_proc = PROCESS_MANAGER.lock().get_current_process();
    PROCESS_MANAGER.free();

    let file = current_proc.fdt.get(file);

    match file {
        Some(file) => {
            let file_ref = unsafe { &(*file) };
            VFS.lock()
                .read_file(file_ref, buffer, length, file_ref.get_offset());
            VFS.free();

            return either!(length == file_ref.size => 0; length as i64);
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
                    let file_mut_ref = unsafe { &mut (*file) };
                    VFS.lock()
                        .write_file(file_mut_ref, buffer, length, file_mut_ref.get_offset());
                    VFS.free();

                    return either!(length == file_mut_ref.size => 0; length as i64);
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

    let file = VFS.lock().open_addr(filepath);
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

fn lseek(fd: usize, new_offset: isize, whence: usize) -> i64 {
    assert!(
        fd == 0 || fd == 1 || fd == 2,
        "Error: Invalid file for lseek"
    );

    assert!(new_offset > 0, "Error: Offset must be above 0");

    let current_proc = PROCESS_MANAGER.lock().get_current_process();
    PROCESS_MANAGER.free();

    let file = current_proc.fdt.get(fd);

    match file {
        // Need references rather then files need to fix
        Some(mut file) => {
            let file_mut_ref = unsafe { &mut (*file) };

            let new_offset = match whence {
                0 => new_offset,
                1 => file_mut_ref.get_offset() as isize + new_offset,
                2 => file_mut_ref.size as isize + new_offset,
                _ => panic!("Error: Invalid whence"),
            };

            file_mut_ref.set_offset(new_offset as usize);

            return file_mut_ref.get_offset() as i64;
        }
        None => panic!("Error: File not found"),
    }

    new_offset as i64
}

fn mmap(addr: usize, length: usize, prot: usize, flags: usize, fd: i32, offset: usize) -> i64 {
    assert!(addr == 0, "Error: Cannot set addr");
    assert!(fd == -1, "Error: Cannot currently set fd");

    assert!(
        prot == (MemoryProtectionAttributes::Read as usize)
            || prot
                == (MemoryProtectionAttributes::Read as usize
                    | MemoryProtectionAttributes::Write as usize)
            || prot == (MemoryProtectionAttributes::None as usize)
    );

    assert!(
        flags
            == (MemoryMappingFlags::MapAnonymous as usize
                | MemoryMappingFlags::MapPrivate as usize)
    );

    print_serial!("{} {} {} {} {} {}\n", addr, length, prot, flags, fd, offset);

    let adddr = kmalloc(length);

    return adddr as i64;
}

fn brk(addr: usize) -> i64 {
    -1
}

fn ioctl(cmd: usize, arg: usize) -> i64 {
    0
}

fn writev(fd: usize, iovec: *const Iovec, count: usize) -> i64 {
    // print_serial!("fd: {}\n", fd);

    assert!(fd == 1 || fd == 2, "Error: Writev not for stdout");

    // print_serial!("{} {}\n", fd, count);

    let mut total_length = 0;

    for i in 0..count {
        let iov = unsafe { &*iovec.offset(i as isize) };
        let buffer = iov.base as *mut u8;
        let length = iov.len;
        total_length += length;

        for j in 0..(length) {
            let character = unsafe { *buffer.offset(j as isize) };
            print_serial!("{}", character as char);
        }
    }

    total_length as i64
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
    print_serial!("allocating {}\n", pages_required);

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

fn send_message(message: *mut Message) -> i64 {
    PROCESS_MANAGER.lock().send_message(message);
    PROCESS_MANAGER.free();

    1
}

fn receive_message() -> i64 {
    let message = PROCESS_MANAGER.lock().receive_message();
    PROCESS_MANAGER.free();

    if let Some(message_addr) = message {
        return message_addr as i64;
    }

    -1
}

fn create_window(new_window: *mut SimpleWindow) -> i64 {
    let window_properties = unsafe { &mut *new_window };

    // let new_window_name = "terminal";

    let mut new_window_name = string::get_string_from_ptr(window_properties.name);

    let new_window = Window::from(&window_properties, &new_window_name);

    WM.lock().add_window(new_window);
    WM.free();

    WM.lock().paint();
    WM.free();

    new_window.wid as i64
}

fn get_event() -> i64 {
    let event = EVENT_MANAGER.lock().get_event();
    EVENT_MANAGER.free();

    event as i64
}

fn paint_string(ptr: *mut u8, wid: usize, x: usize, y: usize) -> i64 {
    let string = string::get_string_from_ptr(ptr);

    let window = WM.lock().find_get_mut(wid);
    WM.free();

    let rect = window.generate_rect();
    window.copy_string_to_buffer(string, x as u16, y as u16, 0x00);

    unsafe {
        let fb_addr = FB_ADDR;
        rect.paint_text(string, x as u16, y as u16, fb_addr, 0x00);
    }

    1
}

fn copy_to_win_buffer(wid: usize, buffer: *const u32) -> i64 {
    let window = WM.lock().find_get_mut(wid);
    WM.free();
    window.copy_buffer_to_buffer(buffer);
    1
}

/*
Underneath is the current implementation of syscalls for musl which unfortunately does not work
return match syscall_id {
    0 => read(registers.rdi, registers.rsi as *mut u8, registers.rdx),
    1 => write(registers.rdi, registers.rsi as *mut u8, registers.rdx),
    2 => open(registers.rdi as *mut u8, registers.rsi),
    3 => close(registers.rdi),
    8 => lseek(registers.rdi, registers.rsi as isize, registers.rdx),
    9 => mmap(
        registers.rdi,
        registers.rsi,
        registers.rdx,
        registers.r10,
        registers.r8 as i32,
        registers.r9,
    ),
    12 => brk(registers.rdi),
    16 => ioctl(registers.rdi, registers.rsi),
    20 => writev(registers.rdi, registers.rsi as *const Iovec, registers.rdx),
    56 => exit(),
    350 => getpid(),
    351 => isatty(registers.rdi),
    _ => {
        panic!("Unknown syscall? {}\n", syscall_id);
        return 0;
    }
};
*/
