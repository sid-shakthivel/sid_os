/*
Physical memory is split into 4096 byte chunks called page frames
To manage the frames, a stack of free pages along with a pointer to first page are used
*/

use crate::{print_serial, utils::spinlock::Lock, CONSOLE};

const PAGE_SIZE: usize = 4096;

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

    pub fn init(&mut self, mut memory_start: usize, mut memory_end: usize) {
        // TODO: Fix this because literally adding a page purely for safety
        memory_start = round_to_nearest_page(memory_start) + 0x1000;
        memory_end = round_to_nearest_page(memory_end);

        self.memory_start = memory_start + (PAGE_SIZE * 2);
        self.memory_end = memory_end;
        self.free_page_frames =
            unsafe { Some(&mut *((memory_start + PAGE_SIZE) as *mut FreeStack)) };
        self.current_page = memory_start + (PAGE_SIZE * 2);
    }

    /*
       Return a free page if available within the stack
       Else return the address of the current page and increment
    */
    pub fn alloc_page_frame(&mut self) -> Option<*mut usize> {
        if self
            .free_page_frames
            .as_mut()
            .expect("Shouldn't be none")
            .is_empty()
        {
            // Check if over memory limit
            let address = self.current_page;

            if address > self.memory_end {
                return None;
            } else {
                self.current_page += 4096;
                return Some(self.current_page as *mut usize);
            }
        } else {
            match self
                .free_page_frames
                .as_mut()
                .expect("Shouldn't be none")
                .pop()
            {
                Some(page_frame) => unsafe {
                    return Some((*page_frame).get_address() as *mut usize);
                },
                None => None,
            }
        }
    }

    // Add the address of the free'd page to the stack
    pub unsafe fn free_page_frame(&mut self, frame_address: *mut usize) {
        let new_free_frame = unsafe { &mut *(frame_address as *mut PageFrame) };
        self.free_page_frames
            .as_mut()
            .expect("Shouldn't be none")
            .push(new_free_frame);
    }

    // Allocates a continuous amount of pages subsequently
    fn alloc_page_frames(&mut self, pages_required: usize) -> *mut usize {
        let address = self.current_page;
        for _i in 0..pages_required {
            self.current_page += 4096;
        }
        return address as *mut usize;
    }

    // Frees a continuous amount of memory
    fn free_page_frames(&mut self, frame_address: *mut usize, pages_required: usize) {
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

    // TODO: Might want to refactor!?
    pub fn pop(&mut self) -> Option<*mut PageFrame> {
        match self.length {
            0 => None,
            1 => {
                let temp_frame = self.top;
                self.top = None;
                return temp_frame;
            }
            _ => unsafe {
                if let Some(cloned_top) = self.top {
                    self.top = (*cloned_top).next;
                    return Some(cloned_top);
                }

                return None;
            },
        }
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

pub static PAGE_FRAME_ALLOCATOR: Lock<PageFrameAllocator> = Lock::new(PageFrameAllocator::new());
