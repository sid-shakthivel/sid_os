// src/elf.rs

/*
    Executable and Linkable Format which is used to store programs
    Linkers combine elf files into an executable or library (uses sections)
    Loaders load the file into memory (uses segments)

    +-----------+----------------------------------+
    |   Name    |             Purpose              |
    +-----------+----------------------------------+
    | .text     | code                             |
    | .data     | initialised data with read/write |
    | .bss      | unitialised data                 |
    | .roadata  | initialised data with read only  |
    +-----------+----------------------------------+
*/

#![allow(dead_code)]
#![allow(unused_variables)]

use crate::memory::allocator::kmalloc;
use crate::memory::page_frame_allocator::PAGE_FRAME_ALLOCATOR;
use crate::memory::{page_frame_allocator, paging};
use crate::{print_serial, CONSOLE};
use core::future::pending;
use core::{mem, num};

type Elf64Half = u16;
type Elf64Off = usize;
type Elf64Addr = usize;
type Elf64Word = u32;
type Elf64Xword = usize;

const ELF_DATA: u8 = 1; // Little Endian
const ELF_CLASS: u8 = 2; // 64 Bit
const ELF_VERSION: u8 = 1;
const ELF_MACHINE: Elf64Half = 0x3E; // AMD x86-64
const ELF_FLAG_MAG0: u8 = 0x7F;

#[repr(C, packed)]
struct ElfHeader {
    e_ident: [u8; 16],      // Magic number and other info
    e_type: Elf64Half,      // Object file type
    e_machine: Elf64Half,   // Architecture
    e_version: Elf64Word,   // Object file version
    e_entry: Elf64Addr,     // Entry
    e_phoff: Elf64Off,      // Program header table file offset
    e_shoff: Elf64Off,      // Section header table file offset
    e_flags: Elf64Word,     // Processor-specific flags
    e_ehsize: Elf64Half,    // ELF header size in bytes
    e_phentsize: Elf64Half, // Program header table entry size
    e_phnum: Elf64Half,     // Program header table entry count
    e_shentsize: Elf64Half, // Section header table entry size
    e_shnum: Elf64Half,     // Section header table entry count
    e_shstrndx: Elf64Half,  // Section header string table index
}

enum ElfIdent {
    EiMag0 = 0,       // 0x7F
    EiMag1 = 1,       // E
    EiMag2 = 2,       // L
    EiMag3 = 3,       // F
    EiClass = 4,      // Architecture
    EiData = 5,       // Byte order
    EiVersion = 6,    // ELF version
    EiOsabi = 7,      // OS specific
    EiAbiversion = 8, // OS specific
    EiPad = 9,        // Padding
}

#[repr(u16)]
#[derive(PartialEq, Debug, Clone, Copy)]
enum ElfType {
    EtNone = 0, // Unknown
    EtRel = 1,  // Relocatable
    EtExec = 2, // Executable
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct ElfProgramHeader {
    p_type: Elf64Word,    // Entry type
    p_flags: Elf64Word,   // Access permission flags
    p_offset: Elf64Off,   // File offset of contents
    p_vaddr: Elf64Addr,   // Virtual address in memory
    p_paddr: Elf64Addr,   // Physical address in memory
    p_filesz: Elf64Xword, // Size of contents in file in bytes
    p_memsz: Elf64Xword,  // Size of contents in memory in bytes
    p_align: Elf64Xword,  // Alignment in memory and file
}

#[derive(PartialEq, Copy, Clone)]
#[repr(u32)]
enum ProgramHeaderType {
    PtNull = 0, // Unused
    PtLoad = 1, // Loadable segment
}

pub fn parse(file_start: usize) {
    let elf_header = unsafe { &*(file_start as *const ElfHeader) };
    validate_file(elf_header);
    parse_program_headers(file_start, elf_header);
}

// Verify file starts with ELF Magic number and is built for the correct system
fn validate_file(elf_header: &ElfHeader) -> bool {
    assert!(
        elf_header.e_ident[ElfIdent::EiMag0 as usize] == ELF_FLAG_MAG0,
        "ELF Header EI_MAG0 incorrect\n"
    );

    assert!(
        elf_header.e_ident[ElfIdent::EiMag1 as usize] == ('E' as u8),
        "ELF header EI_MAG1 incorrect\n"
    );

    assert!(
        elf_header.e_ident[ElfIdent::EiMag2 as usize] == ('L' as u8),
        "ELF header EI_MAG2 incorrect\n"
    );

    assert!(
        elf_header.e_ident[ElfIdent::EiMag3 as usize] == ('F' as u8),
        "ELF header EI_MAG3 incorrect\n"
    );

    assert!(
        elf_header.e_ident[ElfIdent::EiClass as usize] == ELF_CLASS,
        "Unsupported ELF File class\n"
    );

    assert!(
        elf_header.e_ident[ElfIdent::EiData as usize] == ELF_DATA,
        "Unsupported ELF File byte order\n"
    );

    assert!(
        elf_header.e_ident[ElfIdent::EiVersion as usize] == ELF_VERSION,
        "Unsupported ELF version\n"
    );

    assert!(
        elf_header.e_machine == ELF_MACHINE,
        "Unsupported ELF file target\n"
    );

    assert!(
        elf_header.e_type == (ElfType::EtExec as u16),
        "Unsupported ELF file type"
    );

    return true;
}

/*
    Elf program headers specify where segments are located and point to them
    Segments which contain multiple sections
    These are utilised whilst executing
*/
fn parse_program_headers(file_start: usize, elf_header: &ElfHeader) {
    // Loop through the headers and load each loadable segment into memory

    // let unaligned_num_header = core::ptr::addr_of!(elf_header.e_phnum);
    // let aligned_num_header = unsafe { core::ptr::read_unaligned(unaligned_num_header) };
    // print_serial!("number of elf headers: {}\n", aligned_num_header);

    for i in 0..elf_header.e_phnum {
        let address =
            file_start + elf_header.e_phoff + (mem::size_of::<ElfProgramHeader>()) * (i as usize);
        let program_header = unsafe { &*(address as *const ElfProgramHeader) };

        match program_header.p_type {
            1 => {
                // LOAD
                // TODO: With multiple program headers may need to mulitply by i
                let source = file_start + program_header.p_offset as usize;
                load_segment_into_memory(
                    source,
                    program_header.p_filesz,
                    program_header.p_memsz,
                    program_header.p_vaddr,
                );
            }
            _ => {}
        }
    }
}

fn load_segment_into_memory(
    source_raw: usize,
    filesz: usize,
    memsz: usize,
    v_address: usize,
) -> usize {
    // Allocate appropriate amount of memory
    let rounded_size = page_frame_allocator::round_to_nearest_page(memsz);
    let number_of_pages = page_frame_allocator::get_page_number(rounded_size);

    let dest: *mut usize = PAGE_FRAME_ALLOCATOR
        .lock()
        .alloc_page_frames(number_of_pages);
    PAGE_FRAME_ALLOCATOR.free();
    let source = source_raw as *mut usize;

    unsafe {
        core::ptr::write_bytes(dest as *mut u8, 0, memsz as usize);
        core::ptr::copy_nonoverlapping(source as *mut u8, dest as *mut u8, filesz as usize);
    }

    // Map the physical pages to the virtual address provided
    // paging::map_pages(number_of_pages, dest as usize, v_address);
    paging::map_pages(number_of_pages, v_address, dest as usize);

    v_address + (rounded_size)
}
