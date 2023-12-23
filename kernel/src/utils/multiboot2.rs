use crate::print_serial;
use crate::CONSOLE;

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
