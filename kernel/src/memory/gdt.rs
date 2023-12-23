/*
    Global descriptor table which contains 8 byte entries about memory segments
    Previousl setting the GDT within rust and calling the relevant assembly has proven futile
    The entries for the GDT will be generated here and the values will be copied into the boot file
*/

/*
GDT Entry:
+---------+------------+------------+-----------------+-----------+---------+---------+--------+--------+---------+
|  0-41   |     42     |     43     |       44        |   45-46   |   47    |  48-52  |   53   |   54   |  55-63  |
+---------+------------+------------+-----------------+-----------+---------+---------+--------+--------+---------+
| Ignored | Conforming | Executable | Descriptor Type | Privilege | Present | Ignored | 64-Bit | 32-Bit | Ignored |
+---------+------------+------------+-----------------+-----------+---------+---------+--------+--------+---------+
*/

use crate::{output::uart::CONSOLE, print_serial};
use core::arch::asm;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct GlobalDescriptorTable {
    table: [usize; 8],
    len: usize,
}

#[no_mangle]
pub static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();
pub static mut TSS: TaskStateSegment = TaskStateSegment::new();

// Load GDT
fn lgdt(gdt: &GDTPointer) {
    unsafe {
        asm!("lgdt [{}]", in(reg) gdt, options(readonly, nostack, preserves_flags));
    }
}

pub fn init() {
    unsafe {
        GDT.initalise();
        GDT.load();
        GDT.load_tss();
    }
}

#[repr(C, packed(2))]
pub struct GDTPointer {
    pub limit: u16,
    pub base: usize,
}

impl GlobalDescriptorTable {
    pub const fn new() -> GlobalDescriptorTable {
        GlobalDescriptorTable {
            table: [0; 8],
            len: 1,
        }
    }

    pub fn initalise(&mut self) {
        self.add_entry(Descriptor::kernel_code_segment());
        self.add_entry(Descriptor::kernel_data_segment());
        self.add_entry(Descriptor::user_code_segment());
        self.add_entry(Descriptor::user_data_segment());
        unsafe {
            self.add_entry(Descriptor::task_state_segment(&TSS));
        }
    }

    pub fn load(&self) {
        lgdt(&self.pointer());
    }

    pub fn load_tss(&self) {
        unsafe {
            asm!("ltr {0:x}", in(reg) 0x28, options(nostack, preserves_flags));
        }
    }

    pub fn pointer(&self) -> GDTPointer {
        use core::mem::size_of;
        GDTPointer {
            base: self.table.as_ptr() as usize,
            limit: (self.len * size_of::<usize>() - 1) as u16,
        }
    }

    pub fn add_entry(&mut self, entry: Descriptor) {
        match entry {
            Descriptor::UserSegment(value) => {
                self.table[self.len] = value;
                self.len += 1;
            }
            Descriptor::SystemSegment(low) => {
                let test = self.len;

                self.table[self.len] = low;
                self.len += 1;

                // Has to be done to avoid errors
                self.table[self.len] = 0;
                self.len += 1;
            }
        }
    }
}

pub enum Descriptor {
    UserSegment(usize),   // Code/data segments
    SystemSegment(usize), // TSS
}

impl Descriptor {
    pub fn kernel_code_segment() -> Descriptor {
        Descriptor::UserSegment(Descriptor::generate(GDTBits::parse_flags(&[
            GDTBits::IsAccessed,
            GDTBits::IsWriteable,
            GDTBits::IsExecutable,
            GDTBits::IsCodeOrData,
            GDTBits::IsPresent,
            GDTBits::Is64Bit,
        ])))
    }

    pub fn kernel_data_segment() -> Descriptor {
        Descriptor::UserSegment(Descriptor::generate(GDTBits::parse_flags(&[
            GDTBits::IsAccessed,
            GDTBits::IsWriteable,
            GDTBits::IsCodeOrData,
            GDTBits::IsPresent,
            GDTBits::Is64Bit,
        ])))
    }

    pub fn user_code_segment() -> Descriptor {
        Descriptor::UserSegment(Descriptor::generate(GDTBits::parse_flags(&[
            GDTBits::IsAccessed,
            GDTBits::IsWriteable,
            GDTBits::IsExecutable,
            GDTBits::IsUserPage,
            GDTBits::IsCodeOrData,
            GDTBits::IsPresent,
            GDTBits::Is64Bit,
        ])))
    }

