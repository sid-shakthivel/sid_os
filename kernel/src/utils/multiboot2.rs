use crate::print_serial;
use crate::CONSOLE;

pub const MULTIBOOT2_BOOTLOADER_MAGIC: usize = 0x36d76289;

#[repr(u32)]
pub enum MultibootTagType {
    End = 0, // Marks end of tags
    Module = 3,
    MemoryMap = 6,
    Framebuffer = 8,
}

#[repr(u32)]
pub enum MemoryAreaType {
    Available = 1, // Available memory
    Reserved = 2,  // Memory which shouldn't be used
}

#[repr(C)]
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
pub struct MultibootTagModule {
    pub ty: u32,
    pub size: u32,
    pub mod_start: u32,
    pub mod_end: u32,
    pub cmdline: [char; 0],
}

#[repr(C)]
pub struct MultibootMmapEntry {
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
    pub entries: *const MultibootMmapEntry,
}

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

    pub fn get_size(&self) -> usize {
        self.total_size as usize
    }

    pub fn get_tag_address(&self) -> *const MultibootTag {
        (self.start_address() + 8) as *const MultibootTag
    }

    pub fn get_module_tags(&self) -> ModuleTagIter {
        let tags_iter = self.tags();
        let module_tags_iter = ModuleTagIter::new(tags_iter);
        return module_tags_iter;
    }

    pub fn get_memory_map_tag(&self) -> Option<&MultibootTagMmap> {
        self.tags()
            .find(|tag| unsafe { (**tag).typ == MultibootTagType::MemoryMap as u32 })
            .map(|tag| unsafe { &*(tag as *const MultibootTagMmap) })
    }

    pub fn tags(&self) -> TagIter {
        TagIter::new(self.get_tag_address())
    }
}

impl MultibootTagMmap {
    pub fn get_available_mmap_entries(&self) -> impl Iterator<Item = *const MultibootMmapEntry> {
        let address = self as *const MultibootTagMmap as usize;
        let mmap_iter = MmapIter::new(address);
        let test = mmap_iter
            .filter(|mmap_entry| unsafe { (**mmap_entry).typ == MemoryAreaType::Available as u32 });
        return test;
    }
}

pub fn load(multiboot_info_addr: usize, magic: usize) -> &'static MultibootBootInfo {
    assert!(
        magic == MULTIBOOT2_BOOTLOADER_MAGIC,
        "Multiboot2: Magic values do not match"
    );

    assert!(!(multiboot_info_addr & 7 > 1), "Multiboot2: Unaligned MBI");

    unsafe { &*(multiboot_info_addr as *const MultibootBootInfo) }
}

pub struct MmapIter {
    end_address: usize,
    current_mmap_entry: *const MultibootMmapEntry,
}

impl MmapIter {
    pub fn new(mmap_tag_address: usize) -> MmapIter {
        let tag_mmap = unsafe { &*(mmap_tag_address as *const MultibootTagMmap) };
        let mut first_mmap_entry = ((mmap_tag_address + 16) as *const MultibootMmapEntry);

        MmapIter {
            end_address: mmap_tag_address + (tag_mmap.size as usize),
            current_mmap_entry: first_mmap_entry,
        }
    }
}

impl Iterator for MmapIter {
    type Item = *const MultibootMmapEntry;

    fn next(&mut self) -> Option<*const MultibootMmapEntry> {
        if (self.current_mmap_entry as usize) < self.end_address {
            unsafe {
                self.current_mmap_entry = {
                    (self.current_mmap_entry as *const u8)
                        .add(core::mem::size_of::<MultibootMmapEntry>())
                        .cast::<MultibootMmapEntry>()
                };
            }

            return Some(self.current_mmap_entry);
        } else {
            return None;
        }
    }
}

pub struct ModuleTagIter {
    iter: TagIter,
}

impl ModuleTagIter {
    pub fn new(iter: TagIter) -> ModuleTagIter {
        ModuleTagIter { iter }
    }
}

impl Iterator for ModuleTagIter {
    type Item = *const MultibootTagModule;

    fn next(&mut self) -> Option<*const MultibootTagModule> {
        let test = self
            .iter
            .find(|tag| unsafe { (**tag).typ == MultibootTagType::Module as u32 })
            .map(|tag| unsafe { (tag as *const MultibootTagModule) });

        return test;
    }
}

#[derive(Clone, Debug)]
pub struct TagIter {
    pub current: *const MultibootTag,
}

impl TagIter {
    // Creates a new iterator
    pub fn new(tags: *const MultibootTag) -> TagIter {
        assert_eq!(tags.align_offset(8), 0);
        // let first_tag = &*tags;
        TagIter { current: tags }
    }
}

impl Iterator for TagIter {
    type Item = *const MultibootTag;

    fn next(&mut self) -> Option<*const MultibootTag> {
        let tag = unsafe { &*self.current };

        match tag.typ {
            0 => None, // end tag
            _ => {
                let ptr_offset = (tag.size as usize + 7) & !7;

                self.current = unsafe {
                    self.current
                        .cast::<u8>()
                        .add(ptr_offset)
                        .cast::<MultibootTag>()
                };

                Some(tag)
            }
        }
    }
}
