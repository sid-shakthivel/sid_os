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

use crate::{print_serial, output::uart::CONSOLE};

enum GDTBits {
    IsAccessed,
    IsWriteable,
    IsExecutable,
    IsCodeOrData,
    IsUserPage,
    IsPresent,
    Is64Bit,
}

pub fn generate_gdt_values() {
    let kernel_code = process_gdt_entry(&[
        GDTBits::IsAccessed,
        GDTBits::IsWriteable,
        GDTBits::IsExecutable,
        GDTBits::IsCodeOrData,
        GDTBits::IsPresent,
        GDTBits::Is64Bit,
    ]);

    let kernel_data = process_gdt_entry(&[
        GDTBits::IsAccessed,
        GDTBits::IsWriteable,
        GDTBits::IsCodeOrData,
        GDTBits::IsPresent,
        GDTBits::Is64Bit,
    ]);

    let user_code = process_gdt_entry(&[
        GDTBits::IsAccessed,
        GDTBits::IsWriteable,
        GDTBits::IsExecutable,
        GDTBits::IsUserPage,
        GDTBits::IsCodeOrData,
        GDTBits::IsPresent,
        GDTBits::Is64Bit,
    ]);

    let user_data = process_gdt_entry(&[
        GDTBits::IsAccessed,
        GDTBits::IsWriteable,
        GDTBits::IsCodeOrData,
        GDTBits::IsUserPage,
        GDTBits::IsPresent,
        GDTBits::Is64Bit,
    ]);

    print_serial!("{:x}\n", kernel_code);
    print_serial!("{:x}\n", kernel_data);
    print_serial!("{:x}\n", user_code);
    print_serial!("{:x}\n", user_data);
}

fn process_gdt_entry(required_bits: &[GDTBits]) -> usize {
    let mut value: usize = 0;

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

    // For limit 0..15
    value |= 0x0000FFFF;

    // For limit 16..19
    value |= 0xF << 48;

    return value;
}

/*
TSS:
+------------+-----------+-------+------+-----------+---------+-------------+-----------+---------+-------------+------------+------------+---------+
|    0-15    |   16-39   | 40-43 |  44  |   45-46   |   47    |    48-51    |    52     |  43-54  |     55      |   56-63    |   64-95    | 96-127  |
+------------+-----------+-------+------+-----------+---------+-------------+-----------+---------+-------------+------------+------------+---------+
| Limit 0-15 | Base 0-23 | Type  | Zero | Privilege | Present | Limit 16-19 | Available | Ignored | Granularity | Base 24-31 | Base 32-63 | Ignored |
+------------+-----------+-------+------+-----------+---------+-------------+-----------+---------+-------------+------------+------------+---------+
*/