    pub fn user_data_segment() -> Descriptor {
        Descriptor::UserSegment(Descriptor::generate(GDTBits::parse_flags(&[
            GDTBits::IsAccessed,
            GDTBits::IsWriteable,
            GDTBits::IsCodeOrData,
            GDTBits::IsUserPage,
            GDTBits::IsPresent,
            GDTBits::Is64Bit,
        ])))
    }

    pub fn task_state_segment(tss: &TaskStateSegment) -> Descriptor {
        use core::mem::size_of;

        let mut descriptor: usize = 0;
        let base = tss as *const _ as usize;
        let limit: usize = (size_of::<TaskStateSegment>() - 1);
        let flag: usize = 0x89;

        descriptor = limit & 0x000F0000; // set limit bits 19:16
        descriptor |= (flag << 8) & 0x00F0FF00; // set type, p, dpl, s, g, d/b, l and avl fields
        descriptor |= (base >> 16) & 0x000000FF; // set base bits 23:16
        descriptor |= base & 0xFF000000; // set base bits 31:24

        // Shift by 32 to allow for low part of segment
        descriptor <<= 32;

        // Create the low 32 bit segment
        descriptor |= base << 16; // set base bits 15:0
        descriptor |= limit & 0x0000FFFF; // set limit bits 15:0

        let high = descriptor >> 32;

        assert!(high > 0, "The high is greater than 0");

        return Descriptor::SystemSegment(descriptor);
    }

    fn generate(mut entry: usize) -> usize {
        entry |= 0x0000FFFF;
        entry |= 0xF << 48; // For limit 16..19
        entry
    }
}

enum GDTBits {
    IsAccessed,
    IsWriteable,
    IsExecutable,
    IsCodeOrData,
    IsUserPage,
    IsPresent,
    Is64Bit,
}

impl GDTBits {
    pub fn parse_flags(required_bits: &[GDTBits]) -> usize {
        let mut value = 0;

        for bit in required_bits {
            value = match bit {
                GDTBits::IsAccessed => value | (1 << 40),
                GDTBits::IsWriteable => value | (1 << 41),
                GDTBits::IsExecutable => value | (1 << 43), // If 1 => code
                GDTBits::IsCodeOrData => value | (1 << 44), // If 1 => data
                GDTBits::IsUserPage => value | (3 << 45),   // If 3 => user
                GDTBits::IsPresent => value | (1 << 47),    // If 1 => valid segment
                GDTBits::Is64Bit => value | (1 << 53),
            };
        }

        value
    }
}

/*
TSS:
+------------+-----------+-------+------+-----------+---------+-------------+-----------+---------+-------------+------------+------------+---------+
|    0-15    |   16-39   | 40-43 |  44  |   45-46   |   47    |    48-51    |    52     |  43-54  |     55      |   56-63    |   64-95    | 96-127  |
+------------+-----------+-------+------+-----------+---------+-------------+-----------+---------+-------------+------------+------------+---------+
| Limit 0-15 | Base 0-23 | Type  | Zero | Privilege | Present | Limit 16-19 | Available | Ignored | Granularity | Base 24-31 | Base 32-63 | Ignored |
+------------+-----------+-------+------+-----------+---------+-------------+-----------+---------+-------------+------------+------------+---------+
*/

#[repr(C, packed(4))]
pub struct TaskStateSegment {
    reserved_1: u32,
    pub privilege_stack_table: [usize; 3],
    reserved_2: usize,
    pub interrupt_stack_table: [usize; 7],
    reserved_3: u64,
    reserved_4: u16,
    pub iomap_base: u16,
}

impl TaskStateSegment {
    pub const fn new() -> TaskStateSegment {
        TaskStateSegment {
            privilege_stack_table: [0; 3],
            interrupt_stack_table: [0; 7],
            iomap_base: core::mem::size_of::<TaskStateSegment>() as u16,
            reserved_1: 0,
            reserved_2: 0,
            reserved_3: 0,
            reserved_4: 0,
        }
    }
}
