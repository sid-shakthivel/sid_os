use super::list::{List, ListIterator, ListNode};
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
        let addr = kmalloc(core::mem::size_of::<ListNode<PriorityWrapper<T>>>()) as usize;

        self.list.push_back(priority_wrapped_node, addr);
    }

    // pub fn dequeue(&mut self) -> T {
    //     let ret =self
    //         .list
    //         .remove_at(self.list.length)
    //         .expect("Value expected when dequeing")
    // }

    pub fn get_head(&mut self) -> &mut T {
        let value = self.list.head.expect("ERROR: Queue is empty");
        return unsafe { &mut (*value).payload.value };
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
        let addr = kmalloc(core::mem::size_of::<ListNode<T>>()) as usize;
        self.list.push_back(payload, addr);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.list.remove_at(0).map(|node| {
            kfree(node.1);
            node.0
        })
    }

    pub fn empty(&mut self) {
        while self.dequeue().is_some() {}
    }

    pub fn get_element(&mut self, index: usize) -> &mut T {
        return self
            .list
            .get_at(index)
            .expect("ERROR: Queue does not contain element at specified index");
    }

    pub fn get_head(&mut self) -> &mut T {
        let value = self.list.head.expect("ERROR: Queue is empty");
        return unsafe { &mut (*value).payload };
    }
}
