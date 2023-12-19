- Add scrolling to vga_text mode
- Should there be hashmap of pages a process is using (address + numbers) or something similar
- Add support for grub modules
- Parse the multiboot properly
- Figure whether TSS is needed
- Change interrupts to allow for simple more flags (present, user, etc just more customisable)
- Bochs breakpoint is xchg bx, bx
- https://wiki.osdev.org/Exceptions
- https://wiki.osdev.org/Programmable_Interval_Timer

macro_rules! setup_exception_with_error_handler {
    ($exception_num: expr) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!(
                    "cld",
                    "push rax", // Push all registers
                    "push rbp",
                    "push rdi",
                    "push rsi",
                    "mov rdx, [rsp + 4*9]" // Load the error code
                    "mov rdi, rsp", // Load the ExceptionStackFrame
                    "mov rsi, {0}", // Load the exception id
                    "add rdi, 4*8", // Adjust for the pushed variables
                    "sub rsp, 8", // Align the stack pointer
                    "call {1}", // Call the handler
                    "pop rsi", // Pop all registers
                    "pop rdi",
                    "pop rbp",
                    "pop rax",
                    "add rsp, 8",
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