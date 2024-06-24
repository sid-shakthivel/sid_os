// Define all base isr's

use super::{
    exception_handler, exception_with_error_handler, interrupt_handler, pit_handler,
    test_syscall_handler,
};
use core::arch::asm;

// Purely for exceptions with an error code eg page faults
#[macro_export]
macro_rules! setup_exception_with_e_handler {
    ($exception_num: expr) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!(
                    "push rax",
                    "push rbx",
                    "push rcx",
                    "push rdx",
                    "push rbp",
                    "push rdi",
                    "push rsi",
                    "mov rdx, [rsp + 7*8]", // Load error code
                    "mov rdi, rsp", // Load ExceptionStackFrame
                    "mov rsi, {0}", // Load exception id
                    "add rdi, 7*8",
                    "sub rsp, 8", // Allign stack pointer
                    "cld",
                    "call {1}",
                    "add rsp, 8", // Reallign stack pointer
                    "pop rsi",
                    "pop rdi",
                    "pop rbp",
                    "pop rdx",
                    "pop rcx",
                    "pop rbx",
                    "pop rax",
                    "iretq",
                    const $exception_num,
                    sym exception_with_error_handler,
                    options(noreturn)
                );
            }
        }
        wrapper
    }};
}

// Includes exceptions and general interrupts
#[macro_export]
macro_rules! setup_interrupt_handler {
    ($func_name: ident, $interrupt_num: expr) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!(
                    "push rax",
                    "push rbx",
                    "push rcx",
                    "push rdx",
                    "push rbp",
                    "push rdi",
                    "push rsi",
                    "mov rdi, rsp",
                    "mov rsi, {0}",
                    "cld",
                    "call {1}",
                    "pop rsi",
                    "pop rdi",
                    "pop rbp",
                    "pop rdx",
                    "pop rcx",
                    "pop rbx",
                    "pop rax",
                    "iretq",
                    const $interrupt_num,
                    sym $func_name,
                    options(noreturn)
                );
            }
        }
        wrapper
    }};
}

// Must save data in rax eg mov rsp, rax
#[naked]
pub extern "C" fn setup_pit_handler() -> ! {
    // HAVE NOT ACTUALLY MOV'D RAX INTO CR3
    unsafe {
        asm!(
            "push rax",
            "push rbx",
            "push rcx",
            "push rdx",
            "push rbp",
            "push rdi",
            "push rsi",
            "push r8",
            "push r9",
            "push r10",
            "push r11",
            "push r12",
            "push r13",
            "push r14",
            "push r15",
            "mov rax, cr3",
            "push rax",
            "mov rdi, rsp",
            "cld",
            "call {0}",
            "mov rsp, rax",
            "mov rax, [rsp]",
            "add rsp, 0x08",
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop r11",
            "pop r10",
            "pop r9",
            "pop r8",
            "pop rsi",
            "pop rdi",
            "pop rbp",
            "pop rdx",
            "pop rcx",
            "pop rbx",
            "pop rax",
            "iretq",
            sym pit_handler,
            options(noreturn)
        );
    }
}

/*
    Very similar to handling a regular interrupt but preserve the return value
*/
#[naked]
pub extern "C" fn setup_syscall_handler() -> ! {
    unsafe {
        asm!(
            "push rax",
            "push rbx",
            "push rcx",
            "push rdx",
            "push rbp",
            "push rdi",
            "push rsi",
            "push r8",
            "push r9",
            "push r10",
            "push r11",
            "push r12",
            "push r13",
            "push r14",
            "push r15",
            "mov rdi, rsp",
            "add rdi, 8*8",
            "cld",
            "call {0}",
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop r11",
            "pop r10",
            "pop r9",
            "pop r8",
            "pop rsi",
            "pop rdi",
            "pop rbp",
            "pop rdx",
            "pop rcx",
            "pop rbx",
            "add rsp, 0x08",
            "iretq",
            sym test_syscall_handler,
            options(noreturn)
        );
    }
}
