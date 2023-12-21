use crate::print_serial;
use crate::CONSOLE;

// How many bytes from the start of the file we search for the header.
pub const MULTIBOOT_SEARCH: u32 = 32768;
pub const MULTIBOOT_HEADER_ALIGN: u32 = 8;

// The magic field should contain this.
pub const MULTIBOOT2_HEADER_MAGIC: u32 = 0xe85250d6;

// This should be in %eax.
pub const MULTIBOOT2_BOOTLOADER_MAGIC: u32 = 0x36d76289;

// Alignment of multiboot modules.
pub const MULTIBOOT_MOD_ALIGN: u32 = 0x00001000;

// Alignment of the multiboot info structure.
pub const MULTIBOOT_INFO_ALIGN: u32 = 0x00000008;

// Flags set in the 'flags' member of the multiboot header.
pub const MULTIBOOT_TAG_ALIGN: u16 = 8;
pub const MULTIBOOT_TAG_TYPE_END: u16 = 0;
pub const MULTIBOOT_TAG_TYPE_CMDLINE: u16 = 1;
pub const MULTIBOOT_TAG_TYPE_BOOT_LOADER_NAME: u16 = 2;
pub const MULTIBOOT_TAG_TYPE_MODULE: u16 = 3;
pub const MULTIBOOT_TAG_TYPE_BASIC_MEMINFO: u16 = 4;
pub const MULTIBOOT_TAG_TYPE_BOOTDEV: u16 = 5;
pub const MULTIBOOT_TAG_TYPE_MMAP: u16 = 6;
pub const MULTIBOOT_TAG_TYPE_VBE: u16 = 7;
pub const MULTIBOOT_TAG_TYPE_FRAMEBUFFER: u16 = 8;
pub const MULTIBOOT_TAG_TYPE_ELF_SECTIONS: u16 = 9;
pub const MULTIBOOT_TAG_TYPE_APM: u16 = 10;
pub const MULTIBOOT_TAG_TYPE_EFI32: u16 = 11;
pub const MULTIBOOT_TAG_TYPE_EFI64: u16 = 12;
pub const MULTIBOOT_TAG_TYPE_SMBIOS: u16 = 13;
pub const MULTIBOOT_TAG_TYPE_ACPI_OLD: u16 = 14;
pub const MULTIBOOT_TAG_TYPE_ACPI_NEW: u16 = 15;
pub const MULTIBOOT_TAG_TYPE_NETWORK: u16 = 16;
pub const MULTIBOOT_TAG_TYPE_EFI_MMAP: u16 = 17;
pub const MULTIBOOT_TAG_TYPE_EFI_BS: u16 = 18;
pub const MULTIBOOT_TAG_TYPE_EFI32_IH: u16 = 19;
pub const MULTIBOOT_TAG_TYPE_EFI64_IH: u16 = 20;
pub const MULTIBOOT_TAG_TYPE_LOAD_BASE_ADDR: u16 = 21;

pub const MULTIBOOT_HEADER_TAG_END: u32 = 0;
pub const MULTIBOOT_HEADER_TAG_INFORMATION_REQUEST: u32 = 1;
pub const MULTIBOOT_HEADER_TAG_ADDRESS: u32 = 2;
pub const MULTIBOOT_HEADER_TAG_ENTRY_ADDRESS: u32 = 3;
pub const MULTIBOOT_HEADER_TAG_CONSOLE_FLAGS: u32 = 4;
pub const MULTIBOOT_HEADER_TAG_FRAMEBUFFER: u32 = 5;
pub const MULTIBOOT_HEADER_TAG_MODULE_ALIGN: u32 = 6;
pub const MULTIBOOT_HEADER_TAG_EFI_BS: u32 = 7;
pub const MULTIBOOT_HEADER_TAG_ENTRY_ADDRESS_EFI32: u32 = 8;
pub const MULTIBOOT_HEADER_TAG_ENTRY_ADDRESS_EFI64: u32 = 9;
pub const MULTIBOOT_HEADER_TAG_RELOCATABLE: u32 = 10;

pub const MULTIBOOT_ARCHITECTURE_I386: u32 = 0;
pub const MULTIBOOT_ARCHITECTURE_MIPS32: u32 = 4;
pub const MULTIBOOT_HEADER_TAG_OPTIONAL: u32 = 1;

pub const MULTIBOOT_LOAD_PREFERENCE_NONE: u32 = 0;
pub const MULTIBOOT_LOAD_PREFERENCE_LOW: u32 = 1;
pub const MULTIBOOT_LOAD_PREFERENCE_HIGH: u32 = 2;

pub const MULTIBOOT_CONSOLE_FLAGS_CONSOLE_REQUIRED: u32 = 1;
pub const MULTIBOOT_CONSOLE_FLAGS_EGA_TEXT_SUPPORTED: u32 = 2;

// Type is ty

#[repr(C, align(8))]
pub struct MultibootHeader {
    magic: u32,
    architecture: u32,
    header_length: u32,
    checksum: u32,
}

#[repr(C)]
pub struct MultibootHeaderTag {
    pub ty: u16,
    pub flags: u16,
    pub size: u32,
}

