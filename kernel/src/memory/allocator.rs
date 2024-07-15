use core::{mem, panic};

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

const LIST_NODE_MEMORY_SIZE: isize = core::mem::size_of::<ListNode<MemoryBlock>>() as isize;

/*
   +--------+------+-------+
   | Header | Data | Align |
   +--------+------+-------+
*/
#[derive(Clone, Debug, PartialEq)]
struct MemoryBlock {
    size: usize, // Value in bytes which includes size of the list memory block structure itself
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
    _kmalloc(size, true)
}

fn _kmalloc(mut size: usize, should_update_size: bool) -> *mut usize {
    if (should_update_size) {
        /*
           Align block size
           Include size of memory block so when free'd block is useable
           Recursive method but only need to update size once (repeat would be a waste of memory)
        */

        size = align(size + LIST_NODE_MEMORY_SIZE as usize);
    }

    let (index, wrapped_memory_block) = find_first_fit(size);

    match wrapped_memory_block {
        Some(memory_block) => {
            // Remove old memory block from list
            FREE_MEMORY_BLOCK_LIST.lock().remove(index);
            FREE_MEMORY_BLOCK_LIST.free();

            let data_addr = memory_block.data;
            let header_addr = get_header_address(data_addr);

            create_new_memory_block(size, header_addr, false);

            // If block is larger then memory required, split region and add second part to list
            // Must be enough space to actually hold a memory block
            let new_block_size = memory_block.size - size;
            if new_block_size > LIST_NODE_MEMORY_SIZE as usize {
                // Add remaining section of block
                let new_free_header_addr =
                    unsafe { (header_addr as *mut u8).offset(size as isize) as *mut usize };

                let aligned_addr = align(new_free_header_addr as usize);

                create_new_memory_block(memory_block.size - size, new_free_header_addr, true);
            }

            data_addr
        }
        None => {
            // No memory blocks can be found, thus must allocate more memory according to how many bytes needed
            let pages_required = page_frame_allocator::get_number_of_pages(
                page_frame_allocator::round_to_nearest_page(size),
            );

            extend_memory_region(pages_required);

            return _kmalloc(size as usize, false);
        }
    }
}

/*
    Recives pointer to memory address of payload
    Frees a memory region which can later be allocated
*/
pub fn kfree(data_addr: *mut usize) {
    let header_addr = get_header_address(data_addr);
    let node = unsafe { &mut *(header_addr as *mut ListNode<MemoryBlock>) };
    let memory_block = node.payload.clone();

    // Zero the entirety of the data
    unsafe {
        core::ptr::write_bytes(header_addr as *mut u8, 0, memory_block.size);
    }

    // Add block to list of free blocks (to the front)
    FREE_MEMORY_BLOCK_LIST
        .lock()
        .push_back(memory_block, header_addr as usize);
    FREE_MEMORY_BLOCK_LIST.free();

    // let updated_node = unsafe { &mut *(header_addr as *mut ListNode<MemoryBlock>) };

    // Check next node to merge memory regions together to alleviate fragmentation

    // if let Some(prev_node) = updated_node.prev {
    //     // Update the size of the previous node
    //     unsafe {
    //         (*prev_node).payload.size += updated_node.payload.size;
    //     }

    //     // Remove last node
    //     let length = FREE_MEMORY_BLOCK_LIST.lock().length();
    //     FREE_MEMORY_BLOCK_LIST.free();

    //     FREE_MEMORY_BLOCK_LIST.lock().remove_at(length - 1);
    //     FREE_MEMORY_BLOCK_LIST.free();
    // }
}

/*
    For faster memory access, align blocks by machine word (8 for x64)
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
        FREE_MEMORY_BLOCK_LIST.free();
        if memory_block.payload.size > size {
            return (i, Some(memory_block.payload.clone()));
        }
    }
    FREE_MEMORY_BLOCK_LIST.free();
    return (0, None);
}

pub fn print_memory_list() {
    for memory_block in FREE_MEMORY_BLOCK_LIST.lock().into_iter() {
        print_serial!("{:?}\n", memory_block);
    }
    FREE_MEMORY_BLOCK_LIST.free();
}

// Extends accessible memory region of kernel heap by a number of pages (4096 bytes)
pub fn extend_memory_region(pages: usize) {
    // Allocate another page
    let address = PAGE_FRAME_ALLOCATOR.lock().alloc_page_frames(pages);
    PAGE_FRAME_ALLOCATOR.free();

    let size = paging::PAGE_SIZE * pages;
    create_new_memory_block(size, address, true);
}

/*
    Create a new memory block of a certain size
    Recieves size of block and address in which to create a new block
*/
fn create_new_memory_block(size: usize, addr: *mut usize, is_free: bool) -> *mut usize {
    let dp_addr = unsafe { (addr as *mut u8).offset(LIST_NODE_MEMORY_SIZE) } as *mut usize;
    let new_memory_block = MemoryBlock::new(dp_addr, size);

    if is_free {
        // Push to linked list
        FREE_MEMORY_BLOCK_LIST
            .lock()
            .push_back(new_memory_block, addr as usize);
        FREE_MEMORY_BLOCK_LIST.free();
    } else {
        let new_node = unsafe { &mut *(addr as *mut ListNode<MemoryBlock>) };
        new_node.init(new_memory_block, None, None);
    }

    dp_addr
}

/*
    Returns pointer to address of the list node in general
*/
fn get_header_address(dp: *mut usize) -> *mut usize {
    return unsafe { dp.offset(-1 * (LIST_NODE_MEMORY_SIZE / 8) as isize) };
}
