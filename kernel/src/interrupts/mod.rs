/*
Interrupts are signals which stop the operation flow of a computer in order to perform a set action
Example interrupts include keyboard, mouse, etc
After handling an interrupt the CPU returns to whatever it was doing
Interrupts are a more efficient solution than polling devices
An interrupt descriptor table defines what each interrupt will do (First 32 Exceptions)
*/

use isr::setup_syscall_handler;

use self::pic::PicFunctions;
use self::pic::PICS;
use crate::dev::keyboard::KEYBOARD;
use crate::dev::mouse::MOUSE;
use crate::ds::stack;
#[warn(unused_assignments)]
use crate::interrupts::idt::GateType;
use crate::interrupts::idt::IDTEntry;
use crate::interrupts::idt::PrivilegeLevel;
use crate::interrupts::idt::IDT;
use crate::interrupts::idt::IDTR;
use crate::interrupts::idt::IDT_MAX_DESCRIPTIONS;
use crate::memory::gdt::TSS;
use crate::multitask::syscalls::syscall_handler;
use crate::multitask::PROCESS_MANAGER;
use crate::print_serial;
use crate::setup_exception_with_e_handler;
use crate::setup_interrupt_handler;
use crate::utils::multiboot2::MULTIBOOT2_BOOTLOADER_MAGIC;
use crate::utils::ports::inb;
use crate::CONSOLE;
use core::arch::asm;
use core::panic;

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

pub type InterruptHandlerFunc = extern "C" fn() -> !;

#[derive(Debug)]
#[repr(C)]
struct StackFrame {
    rip: usize,
    cs: usize,
    rflags: usize,
    rsp: usize,
    ss: usize,
}

#[derive(Debug)]
#[repr(C)]
pub struct InterruptStackFrame {
    pub rsi: usize,
    pub rdi: usize,
    pub rbp: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rbx: usize,
    pub rax: usize,
    rip: usize,
    cs: usize,
    rflags: usize,
    rsp: usize,
    ss: usize,
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

pub extern "C" fn exception_handler(stack_frame: &StackFrame, exception_id: usize) {
    match exception_id {
        0..32 => {
            print_serial!("{}\n", EXCEPTION_MESSAGES[exception_id]);
        }
        _ => {}
    }

    print_serial!("{:?}\n", stack_frame);

    panic!("Unhandled exception: {}", exception_id);
}

pub extern "C" fn test_syscall_handler(stack_frame: &InterruptStackFrame) -> isize {
    syscall_handler(stack_frame) as isize
}

pub extern "C" fn interrupt_handler(stack_frame: &InterruptStackFrame, interrupt_id: usize) {
    PICS.lock().acknowledge(interrupt_id as u8);
    PICS.free();

    match interrupt_id {
        0x21 => {
            KEYBOARD.lock().handle_keyboard();
            KEYBOARD.free();
        }
        0x2c => {
            MOUSE.lock().handle_mouse_interrupt();
            MOUSE.free();
        }
        _ => {}
    }
}

pub extern "C" fn exception_with_error_handler(
    stack_frame: &StackFrame,
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

    print_serial!("{}\n", EXCEPTION_MESSAGES[exception_id]);
    panic!("{:?}\n", stack_frame);

    loop {}
}

pub extern "C" fn pit_handler(old_task_rsp: usize) -> usize {
    PICS.lock().acknowledge(0x20 as u8);
    PICS.free();

    let rsp = PROCESS_MANAGER.lock().switch_process(old_task_rsp);
    PROCESS_MANAGER.free();

    rsp
}

pub fn init() {
    unsafe {
        // Setup exceptions
        IDT[0] = IDTEntry::new_default_trap(setup_interrupt_handler!(exception_handler, 0));
        IDT[1] = IDTEntry::new_default_trap(setup_interrupt_handler!(exception_handler, 1));
        IDT[2] = IDTEntry::new_default_trap(setup_interrupt_handler!(exception_handler, 2));
        IDT[3] = IDTEntry::new_default_trap(setup_interrupt_handler!(exception_handler, 3));
        IDT[4] = IDTEntry::new_default_trap(setup_interrupt_handler!(exception_handler, 4));
        IDT[5] = IDTEntry::new_default_trap(setup_interrupt_handler!(exception_handler, 5));
        IDT[6] = IDTEntry::new_default_trap(setup_interrupt_handler!(exception_handler, 6));
        IDT[7] = IDTEntry::new_default_trap(setup_interrupt_handler!(exception_handler, 7));
        IDT[8] = IDTEntry::new_default_trap(setup_exception_with_e_handler!(8));
        IDT[9] = IDTEntry::new_default_trap(setup_interrupt_handler!(exception_handler, 9));
        IDT[10] = IDTEntry::new_default_trap(setup_exception_with_e_handler!(10));
        IDT[11] = IDTEntry::new_default_trap(setup_exception_with_e_handler!(11));
        IDT[12] = IDTEntry::new_default_trap(setup_exception_with_e_handler!(12));
        IDT[13] = IDTEntry::new_default_trap(setup_exception_with_e_handler!(13));
        IDT[14] = IDTEntry::new_default_trap(setup_exception_with_e_handler!(14));
        IDT[16] = IDTEntry::new_default_trap(setup_interrupt_handler!(exception_handler, 16));
        IDT[17] = IDTEntry::new_default_trap(setup_exception_with_e_handler!(17));
        IDT[18] = IDTEntry::new_default_trap(setup_interrupt_handler!(exception_handler, 18));
        IDT[19] = IDTEntry::new_default_trap(setup_interrupt_handler!(exception_handler, 19));
        IDT[20] = IDTEntry::new_default_trap(setup_interrupt_handler!(exception_handler, 20));
        IDT[21] = IDTEntry::new_default_trap(setup_exception_with_e_handler!(21));
        IDT[28] = IDTEntry::new_default_trap(setup_interrupt_handler!(exception_handler, 28));
        IDT[29] = IDTEntry::new_default_trap(setup_exception_with_e_handler!(29));
        IDT[30] = IDTEntry::new_default_trap(setup_exception_with_e_handler!(30));

        // General interrupts
        IDT[0x20] = IDTEntry::new_default_interrupt(setup_pit_handler); // Timer (PIT)
        IDT[0x21] =
            IDTEntry::new_default_interrupt(setup_interrupt_handler!(interrupt_handler, 0x21)); // Keyboard
        IDT[0x2c] =
            IDTEntry::new_default_interrupt(setup_interrupt_handler!(interrupt_handler, 0x2c)); // Mouse

        // Syscalls
        IDT[0x80] = IDTEntry::new_default_interrupt(setup_syscall_handler);

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
