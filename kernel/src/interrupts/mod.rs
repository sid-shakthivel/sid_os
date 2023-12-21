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
use crate::setup_exception_with_error_handler;
use crate::setup_general_interrupt_handler;
use crate::utils::ports::inb;
use crate::CONSOLE;
use core::arch::asm;

use crate::interrupts::isr::setup_pit_handler;

mod idt;
mod isr;
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

pub extern "C" fn exception_handler(stack_frame: &ExceptionStackFrame, exception_id: usize) {
    match exception_id {
        0..32 => {
            print_serial!("{}\n", EXCEPTION_MESSAGES[exception_id]);
        }
        _ => {}
    }

    print_serial!("{:?}\n", stack_frame);
}

pub extern "C" fn interrupt_handler(stack_frame: &ExceptionStackFrame) {
    // Handle keyboard
    let scancode = inb(0x60);

    let letter = translate(scancode, false);

    if letter != '0' {
        print_serial!("{}", letter);
    }

    PICS.lock().acknowledge(0x21 as u8);
    PICS.free();
}

pub extern "C" fn exception_with_error_handler(
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

pub extern "C" fn pit_handler(old_task_rsp: usize) {
    // Retrieve the current task's rsp
    // Return the new_task rsp
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
