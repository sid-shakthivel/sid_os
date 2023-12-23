/*
    Contains implementations for malloc, free
    Uses a free list allocator, which traverses a list of memory blocks until it finds a block which can fit the size
*/

use super::page_frame_allocator::PAGE_FRAME_ALLOCATOR;
use super::{page_frame_allocator, paging};
use crate::ds::list::{List, ListNode};
use crate::print_serial;
use crate::utils::spinlock::Lock;
use crate::CONSOLE;

// Divide by 8 as usize is 8 bytes and a *mut usize points to 8 bytes
const NODE_MEMORY_BLOCK_SIZE: isize = (core::mem::size_of::<ListNode<MemoryBlock>>() / 8) as isize;

/*
   +--------+------+-------+
   | Header | Data | Align |
   +--------+------+-------+
*/
#[derive(Clone, Debug, PartialEq)]
struct MemoryBlock {
    size: usize,      // Value in bytes
    data: *mut usize, // Pointer to any data which is held within
}

impl MemoryBlock {
    fn new(data: *mut usize, size: usize) -> MemoryBlock {
        MemoryBlock { size, data }
    }
}

static FREE_MEMORY_BLOCK_LIST: Lock<List<MemoryBlock>> = Lock::new(List::<MemoryBlock>::new());

/*
    Recives the size of data in bytes which is to be used
    Returns pointer to data region
*/
pub fn kmalloc(mut size: usize) -> *mut usize {
    // Size must include the size of a memory block (in bytes)
    size += (NODE_MEMORY_BLOCK_SIZE as usize) * 8;

    // Must align block size by 8
    size = align(size);

    let (index, wrapped_memory_block) = find_first_fit(size);

    match wrapped_memory_block {
        Some(memory_block) => {
            // If block is larger then memory required, split region and add parts to list
            if memory_block.size > size {
                // Remove old memory block
                FREE_MEMORY_BLOCK_LIST.lock().remove_at(index);
                FREE_MEMORY_BLOCK_LIST.free();

                // Create new memory block for malloc'd memory
                let mut address = unsafe { get_header_address(memory_block.data) };

                // Adjust size correctly for correct offset
                let size_in_u64 = size / 8;

                let dp = create_new_memory_block(size, address, false);

                // Add remaining section of block
                address = unsafe { address.offset(NODE_MEMORY_BLOCK_SIZE + size_in_u64 as isize) };
                create_new_memory_block(memory_block.size - size, address, true);

                // print_memory_list();

                return dp;
            } else {
                return memory_block.data;
            }
        }
        None => {
            // No memory blocks can be found, thus must allocate more memory according to how many bytes needed
            let pages_required = page_frame_allocator::get_number_of_pages(
                page_frame_allocator::round_to_nearest_page(size),
            );

            extend_memory_region(pages_required);

            // print_serial!("Extended memory region\n");
            // print_memory_list();

            return kmalloc(size);
        }
    }
}

/*
    Recives pointer to memory address
    Frees a memory region which can later be allocated
    Only use this function if we decide to purely use this kmalloc/kfree instead of pfa stuff
    Perhaps check if memories are subsequent
*/
pub fn kfree(dp: *mut usize) {
    let header_address = unsafe { get_header_address(dp) };
    let header = unsafe { &mut *(header_address as *mut ListNode<MemoryBlock>) };

    // Add block to list of free blocks (will be at the end)
    FREE_MEMORY_BLOCK_LIST
        .lock()
        .push_back(header.payload.clone(), header_address as usize);
    FREE_MEMORY_BLOCK_LIST.free();

    // Check next node to merge memory regions together to alleviate fragmentation
    // TODO: Add support for more nodes (prev as well)

    if let Some(next_node) = header.next {
        let next_header = unsafe { &mut *next_node };

        // Get total size of other region and update memory block
        header.payload.size += next_header.payload.size;

        // Remove other region from linked list since updated
        let length = FREE_MEMORY_BLOCK_LIST.lock().length;
        FREE_MEMORY_BLOCK_LIST.free();

        FREE_MEMORY_BLOCK_LIST.lock().remove_at(length - 1);
        FREE_MEMORY_BLOCK_LIST.free();
    }
}

/*
    For faster memory access, blocks should be aligned by machine word (8 for x64)
*/
fn align(size: usize) -> usize {
    ((size as i64 + 7) & (-8)) as usize
}

/*
    Uses First-fit algorithm
    Recieves size to determine whether a block will fit or not
    Returns first memory block which fits the size
*/
fn find_first_fit(size: usize) -> (usize, Option<MemoryBlock>) {
    for (i, memory_block) in FREE_MEMORY_BLOCK_LIST.lock().into_iter().enumerate() {
        if memory_block.unwrap().payload.size > size {
            FREE_MEMORY_BLOCK_LIST.free();
            return (i as usize, Some(memory_block.unwrap().payload.clone()));
        }
    }
    FREE_MEMORY_BLOCK_LIST.free();
    return (0, None);
}

fn print_memory_list() {
    for (i, memory_block) in FREE_MEMORY_BLOCK_LIST.lock().into_iter().enumerate() {
        FREE_MEMORY_BLOCK_LIST.free();
        print_serial!("{} {:?}\n", i, memory_block.unwrap());
    }
    FREE_MEMORY_BLOCK_LIST.free();
}

// Extends accessible memory region of kernel heap by another page (4096 bytes)
pub fn extend_memory_region(pages: usize) {
    // Allocate another page
    let address = PAGE_FRAME_ALLOCATOR.lock().alloc_page_frames(pages);
    PAGE_FRAME_ALLOCATOR.free();

    let size = (paging::PAGE_SIZE) * pages;
    create_new_memory_block(size, address, true);
}

/*
    Create a new memory block of a certain size
    Recieves size of block and address in which to create a new block
*/
fn create_new_memory_block(size: usize, address: *mut usize, is_free: bool) -> *mut usize {
    let dp_addr = unsafe { address.offset(NODE_MEMORY_BLOCK_SIZE) };
    let new_memory_block = MemoryBlock::new(dp_addr, size);

    if is_free {
        // Push to linked list
        FREE_MEMORY_BLOCK_LIST
            .lock()
            .push_back(new_memory_block, dp_addr as usize);
        FREE_MEMORY_BLOCK_LIST.free();
    } else {
        // Add meta data regardless
        unsafe {
            *(address as *mut MemoryBlock) = new_memory_block;
        }
    }

    return dp_addr;
}

/*
    Recives pointer to data
    Returns pointer to address of header
*/
unsafe fn get_header_address(dp: *mut usize) -> *mut usize {
    return dp.offset(-1 * (NODE_MEMORY_BLOCK_SIZE) as isize);
}