#[repr(C)]
pub struct MultibootHeaderTagInformationRequest {
    pub ty: u16,
    pub flags: u16,
    pub size: u32,
    pub requests: [u32],
}

#[repr(C)]
pub struct MultibootHeaderTagAddress {
    pub ty: u16,
    pub flags: u16,
    pub size: u32,
    pub header_addr: u32,
    pub load_addr: u32,
    pub load_end_addr: u32,
    pub bss_end_addr: u32,
}

#[repr(C)]
pub struct MultibootHeaderTagFramebuffer {
    pub ty: u16,
    pub flags: u16,
    pub size: u32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

#[repr(C)]
pub struct MultibootHeaderTagModuleAlign {
    pub ty: u16,
    pub flags: u16,
    pub size: u32,
}

#[repr(C)]
pub struct MultibootMmapEntry {
    pub addr: u64,
    pub len: u64,
    pub ty: u32,
    pub zero: u32,
}

#[repr(C)]
pub struct MultibootTag {
    pub typ: u32,
    pub size: u32,
}

#[repr(C)]
pub struct MultibootTagString {
    pub ty: u32,
    pub size: u32,
    pub string: [char; 0],
}

#[repr(C)]
pub struct MultibootTagModule {
    pub ty: u32,
    pub size: u32,
    pub mod_start: u32,
    pub mod_end: u32,
    pub cmdline: [char; 0],
}

#[repr(C)]
pub struct MultibootTagBasicMeminfo {
    pub ty: u32,
    pub size: u32,
    pub mem_lower: u32,
    pub mem_upper: u32,
}

#[repr(C)]
pub struct MultibootMemoryMap {
    pub addr: u64,
    pub len: u64,
    pub typ: u32,
    _reserved: u32,
}

#[repr(C)]
pub struct MultibootTagMmap {
    pub ty: u32,
    pub size: u32,
    pub entry_size: u32,
    pub entry_version: u32,
    pub entries: *const MultibootMemoryMap,
}

#[repr(C)]
pub struct MultibootVbeInfoBlock {
    pub external_specification: [u8; 512],
}

#[repr(C)]
pub struct MultibootVbeModeInfoBlock {
    pub external_specification: [u8; 256],
}

#[repr(C)]
pub struct MultibootTagVbe {
    pub ty: u32,
    pub size: u32,
    pub vbe_mode: u16,
    pub vbe_interface_seg: u16,
    pub vbe_interface_off: u16,
    pub vbe_interface_len: u16,
    pub vbe_control_info: MultibootVbeInfoBlock,
    pub vbe_mode_info: MultibootVbeModeInfoBlock,
}

#[repr(C)]
pub struct MultibootTagFramebufferCommon {
    pub ty: u32,
    pub size: u32,
    pub framebuffer_addr: u64,
    pub framebuffer_pitch: u32,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_bpp: u8,
    pub framebuffer_type: u8,
    pub reserved: u16,
}

#[repr(C)]
pub struct MultibootTagFramebuffer {
    pub common: MultibootTagFramebufferCommon,
    pub palette_num_colors: u16,
    pub palette: [MultibootColor; 0],
}

#[repr(C)]
pub struct MultibootTagLoadBaseAddr {
    pub ty: u32,
    pub size: u32,
    pub load_base_addr: u32,
}

#[repr(C)]
pub struct Multiboot2BootInfo {
    pub total_size: u32,
    reserved: u32,
    pub tags: [u8; 5],
}

// #[derive(Clone)]
// pub struct Tag {
//     pub typ: u32, // u32
//     pub size: u32,
//     // followed by additional, tag specific fields
// }

// #[derive(Clone, Debug)]
// pub struct TagIter {
//     /// Pointer to the next tag. Updated in each iteration.
//     pub current: *const Tag,
//     /// The pointer right after the MBI. Used for additional bounds checking.
//     end_ptr_exclusive: *const u8,
// }

// impl TagIter {
//     /// Creates a new iterator
//     pub fn new(mem: &[u8]) -> TagIter {
//         assert_eq!(mem.as_ptr().align_offset(8), 0);
//         TagIter {
//             current: mem.as_ptr().cast(),
//             end_ptr_exclusive: unsafe { mem.as_ptr().add(mem.len()) },
//         }
//     }
// }

// impl Iterator for TagIter {
//     type Item = Tag;

//     fn next(&mut self) -> Option<Tag> {
//         // This never failed so far. But better be safe.
//         assert!(self.current.cast::<u8>() < self.end_ptr_exclusive);

//         let tag = unsafe { &*self.current };

//         match tag.typ {
//             0 => None, // end tag
//             _ => {
//                 // next pointer (rounded up to 8-byte alignment)
//                 let ptr_offset = (tag.size as usize + 7) & !7;

//                 // go to next tag
//                 self.current = unsafe { self.current.cast::<u8>().add(ptr_offset).cast::<Tag>() };

//                 Some(tag.clone())
//             }
//         }
//     }
// }

// impl Multiboot2BootInfo {
//     pub fn tags(&self) -> TagIter {
//         TagIter::new(&self.tags)
//     }
// }

#[repr(C)]
pub struct MultibootColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}
