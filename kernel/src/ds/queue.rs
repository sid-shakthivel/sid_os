use super::list::{List, ListIntoIterator, ListNode};
use crate::memory::{
    allocator::{kfree, kmalloc},
    page_frame_allocator::PAGE_FRAME_ALLOCATOR,
    paging::PAGE_SIZE,
};

// A wrapper to add priority to any type
#[derive(Clone, Copy)]
struct PriorityWrapper<T> {
    priority: usize,
    value: T,
}

impl<T> PriorityWrapper<T> {
    pub fn new(payload: T, priority: usize) -> PriorityWrapper<T> {
        PriorityWrapper {
            priority,
            value: payload,
        }
    }
}

pub struct PriorityQueue<T: 'static> {
    pub list: List<PriorityWrapper<T>>,
}

impl<T: Clone> PriorityQueue<T> {
    pub const fn new() -> PriorityQueue<T> {
        PriorityQueue { list: List::new() }
    }

    pub fn enqueue(&mut self, payload: T, priority: usize) {
        let priority_wrapped_node = PriorityWrapper::new(payload, priority);
        let addr = kmalloc(core::mem::size_of::<PriorityWrapper<T>>()) as usize;

        self.list.push_back(priority_wrapped_node, addr);
    }

    // pub fn dequeue(&mut self) -> T {
    //     let ret =self
    //         .list
    //         .remove_at(self.list.length)
    //         .expect("Value expected when dequeing")
    // }

    pub fn get_head(&mut self) -> &mut T {
        let value = self.list.head.expect("Head is undefined");
        let best = unsafe { &mut (*value).payload.value };
        return best;
    }
}

pub struct Queue<T: 'static> {
    pub list: List<T>,
}

impl<T: Clone> Queue<T> {
    pub const fn new() -> Queue<T> {
        Queue { list: List::new() }
    }

    pub fn enqueue(&mut self, payload: T) {
        // let addr = kmalloc(core::mem::size_of::<ListNode<T>>()) as usize;
        let addr = PAGE_FRAME_ALLOCATOR.lock().alloc_page_frame().unwrap() as usize;
        PAGE_FRAME_ALLOCATOR.free();
        self.list.push_back(payload, addr);
    }

    pub fn dequeue(&mut self) -> T {
        let ret = self
            .list
            .remove_at(self.list.length - 1)
            .expect("Value expected when popping");
        // kfree(ret.1);
        ret.0
    }

    pub fn empty(&mut self) {
        while self.list.head.is_some() {
            self.dequeue();
        }
    }

    pub fn get_element(&mut self, target_index: usize) -> Option<(usize, T)> {
        for (index, node) in self.list.into_iter().enumerate() {
            if (index == target_index) {
                return Some((index, node.unwrap().payload.clone()));
            }
        }
        None
    }

    pub fn get_head(&mut self) -> &mut T {
        let value = self.list.head.expect("Head is undefined");
        let best = unsafe { &mut (*value).payload };
        return best;
    }
}
