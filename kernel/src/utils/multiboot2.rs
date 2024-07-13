use crate::print_serial;
use crate::CONSOLE;

extern "C" {
    static __kernel_end: u8;
    static __kernel_start: u8;
}

pub const MULTIBOOT2_BOOTLOADER_MAGIC: usize = 0x36d76289;

#[repr(C)]
pub struct MultibootBootInfo {
    total_size: u32,
    reserved: u32,
}

impl MultibootBootInfo {
    pub fn start_address(&self) -> usize {
        self as *const MultibootBootInfo as usize
    }

    pub fn end_address(&self) -> usize {
        self.start_address() + self.get_size()
    }

    pub fn start_of_useable_memory(&self) -> usize {
        let kernel_end_addr = unsafe { &__kernel_end as *const u8 as usize };
        let multiboot_end_addr = self.end_address();

        let end_module_addr = self
            .get_module_tags()
            .map(|entry| entry.mod_end)
            .max()
            .unwrap_or(0) as usize;

        // print_serial!("kernel end addr {:#X}\n", kernel_end_addr);
        // print_serial!("multiboot end addr {:#X}\n", multiboot_end_addr);
        // print_serial!("module end addr {:#X}\n", end_module_addr);

        assert!(
            end_module_addr > kernel_end_addr,
            "Kernel end addr > End module addr\n"
        );

        // assert!(
        //     end_module_addr > multiboot_end_addr,
        //     "Multiboot adr > End module addr\n"
        // );

        end_module_addr.max(multiboot_end_addr).max(kernel_end_addr)
    }

    pub fn end_of_useable_memory(&self) -> usize {
        // let mut end_memory: usize = 0;
        let mmap_tag = self.get_memory_map_tag().expect("Expected mmap");

        let end_memory = mmap_tag
            .get_available_mmap_entries()
            .into_iter()
            .map(|entry| entry.end_address())
            .max()
            .unwrap_or(0);

        end_memory
    }

    pub fn get_size(&self) -> usize {
        self.total_size as usize
    }

    pub fn get_tag_address(&self) -> *const Tag {
        (self.start_address() + 8) as *const Tag
    }

    pub fn get_module_tags(&self) -> ModuleTagIter {
        ModuleTagIter::new(self.tags())
    }

    pub fn get_memory_map_tag(&self) -> Option<&MmapTag> {
        self.tags()
            .find(|&tag| tag.typ == TagType::MemoryMap as u32)
            .map(|tag| unsafe { &*(tag as *const Tag as *const MmapTag) })
    }

    pub fn get_framebuffer_tag(&self) -> Option<&FramebufferTag> {
        self.tags()
            .find(|&tag| tag.typ == TagType::Framebuffer as u32)
            .map(|tag| unsafe { &*(tag as *const Tag as *const FramebufferTag) })
    }

    pub fn tags(&self) -> TagIter {
        TagIter::new(self.get_tag_address())
    }
}

#[repr(u32)]
pub enum TagType {
    End = 0,            // Marks end of tags
    BootLoaderName = 2, // Bootloader name tag
    Module = 3,         // Module tag
    BasicMemInfo = 4,   // Basic memory information tag
    BootDev = 5,        // Boot device tag
    MemoryMap = 6,      // Memory map tag
    Vbe = 7,            // VBE (Video Display Information) tag
    Framebuffer = 8,    // Framebuffer tag
    AcpiOld = 9,        // ACPI (Advanced Configuration and Power Interface) tag (old)
    AcpiNew = 10,       // ACPI (Advanced Configuration and Power Interface) tag (new)
    Network = 11,       // Network configuration tag
    Efi32 = 12,         // EFI 32-bit tag
    Efi64 = 13,         // EFI 64-bit tag
    Smbios = 14,        // SMBIOS (System Management BIOS) tag
    ElfSections = 15,   // ELF sections tag
    Apm = 16,           // APM (Advanced Power Management) BIOS tag
    Efi32Ih = 17,       // EFI 32-bit real-time clock tag
    Efi64Ih = 18,       // EFI 64-bit real-time clock tag
    Scsi = 19,          // SCSI (Small Computer System Interface) tag
    Edid = 20,          // EDID (Extended Display Identification Data) tag
    EfiMmap = 21,       // EFI memory map tag
    EfiBs = 22,         // EFI boot services tag
    Efi32St = 23,       // EFI 32-bit system table tag
    Efi64St = 24,       // EFI 64-bit system table tag
    LoadBaseAddr = 25,  // Load base address tag
}

#[repr(u32)]
pub enum MemoryAreaType {
    Available = 1, // Available memory
    Reserved = 2,  // Memory which shouldn't be used
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Tag {
    pub typ: u32,
    pub size: u32,
}

#[derive(Clone, Debug)]
pub struct TagIter<'a> {
    pub current: *const Tag,
    marker: core::marker::PhantomData<&'a Tag>,
}

