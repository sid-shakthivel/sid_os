use core::panic;

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
const TEST_BLOCK_SIZE: isize = (core::mem::size_of::<MemoryBlock>() / 8) as isize;

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
    _kmalloc(size, true)
}

pub static mut counter: u32 = 0;

fn _kmalloc(mut size: usize, should_update_size: bool) -> *mut usize {
    if (should_update_size) {
        // Size must include the size of a memory block (in bytes)
        size += (NODE_MEMORY_BLOCK_SIZE as usize) * 8;

        // Must align block size by 8
        size = align(size);

        print_serial!("Final size is {}\n", size);
    }

    let (index, wrapped_memory_block) = find_first_fit(size);

    match wrapped_memory_block {
        Some(memory_block) => {
            // If block is larger then memory required, split region and add parts to list
            if memory_block.size > size {
                // Remove old memory block
                FREE_MEMORY_BLOCK_LIST.lock().remove_at(index);
                FREE_MEMORY_BLOCK_LIST.free();

                // Create new memory block for malloc'd memory
                let mut address_of_header = get_header_address(memory_block.data);
                let mut address_of_node = get_base_address(memory_block.data);

                let header = unsafe { &mut *(address_of_header as *mut MemoryBlock) };

                // Adjust size correctly for correct offset
                let size_in_u64 = size / 8;

                let dp = create_new_memory_block(size, address_of_node, false);

                // Add remaining section of block
                address_of_node =
                    unsafe { address_of_node.offset(NODE_MEMORY_BLOCK_SIZE as isize) };
                create_new_memory_block(memory_block.size - size, address_of_node, true);

                unsafe {
                    if counter == 1 {
                        // panic!("oh no {:?}\n", memory_block);
                    }

                    counter += 1;
                }

                return dp;
            } else {
                // TODO: Check this soon
                FREE_MEMORY_BLOCK_LIST.lock().remove_at(index);
                FREE_MEMORY_BLOCK_LIST.free();

                let mut address_of_node = get_base_address(memory_block.data);
                let size_in_u64 = size / 8;

                let dp = create_new_memory_block(size, address_of_node, false);

                return dp;
            }
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
    Only use this function if we decide to purely use this kmalloc/kfree instead of pfa stuff
    Perhaps check if memories are subsequent
*/
pub fn kfree(dp: *mut usize) {
    let node_address = get_base_address(dp);
    let header_address = get_header_address(dp);

    let header = unsafe { &mut *(header_address as *mut MemoryBlock) };

    print_serial!("0x{:x} {:?}\n", header_address as usize, header);

    let size_in_u64 = header.size / 8;

    // Need to zero the data for safety
    // for i in 0..size_in_u64 {
    //     unsafe {
    //         *header_address.offset(i as isize) = 0;
    //     }
    // }

    // Add block to list of free blocks (to the front)
    FREE_MEMORY_BLOCK_LIST
        .lock()
        .push_front(header.clone(), node_address as usize);
    FREE_MEMORY_BLOCK_LIST.free();

    // Check next node to merge memory regions together to alleviate fragmentation
    // TODO: Add support for more nodes (prev as well)

    // if let Some(next_node) = header.next {
    //     let next_header = unsafe { &mut *next_node };

    //     // Get total size of other region and update memory block
    //     header.payload.size += next_header.payload.size;

    //     // Remove other region from linked list since updated
    //     let length = FREE_MEMORY_BLOCK_LIST.lock().length;
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
        if memory_block.unwrap().payload.size >= size {
            FREE_MEMORY_BLOCK_LIST.free();
            return (i as usize, Some(memory_block.unwrap().payload.clone()));
        }
    }
    FREE_MEMORY_BLOCK_LIST.free();
    return (0, None);
}

pub fn print_memory_list() {
    for (i, memory_block) in FREE_MEMORY_BLOCK_LIST.lock().into_iter().enumerate() {
        FREE_MEMORY_BLOCK_LIST.free();
        let test = memory_block.unwrap();
        print_serial!("{} {:?}\n", i, test);
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
    print_serial!("Creating a new memory block at 0x{:x}\n", address as usize);
    let dp_addr = unsafe { address.offset(NODE_MEMORY_BLOCK_SIZE) };
    let new_memory_block = MemoryBlock::new(dp_addr, size);

    if is_free {
        // Push to linked list
        FREE_MEMORY_BLOCK_LIST
            .lock()
            .push_back(new_memory_block, address as usize);
        FREE_MEMORY_BLOCK_LIST.free();
    } else {
        // Add meta data regardless
        unsafe {
            // *(address as *mut MemoryBlock) = new_memory_block;
            let new_node = unsafe { &mut *(address as *mut ListNode<MemoryBlock>) };
            new_node.init(new_memory_block);
        }
    }

    return dp_addr;
}

/*
    Returns pointer to address of the list node in general
*/
fn get_base_address(dp: *mut usize) -> *mut usize {
    return unsafe { dp.offset(-1 * (NODE_MEMORY_BLOCK_SIZE) as isize) };
}

/*
    Returns pointer to the address of the header in particular
*/
fn get_header_address(dp: *mut usize) -> *mut usize {
    return unsafe { dp.offset(-1 * (TEST_BLOCK_SIZE) as isize) };
}
