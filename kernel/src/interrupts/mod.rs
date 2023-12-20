/*
Interrupts are signals which stop the operation flow of a computer in order to perform a set action
Example interrupts include keyboard, mouse, etc
After handling an interrupt the CPU returns to whatever it was doing
Interrupts are a more efficient solution than polling devices
An interrupt descriptor table defines what each interrupt will do (First 32 Exceptions)
*/

use self::pic::PicFunctions;
use self::pic::PICS;
#[warn(unused_assignments)]
use crate::interrupts::idt::GateType;
use crate::interrupts::idt::IDTEntry;
use crate::interrupts::idt::PrivilegeLevel;
use crate::interrupts::idt::IDT;
use crate::interrupts::idt::IDTR;
use crate::interrupts::idt::IDT_MAX_DESCRIPTIONS;
use crate::print_serial;
use crate::utils::ports::inb;
use crate::CONSOLE;

use core::arch::asm;

mod idt;
pub mod pic;
pub mod pit;

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

const LETTERS: &'static [char; 0x3A] = &[
    '\0', '\0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\0', '\t', 'q', 'w',
    'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n', '\0', 'a', 's', 'd', 'f', 'g', 'h',
    'j', 'k', '\0', ';', '\'', '`', '\0', '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/',
    '\0', '*', '\0', ' ',
];

pub type InterruptHandlerFunc = extern "C" fn() -> !;

#[derive(Debug)]
#[repr(C)]
struct ExceptionStackFrame {
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

// Purely for exceptions with an error code eg page faults
macro_rules! setup_exception_with_error_handler {
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
macro_rules! setup_general_interrupt_handler {
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
                    "add rdi, 7*8",
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

// Specifically for multitasking
macro_rules! setup_pit_handler {
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
                    "push r8",
                    "push r9",
                    "push r10",
                    "push r11",
                    "push r12",
                    "push r13",
                    "push r14",
                    "push r15",
                    "mov rdi, rsp", // Load ExceptionStackFrame
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
                    "pop rax",
                    "iretq",
                    sym exception_with_error_handler,
                    options(noreturn)
                );
        }
        wrapper
    }};
}

#[derive(Clone, Copy, Debug)]
enum PageFaultFlags {
    IsPresent, // Caused by non present page
    IsWrite,   // Caused by write access
    IsUser,    // Page fault occured in user mode
    IsReservedWrite,
    IsInstructionFetch,
    IsProtectionKey,
    IsShadowStack,
}

impl PageFaultFlags {
    fn iter() -> impl Iterator<Item = (usize, PageFaultFlags)> {
        [
            PageFaultFlags::IsPresent,
            PageFaultFlags::IsWrite,
            PageFaultFlags::IsUser,
            PageFaultFlags::IsReservedWrite,
            PageFaultFlags::IsInstructionFetch,
            PageFaultFlags::IsProtectionKey,
            PageFaultFlags::IsShadowStack,
        ]
        .iter()
        .enumerate()
        .map(|(index, &variant)| (index, variant))
    }
}

extern "C" fn exception_handler(stack_frame: &ExceptionStackFrame, exception_id: usize) {
    match exception_id {
        0..32 => {
            print_serial!("{}\n", EXCEPTION_MESSAGES[exception_id]);
        }
        _ => {}
    }

    print_serial!("{:?}\n", stack_frame);
}

extern "C" fn interrupt_handler(stack_frame: &ExceptionStackFrame) {
    // Handle keyboard
    let scancode = inb(0x60);

    let letter = translate(scancode, false);

    if letter != '0' {
        print_serial!("{}", letter);
    }

    PICS.lock().acknowledge(0x21 as u8);
    PICS.free();
}

extern "C" fn exception_with_error_handler(
    stack_frame: &ExceptionStackFrame,
    exception_id: usize,
    error_code: usize,
) {
    match exception_id {
        14 => {
            // Handle page fault by displaying which flags are set within error code
            print_serial!("{}\n", EXCEPTION_MESSAGES[exception_id]);

            for (index, flag) in PageFaultFlags::iter() {
                if ((index << 1) & error_code) != 0 {
                    print_serial!("{:?} is SET\n", flag);
                }
            }
        }
        _ => {}
    }

    print_serial!("{:?}\n", stack_frame);

    loop {}
}

fn translate(scancode: u8, uppercase: bool) -> char {
    if scancode > 0x3A {
        return '0';
    }

    if uppercase {
        return ((LETTERS[scancode as usize] as u8) - 0x20) as char;
    } else {
        return LETTERS[scancode as usize];
    }
}

pub fn init() {
    unsafe {
        // Setup exceptions
        IDT[0] = IDTEntry::new(
            GateType::Trap,
            PrivilegeLevel::Ring3,
            setup_general_interrupt_handler!(exception_handler, 0),
        );

        IDT[1] = IDTEntry::new(
            GateType::Trap,
            PrivilegeLevel::Ring3,
            setup_general_interrupt_handler!(exception_handler, 1),
        );

        IDT[3] = IDTEntry::new(
            GateType::Trap,
            PrivilegeLevel::Ring3,
            setup_general_interrupt_handler!(exception_handler, 3),
        );

        IDT[6] = IDTEntry::new(
            GateType::Trap,
            PrivilegeLevel::Ring3,
            setup_general_interrupt_handler!(exception_handler, 6),
        );

        // Setup exceptions with an error code

        IDT[14] = IDTEntry::new(
            GateType::Trap,
            PrivilegeLevel::Ring3,
            setup_exception_with_error_handler!(14),
        );

        // Interrupts

        // Keyboard
        IDT[0x21] = IDTEntry::new(
            GateType::Interrupt,
            PrivilegeLevel::Ring3,
            setup_general_interrupt_handler!(interrupt_handler, 0x21),
        );

        // Syscalls

        // Actually set the IDTR values
        let idt_address = (&IDT[0] as *const IDTEntry) as u64;
        IDTR.limit = (core::mem::size_of::<IDTEntry>() as u16) * (IDT_MAX_DESCRIPTIONS as u16 - 1);
        IDTR.base = idt_address;

        // Refresh
        flush_idt();
    }
}

pub fn enable() {
    unsafe {
        asm!("sti");
    }
}

pub fn disable() {
    unsafe {
        asm!("cld");
    }
}

extern "C" {
    fn flush_idt();
}
