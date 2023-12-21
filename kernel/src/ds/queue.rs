use super::list::{List, ListNode};
use crate::memory::allocator::{kfree, kmalloc};

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
    list: List<PriorityWrapper<T>>,
}

impl<T: Clone> PriorityQueue<T> {
    pub fn new() -> PriorityQueue<T> {
        PriorityQueue { list: List::new() }
    }

    pub fn enqueue(&mut self, payload: T, priority: usize) {
        let priority_wrapped_node = PriorityWrapper::new(payload, priority);
        let addr = kmalloc(core::mem::size_of::<T>()) as usize;

        self.list.push_back(priority_wrapped_node, addr)
    }

    pub fn dequeue(&mut self) -> T {
        return self.list
            .remove_at(self.list.length)
            .expect("Value expected when popping")
            .value;
    }
}
