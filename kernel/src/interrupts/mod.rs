/*
Interrupts are signals which stop the operation flow of a computer in order to perform a set action
Example interrupts include keyboard, mouse, etc
After handling an interrupt the CPU returns to whatever it was doing
Interrupts are a more efficient solution than polling devices
An interrupt descriptor table defines what each interrupt will do (First 32 Exceptions)
*/

#[warn(unused_assignments)]
use crate::print_serial;
use crate::CONSOLE;

use core::arch::asm;

const IDT_MAX_DESCRIPTIONS: usize = 8;

pub type InterruptHandlerFunc = extern "C" fn();

macro_rules! push_registers {
    () => {
        // asm!("push rax
        //       push rbx
        //       push rcx
        //       push rdx
        //       push rbp
        //       push rdi
        //       push rsi
        //       push r8
        //       push r9
        //       push r10
        //       push r11
        //       push r12
        //       push r13
        //       push r14
        //       push r15
        // " :::: "intel", "volatile");
    };
}

macro_rules! pop_registers {
    () => {
        // asm!("pop r15
        //       pop r14
        //       pop r13
        //       pop r12
        //       pop r11
        //       pop r10
        //       pop r9
        //       pop r8
        //       pop rsi
        //       pop rdi
        //       pop rbp
        //       pop rdx
        //       pop rcx
        //       pop rbx
        //       pop rax
        //     " :::: "intel", "volatile");
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

macro_rules! exception_handler {
    ($func_name: ident, $exception_num: expr) => {{
        // #[naked]
        extern "C" fn wrapper() {
            //     unsafe {
            //         push_registers!();
            //         asm!("
            //               cld
            //               mov rdi, rsp
            //               mov rsi, $1  // move the number to rsi
            //               add rdi, 9*8 // skip exception stack frame pointer
            //               call $0"
            //               :: "i"($func_name as extern "C" fn(
            //                   &ExceptionStackFrame)), "i"(exception_num)
            //               : "rdi", "rsi" : "intel");

            //         pop_registers!();
            //         asm!("iretq" :::: "intel", "volatile");
            //         ::core::intrinsics::unreachable();
            //     }
        }
        wrapper
    }};
}

#[derive(Copy, Clone)]
#[repr(C)]
struct IDTEntry {
    isr_low: u16,   // Lower 16 bits of the ISR's address
    kernel_cs: u16, // GDT segment selector that CPU loads before calling ISR
    ist: u8,        // IST in TSS which CPU will load into RSP (zero currently)
    attributes: u8, // Type and attributes
    isr_mid: u16,   // Higher 16 bits of the lower 32 bits of ISR's address
    isr_high: u32,  // Higher 32 bits of ISR's address
    reserved: u32,  // Set to zero
}

impl IDTEntry {
    pub fn new(
        gate_type: GateType,
        privilege_level: PrivilegeLevel,
        func_addr_raw: InterruptHandlerFunc,
    ) -> IDTEntry {
        let func_addr = func_addr_raw as usize;

        return IDTEntry {
            isr_low: (func_addr & 0xFFFF) as u16,
            kernel_cs: 0x08,
            ist: 0,
            attributes: IDTEntry::generate_flags((gate_type, privilege_level)),
            isr_mid: ((func_addr >> 16) & 0xFFFF) as u16,
            isr_high: (func_addr >> 32) as u32,
            reserved: 0,
        };
    }

    fn generate_flags(data: (GateType, PrivilegeLevel)) -> u8 {
        let mut attributes: u8 = match data.0 {
            GateType::Trap => 0x8F,
            GateType::Interrupt => 0x8E,
        };

        attributes = match data.1 {
            PrivilegeLevel::Ring3 => attributes | (1 << 5) | (1 << 6),
            _ => attributes,
        };

        return attributes;
    }
}

#[repr(C, packed)]
struct Idtr {
    limit: u16, // Memory taken up by IDT in bytes ((256 - 1) * 16)
    base: u64,  // Base address of IDT
}

enum GateType {
    Interrupt,
    Trap, // For exceptions
}

enum PrivilegeLevel {
    Ring0, // Kernel mode
    Ring3, // User mode
}

#[no_mangle]
static mut IDTR: Idtr = Idtr { limit: 0, base: 0 };
static mut IDT: [IDTEntry; IDT_MAX_DESCRIPTIONS] = [IDTEntry {
    isr_low: 0,
    kernel_cs: 0,
    ist: 0,
    attributes: 0,
    isr_mid: 0,
    isr_high: 0,
    reserved: 0,
}; IDT_MAX_DESCRIPTIONS];

extern "C" fn divide_by_zero_handler() -> ! {
    print_serial!("EXCEPTION: DIVIDE BY ZERO");
    loop {}
}

pub fn init() {
    unsafe {
        // Exceptions
        IDT[0] = IDTEntry::new(
            GateType::Trap,
            PrivilegeLevel::Ring3,
            exception_handler!(divide_by_zero_handler, 0),
        );

        // Interrupts

        // Syscalls

        // Actually set the IDT values
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
