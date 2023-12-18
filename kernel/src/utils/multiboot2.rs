use core::fmt;

// Definitions
const MULTIBOOT_SEARCH: u32 = 8192;
const MULTIBOOT_HEADER_ALIGN: u32 = 4;
const MULTIBOOT_HEADER_MAGIC: u32 = 0x1BADB002;
const MULTIBOOT_BOOTLOADER_MAGIC: u32 = 0x2BADB002;
const MULTIBOOT_MOD_ALIGN: u32 = 0x00001000;
const MULTIBOOT_INFO_ALIGN: u32 = 0x00000004;

// Flags set in the ’flags’ member of the multiboot header.
const MULTIBOOT_PAGE_ALIGN: u32 = 0x00000001;
const MULTIBOOT_MEMORY_INFO: u32 = 0x00000002;
const MULTIBOOT_VIDEO_MODE: u32 = 0x00000004;
const MULTIBOOT_AOUT_KLUDGE: u32 = 0x00010000;

// Flags to be set in the ’flags’ member of the multiboot info structure.
const MULTIBOOT_INFO_MEMORY: u32 = 0x00000001;
const MULTIBOOT_INFO_BOOTDEV: u32 = 0x00000002;
const MULTIBOOT_INFO_CMDLINE: u32 = 0x00000004;
const MULTIBOOT_INFO_MODS: u32 = 0x00000008;
const MULTIBOOT_INFO_AOUT_SYMS: u32 = 0x00000010;
const MULTIBOOT_INFO_ELF_SHDR: u32 = 0x00000020;
const MULTIBOOT_INFO_MEM_MAP: u32 = 0x00000040;
const MULTIBOOT_INFO_DRIVE_INFO: u32 = 0x00000080;
const MULTIBOOT_INFO_CONFIG_TABLE: u32 = 0x00000100;
const MULTIBOOT_INFO_BOOT_LOADER_NAME: u32 = 0x00000200;
const MULTIBOOT_INFO_APM_TABLE: u32 = 0x00000400;
const MULTIBOOT_INFO_VBE_INFO: u32 = 0x00000800;
const MULTIBOOT_INFO_FRAMEBUFFER_INFO: u32 = 0x00001000;

// Type definitions
type MultibootUint8 = u8;
type MultibootUint16 = u16;
type MultibootUint32 = u32;
type MultibootUint64 = u64;

#[repr(C)]
#[derive(Copy, Clone)]
struct MultibootHeader {
    magic: MultibootUint32,
    flags: MultibootUint32,
    checksum: MultibootUint32,
    header_addr: MultibootUint32,
    load_addr: MultibootUint32,
    load_end_addr: MultibootUint32,
    bss_end_addr: MultibootUint32,
    entry_addr: MultibootUint32,
    mode_type: MultibootUint32,
    width: MultibootUint32,
    height: MultibootUint32,
    depth: MultibootUint32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct MultibootSymbolTableAout {
    tabsize: MultibootUint32,
    strsize: MultibootUint32,
    addr: MultibootUint32,
    reserved: MultibootUint32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct MultibootElfSectionHeaderTable {
    num: MultibootUint32,
    size: MultibootUint32,
    addr: MultibootUint32,
    shndx: MultibootUint32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MultibootModule {
    pub mod_start: MultibootUint32,
    pub mod_end: MultibootUint32,
    cmdline: MultibootUint32,
    pad: MultibootUint32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct MultibootColour {
    red: MultibootUint8,
    green: MultibootUint8,
    blue: MultibootUint8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MultibootMemoryMap {
    pub size: MultibootUint32,
    pub addr: MultibootUint64,
    pub len: MultibootUint64,
    r#type: MultibootUint32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct MultibootApmInfo {
    version: MultibootUint16,
    cseg: MultibootUint16,
    offset: MultibootUint32,
    cseg_16: MultibootUint16,
    dseg: MultibootUint16,
    flags: MultibootUint16,
    cseg_len: MultibootUint16,
    cseg_16_len: MultibootUint16,
    dseg_len: MultibootUint16,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MultibootInfo {
    flags: MultibootUint32,
    mem_lower: MultibootUint32,
    mem_upper: MultibootUint32,
    boot_device: MultibootUint32,
    cmdline: MultibootUint32,
    pub mods_count: MultibootUint32,
    pub mods_addr: MultibootUint32,
    u: UnionType, // This could be a problem
    pub mmap_length: MultibootUint32,
    pub mmap_addr: MultibootUint32,
    drives_length: MultibootUint32,
    drives_addr: MultibootUint32,
    config_table: MultibootUint32,
    boot_loader_name: MultibootUint32,
    apm_table: MultibootUint32,
    vbe_control_info: MultibootUint32,
    vbe_mode_info: MultibootUint32,
    vbe_mode: MultibootUint16,
    vbe_interface_seg: MultibootUint16,
    vbe_interface_off: MultibootUint16,
    vbe_interface_len: MultibootUint16,
    framebuffer_addr: MultibootUint64,
    framebuffer_pitch: MultibootUint32,
    framebuffer_width: MultibootUint32,
    framebuffer_height: MultibootUint32,
    framebuffer_bpp: MultibootUint8,
    framebuffer_type: MultibootUint8,
    palette: PaletteType,
}

// impl core::fmt::Debug for MultibootInfo {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         f.debug_struct("PackedStruct")
//             .field("module number", &self.mods_count)
//             .finish()
//     }
// }

#[repr(C)]
#[derive(Copy, Clone)]
union UnionType {
    aout_sym: MultibootSymbolTableAout,
    elf_sec: MultibootElfSectionHeaderTable,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct PaletteType {
    addr: MultibootUint32,
    num_colors: MultibootUint16,
}
