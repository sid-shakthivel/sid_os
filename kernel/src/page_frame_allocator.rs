/*
Physical memory is split into 4096 byte chunks called page frames
To manage the frames, a stack of free pages along with a pointer to first page are used
*/

const PAGE_SIZE: usize = 4096;

pub struct PageFrame {
    pub next: Option<*mut PageFrame>,
}

pub struct FreeStack {
    pub top: Option<*mut PageFrame>,
    pub length: usize,
}

pub struct PageFrameAllocator {
    memory_start: usize,
    memory_end: usize,
    pub free_page_frames: &'static mut FreeStack,
    pub current_page: *mut usize,
}

impl PageFrame {
    pub fn get_address(&self) -> usize {
        return self as *const PageFrame as usize;
    }
}

impl PageFrameAllocator {
    pub fn new(memory_start: usize, memory_end: usize) -> PageFrameAllocator {
        let page_frame_allocator = PageFrameAllocator {
            memory_start: memory_start + (PAGE_SIZE * 2),
            memory_end,
            free_page_frames: unsafe { &mut *((memory_start + PAGE_SIZE) as *mut FreeStack) },
            current_page: unsafe { &mut *((memory_start + (PAGE_SIZE * 2)) as *mut usize) },
        };

        return page_frame_allocator;
    }

    /*
       Return a free page if available within the stack
       Else return the address of the current page and increment
    */
    pub fn alloc_page(&mut self) -> Option<*mut usize> {
        if self.free_page_frames.is_empty() {
            // Check if over memory limit
            if (self.current_page + PAGE_SIZE) > self.memory_end {
                return None;
            } else {
                return Some(self.current_page as *mut usize);
            }
        } else {
            match self.free_page_frames.pop() {
                Some(page_frame) => unsafe {
                    return Some((*page_frame).get_address() as *mut usize);
                },
                None => None,
            }
        }
    }

    // Add the address of the free'd page to the stack
    pub fn free_page(&mut self, frame_address: *mut usize) {
        let new_free_frame = unsafe { &mut *(frame_address as *mut PageFrame) };
        self.free_frames.push(new_free_frame);
    }
}

impl FreeStack {
    pub fn initalise(&mut self) {
        self.top = None;
        self.length = 0;
    }

    pub fn is_empty(&self) -> bool {
        return self.length > 0;
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
