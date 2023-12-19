use crate::interrupts::InterruptHandlerFunc;

pub const IDT_MAX_DESCRIPTIONS: usize = 256;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct IDTEntry {
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
pub struct Idtr {
    pub limit: u16, // Memory taken up by IDT in bytes ((256 - 1) * 16)
    pub base: u64,  // Base address of IDT
}

pub enum GateType {
    Interrupt,
    Trap, // For exceptions
}

pub enum PrivilegeLevel {
    Ring0, // Kernel mode
    Ring3, // User mode
}

#[no_mangle]
pub static mut IDTR: Idtr = Idtr { limit: 0, base: 0 };
pub static mut IDT: [IDTEntry; IDT_MAX_DESCRIPTIONS] = [IDTEntry {
    isr_low: 0,
    kernel_cs: 0,
    ist: 0,
    attributes: 0,
    isr_mid: 0,
    isr_high: 0,
    reserved: 0,
}; IDT_MAX_DESCRIPTIONS];