impl<'a> TagIter<'a> {
    // Creates a new iterator
    pub fn new(tags: *const Tag) -> TagIter<'a> {
        assert_eq!(tags.align_offset(8), 0);
        TagIter {
            current: tags,
            marker: core::marker::PhantomData,
        }
    }
}

impl<'a> Iterator for TagIter<'a> {
    type Item = &'a Tag;

    fn next(&mut self) -> Option<&'a Tag> {
        let tag = unsafe { &*self.current };

        match tag.typ {
            0 => None, // end tag
            _ => {
                let ptr_offset = (tag.size as usize + 7) & !7;

                self.current = unsafe { self.current.cast::<u8>().add(ptr_offset).cast::<Tag>() };

                Some(tag)
            }
        }
    }
}

#[repr(C)]
pub struct TagString {
    pub ty: u32,
    pub size: u32,
    pub string: [char; 0],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ModuleTag {
    pub ty: u32,
    pub size: u32,
    pub mod_start: u32,
    pub mod_end: u32,
    pub cmdline: [char; 0],
}

pub struct ModuleTagIter<'a> {
    iter: TagIter<'a>,
}

impl<'a> ModuleTagIter<'a> {
    pub fn new(iter: TagIter) -> ModuleTagIter {
        ModuleTagIter { iter }
    }
}

impl<'a> Iterator for ModuleTagIter<'a> {
    type Item = &'a ModuleTag;

    fn next(&mut self) -> Option<&'a ModuleTag> {
        self.iter
            .find(|&tag| tag.typ == TagType::Module as u32)
            .map(|tag| unsafe { &*(tag as *const Tag as *const ModuleTag) })
    }
}

#[repr(C)]
pub struct MmapEntry {
    pub addr: u64,
    pub len: u64,
    pub typ: u32,
    _reserved: u32,
}

impl MmapEntry {
    pub fn start_address(&self) -> usize {
        self.addr as usize
    }

    pub fn end_address(&self) -> usize {
        (self.addr + self.len) as usize
    }
}

#[repr(C)]
pub struct MmapTag {
    pub ty: u32,
    pub size: u32,
    pub entry_size: u32,
    pub entry_version: u32,
    pub entries: *const MmapEntry,
}

impl MmapTag {
    pub fn get_available_mmap_entries(&self) -> impl Iterator<Item = &MmapEntry> {
        let address = self as *const MmapTag as usize;
        let mmap_iter = MmapIter::new(address);
        mmap_iter.filter(|&mmap_entry| mmap_entry.typ == MemoryAreaType::Available as u32)
    }
}

pub struct MmapIter<'a> {
    end_address: usize,
    current_mmap_entry: *const MmapEntry,
    marker: core::marker::PhantomData<&'a Tag>,
}

impl<'a> MmapIter<'a> {
    pub fn new(mmap_tag_address: usize) -> MmapIter<'a> {
        let tag_mmap = unsafe { &*(mmap_tag_address as *const MmapTag) };
        let mut first_mmap_entry = ((mmap_tag_address + 16) as *const MmapEntry);

        MmapIter {
            end_address: mmap_tag_address + (tag_mmap.size as usize),
            current_mmap_entry: first_mmap_entry,
            marker: core::marker::PhantomData,
        }
    }
}

impl<'a> Iterator for MmapIter<'a> {
    type Item = &'a MmapEntry;

    fn next(&mut self) -> Option<&'a MmapEntry> {
        if (self.current_mmap_entry as usize) < self.end_address {
            self.current_mmap_entry = unsafe {
                self.current_mmap_entry
                    .cast::<u8>()
                    .add(core::mem::size_of::<MmapEntry>())
                    .cast::<MmapEntry>()
            };

            let current_mmap_entry = unsafe { &*(self.current_mmap_entry as *const MmapEntry) };

            Some(current_mmap_entry)
        } else {
            None
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct FramebufferTag {
    typ: u32,
    size: u32,
    pub addr: u64,
    pub pitch: u32,
    pub width: u32,
    pub height: u32,
    pub bpp: u8,
    pub fb_type: u8,
    _reserved: u16,
}

pub fn load(multiboot_info_addr: usize, magic: usize) -> &'static MultibootBootInfo {
    assert!(
        magic == MULTIBOOT2_BOOTLOADER_MAGIC,
        "Multiboot2: Magic values do not match"
    );

    assert!(!(multiboot_info_addr & 7 > 1), "Multiboot2: Unaligned MBI");

    unsafe { &*(multiboot_info_addr as *const MultibootBootInfo) }
}
