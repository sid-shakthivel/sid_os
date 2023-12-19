/*
Interrupts are signals which stop the operation flow of a computer in order to perform a set action
Example interrupts include keyboard, mouse, etc
After handling an interrupt the CPU returns to whatever it was doing
Interrupts are a more efficient solution than polling devices
An interrupt descriptor table defines what each interrupt will do (First 32 Exceptions)
*/

use crate::interrupts::idt::GateType;
use crate::interrupts::idt::IDTEntry;
use crate::interrupts::idt::PrivilegeLevel;
use crate::interrupts::idt::IDT;
use crate::interrupts::idt::IDTR;
use crate::interrupts::idt::IDT_MAX_DESCRIPTIONS;

#[warn(unused_assignments)]
use crate::print_serial;
use crate::CONSOLE;

mod idt;

use core::arch::asm;

// TODO: Replace with a hashmap
const EXCEPTION_MESSAGES: &'static [&'static str] = &[
    "Divide By Zero",
    "Debug",
    "Non-maskable Interrupt",
    "Breakpoint",
    "Overflow",
    "Bound Range Exceeded",
    "Invalid Opcode",
    "Device not Available",
    "Double Fault",
    "Coprocessor Segment Overrun",
    "Invalid TSS",
    "Segment Not Present",
    "Stack-Segment Fault",
    "General Protection Fault",
    "Page Fault",
    "Reserved",
    "x87 Floating Point Exception",
    "Alignment Check",
    "Machine Check",
    "SIMD Floating Point Exception",
    "Virtualisation Exception",
    "Control Protection  Exception",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Hypervisor Injection Exception",
    "VMM Communication Exception",
    "Security Exception",
    "Reserved",
    "Triple Fault",
    "FPU Error Interrupt",
];

const STATIC_NUMBERS: [usize; 32] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32,
];

pub type InterruptHandlerFunc = extern "C" fn() -> !;

macro_rules! push_registers {
    () => {
        asm!(
            "push rax", "push rbx", "push rcx", "push rdx", "push rbp", "push rdi", "push rsi",
            "push r8", "push r9", "push r10", "push r11", "push r12", "push r13", "push r14",
            "push r15"
        );
    };
}

#[derive(Debug)]
#[repr(C)]
struct ExceptionStackFrame {
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

macro_rules! setup_exception_with_error_handler {
    ($exception_num: expr) => {
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
                    sym exception_handler,
                    options(noreturn)
                );
            }
        }
        wrapper
    };
}

macro_rules! setup_exception_handler {
    ($exception_num: expr) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!(
                    "cld",
                    "push rax",
                    "push rbp",
                    "push rdi",
                    "push rsi",
                    "mov rdi, rsp",
                    "mov rsi, {0}",
                    "add rdi, 4*8",
                    "call {1}",
                    "pop rsi",
                    "pop rdi",
                    "pop rbp",
                    "pop rax",
                    "iretq",
                    const $exception_num,
                    sym exception_handler,
                    options(noreturn)
                );
            }
        }
        wrapper
    }};
}



extern "C" fn exception_handler(stack_frame: &ExceptionStackFrame, exception_id: usize) -> ! {
    match exception_id {
        0..20 => {
            print_serial!("{}\n", EXCEPTION_MESSAGES[exception_id]);
        }
        _ => {}
    }

    print_serial!("{:?}\n", stack_frame);

    loop {} // Need to remove this
}

extern "C" fn exception_with_error_handler(
    stack_frame: &ExceptionStackFrame,
    exception_id: usize,
    error_code: usize,
) -> ! {
    match exception_id {
        14 => {
            // Handle page fault
            print_serial!("{}\n", EXCEPTION_MESSAGES[exception_id]);

            
        }
        _ => {}
    }

    print_serial!("{:?}\n", stack_frame);

    loop {} // Need to remove this
}

pub fn init() {
    unsafe {
        // Setup all exceptions
        IDT[0] = IDTEntry::new(
            GateType::Trap,
            PrivilegeLevel::Ring3,
            setup_exception_handler!(0),
        );

        IDT[1] = IDTEntry::new(
            GateType::Trap,
            PrivilegeLevel::Ring3,
            setup_exception_handler!(1),
        );

        IDT[3] = IDTEntry::new(
            GateType::Trap,
            PrivilegeLevel::Ring3,
            setup_exception_handler!(3),
        );

        IDT[6] = IDTEntry::new(
            GateType::Trap,
            PrivilegeLevel::Ring3,
            setup_exception_handler!(6),
        );

        // Setup exceptions with an error code

        IDT[14] = IDTEntry::new(
            GateType::Trap,
            PrivilegeLevel::Ring3,
            setup_exception_handler!(14),
        );

        // Interrupts

        // Syscalls

        // Actually set the IDTR values
        let idt_address = (&IDT[0] as *const IDTEntry) as u64;
        IDTR.limit = (core::mem::size_of::<IDTEntry>() as u16) * (IDT_MAX_DESCRIPTIONS as u16 - 1);
        IDTR.base = idt_address;

        // Refresh
        flush_idt();
    }
}

extern "C" {
    fn flush_idt();
}
