/*
Physical memory is split into 4096 byte chunks called page frames
To manage the frames, a stack of free pages along with a pointer to first page are used
*/

use crate::ds::stack;
use crate::utils::multiboot2::MultibootBootInfo;
use crate::{print_serial, utils::spinlock::Lock, CONSOLE};

use super::paging::{self, PAGE_SIZE};

#[derive(Debug)]
pub struct PageFrame {
    pub next: Option<*mut PageFrame>,
}

#[derive(Debug)]
pub struct FreeStack {
    pub top: Option<*mut PageFrame>,
    pub length: usize,
}

#[derive(Debug)]
pub struct PageFrameAllocator {
    memory_start: usize,
    memory_end: usize,
    pub free_page_frames: Option<&'static mut FreeStack>,
    pub current_page: usize,
}

impl PageFrame {
    pub fn get_address(&self) -> usize {
        return self as *const PageFrame as usize;
    }
}

impl PageFrameAllocator {
    pub const fn new() -> PageFrameAllocator {
        PageFrameAllocator {
            memory_start: 0,
            memory_end: 0,
            free_page_frames: None,
            current_page: 0,
        }
    }

    pub fn init(&mut self, multiboot_info: &MultibootBootInfo) {
        self.memory_start = round_to_nearest_page(multiboot_info.start_of_useable_memory());
        self.memory_end = round_to_nearest_page(multiboot_info.end_of_useable_memory());

        self.free_page_frames = unsafe { Some(&mut *(self.memory_start as *mut FreeStack)) };
        self.current_page = self.memory_start + paging::PAGE_SIZE;
    }

    /*
       Return a free page if available within the stack
       Else return the address of the current page and increment
    */
    pub fn alloc_page_frame(&mut self) -> Option<*mut usize> {
        if let Some(free_frames) = self.free_page_frames.as_mut() {
            if let Some(page_frame) = free_frames.pop() {
                unsafe {
                    return Some((*page_frame).get_address() as *mut usize);
                }
            } else if (self.current_page < self.memory_end) {
                self.current_page += paging::PAGE_SIZE;
                return Some(self.current_page as *mut usize);
            }
        }

        None
    }

    // Add the address of the free'd page to the stack
    pub unsafe fn free_page_frame(&mut self, frame_address: *mut usize) {
        // Need to zero out the page for safety
        unsafe {
            core::ptr::write_bytes(frame_address as *mut u8, 0, PAGE_SIZE);
        }

        let new_free_frame = unsafe { &mut *(frame_address as *mut PageFrame) };

        self.free_page_frames
            .as_mut()
            .expect("Shouldn't be none")
            .push(new_free_frame);
    }

    // Allocates a continuous amount of pages sequentially
    pub fn alloc_page_frames(&mut self, pages_required: usize) -> *mut usize {
        let address = self.current_page + paging::PAGE_SIZE;
        for _i in 0..pages_required {
            self.current_page += paging::PAGE_SIZE;
        }
        return address as *mut usize;
    }

    // Frees a continuous amount of memory
    pub fn free_page_frames(&mut self, frame_address: *mut usize, pages_required: usize) {
        for i in 0..pages_required {
            unsafe { self.free_page_frame(frame_address.offset(i as isize)) }
        }
    }
}

impl FreeStack {
    pub fn initalise(&mut self) {
        self.top = None;
        self.length = 0;
    }

    pub fn is_empty(&self) -> bool {
        return self.length == 0;
    }

    pub fn pop(&mut self) -> Option<*mut PageFrame> {
        if let Some(cloned_top) = self.top {
            unsafe {
                self.top = (*cloned_top).next;
            }
            return Some(cloned_top);
        }

        None
    }

    pub unsafe fn push(&mut self, node: *mut PageFrame) {
        (*node).next = self.top;
        self.top = Some(node);
        self.length += 1;
    }
}

pub fn round_to_nearest_page(size: usize) -> usize {
    ((size as i64 + 4095) & (-4096)) as usize
}

pub fn get_number_of_pages(size: usize) -> usize {
    size / (paging::PAGE_SIZE as usize)
}

pub static PAGE_FRAME_ALLOCATOR: Lock<PageFrameAllocator> = Lock::new(PageFrameAllocator::new());
