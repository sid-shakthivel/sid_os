/*
Paging allows mapping between virtual address to a physical address
This allows different processes to use the same address space to different parts of memory
This ensures different processes cannot overwrite memory nor access the memory of different processes or the kernel which ensures safety
Page tables specify which frame an address points to
Order is: Page Map Level Table(P4), Page Directory Pointer Table(P3), Page Directory Table(P2), Page Table(P1)

Recursive mapping sets the last entry of P4 to itself
To access a page table (and edit it), the CPU loops twice through the P4, on second run it acts as a P3, which then points to a P2 which points to a page table entry itself
By modifying the address passed, CPU can access different parts of the paging hierarchy as different tables act as upper tables
*/

/*
Page table entries have a certain 64 bit format which looks like this:
+---------+-----------+------------------+---------------+---------------+-------+-----------+--------+-----------+------------------+-----------+------------+
|    0    |     1     |        2         |       3       |       4       |   6   |     7     |   8    |   9-11    |      12-51       |   52-62   |     63     |
+---------+-----------+------------------+---------------+---------------+-------+-----------+--------+-----------+------------------+-----------+------------+
| present | writable |  user accessible | write through | disable cache | dirty | huge page | global | available | physical address | available | no execute |
+---------+-----------+------------------+---------------+---------------+-------+-----------+--------+-----------+------------------+-----------+------------+
*/

use core::{future::IntoFuture, num};

use crate::{print_serial, CONSOLE};

use super::{allocator::kmalloc, page_frame_allocator::PAGE_FRAME_ALLOCATOR};

pub const PAGE_SIZE: usize = 4096;

pub const P4: *mut PageTable = 0xffffffff_fffff000 as *mut _;

enum PageFlags {
    Present,
    Writable,
    UserAccessible,
    WriteThrough,
    DisableCache,
    Dirty,
    Huge,
    Global,
}

#[repr(C)]
struct Page(usize);

#[repr(C)]
pub struct PageTable {
    pub entries: [Page; 512],
}

impl Page {
    // Used to unmap a page
    pub fn set_to_unused(&mut self) {
        self.0 = 0;
    }

    pub fn is_unused(&mut self) -> bool {
        self.0 == 0
    }

    pub fn new(p_addr: usize, flags: &[PageFlags]) -> Page {
        let mut entry_data: usize = (0x000fffff_fffff000 & p_addr);

        // for flag in flags {
        //     entry_data = match flag {
        //         PageFlags::Present => (1 << 0) | entry_data,
        //         PageFlags::Writable => (1 << 1) | entry_data,
        //         PageFlags::UserAccessible => (1 << 2) | entry_data,
        //         PageFlags::WriteThrough => (1 << 3) | entry_data,
        //         PageFlags::DisableCache => (1 << 4) | entry_data,
        //         PageFlags::Dirty => (1 << 6) | entry_data,
        //         PageFlags::Huge => (1 << 7) | entry_data,
        //         PageFlags::Global => (1 << 8) | entry_data,
        //         _ => entry_data,
        //     };
        // }

        entry_data = entry_data | 0b111;

        Page(entry_data)
    }

    pub fn get_physical_address(&self) -> usize {
        return 0x000fffff_fffff000 & self.0;
    }
}

impl PageTable {
    // Map a virtual address to a physical address
    fn map_recursive(&mut self, v_addr: usize, p_addr: usize, level: usize) {
        // Set default flags for all
        let flags = [
            PageFlags::Present,
            PageFlags::Writable,
            PageFlags::UserAccessible,
        ];

        if level == 0 {
            // Base case: Map the virtual address to the physical address in the P1 table
            let p1_index = (v_addr >> 12) & 0x1ff;
            self.entries[p1_index] = Page::new(p_addr, &flags);
        } else {
            let index = (v_addr >> (level * 9 + 12)) & 0x1FF;

            if self.entries[index].is_unused() {
                // Create a new page table
                let pf_addr = PAGE_FRAME_ALLOCATOR
                    .lock()
                    .alloc_page_frame()
                    .expect("PFA Ran out of memory") as usize;
                PAGE_FRAME_ALLOCATOR.free();

                // let pf_addr: usize = kmalloc(super::paging::PAGE_SIZE) as usize;

                self.entries[index] = Page::new(pf_addr, &flags)
            }

            let next_level_table =
                unsafe { &mut *((self.entries[index].0 & 0xFFFF_FFFF_F000) as *mut PageTable) };

            next_level_table.map_recursive(v_addr, p_addr, level - 1);
        }
    }

    fn unmap_recursive(&mut self, v_addr: usize, level: usize) {
        if (level == 0) {
            let p1_index = (v_addr >> 12) & 0x1FF;
            self.entries[p1_index].set_to_unused();
        } else {
            let index = (v_addr >> (level * 9 + 12)) & 0x1FF;

            self.entries[index].set_to_unused();

            let next_level_table =
                unsafe { &mut *((self.entries[index].0 & 0xFFFF_FFFF_F000) as *mut PageTable) };

            next_level_table.unmap_recursive(v_addr, level - 1);

            self.drop();
        }
    }

    // Frees a table if it becomes unused
    fn drop(&mut self) {
        let mut count: usize = 0;

        for i in 0..512 {
            if self.entries[0].is_unused() {
                count += 1;
            }
        }

        if (count == 512) {
            unsafe {
                let p_addr = self as *const _ as *mut usize;
                PAGE_FRAME_ALLOCATOR.lock().free_page_frame(p_addr);
                PAGE_FRAME_ALLOCATOR.free();
            }
        }
    }

    fn umap(&mut self, v_addr: usize) {
        self.unmap_recursive(v_addr, 3);
    }

    fn map(&mut self, v_addr: usize, p_addr: usize) {
        self.map_recursive(v_addr, p_addr, 3); // Level starts at 3 as 0..3
    }

    fn map_pages(&mut self, number_of_pages: usize, v_addr: usize, p_addr: usize) {
        for i in 0..number_of_pages {
            let p_addr_mod = p_addr + (i * PAGE_SIZE);
            let v_addr_mod = v_addr + (i * PAGE_SIZE);
            self.map_recursive(v_addr_mod, p_addr_mod, 3);
        }
    }
}

pub fn map_pages(number_of_pages: usize, v_addr: usize, p_addr: usize) {
    unsafe {
        (*P4).map_pages(number_of_pages, v_addr, p_addr);
        flush_tlb();
    }
}

pub fn map_page(v_addr: usize, p_addr: usize, is_user: bool) {
    // map_pages(1, v_addr, p_addr);

    unsafe {
        (*P4).map(v_addr, p_addr);
        flush_tlb();
    }
}

extern "C" {
    fn flush_tlb();
}
