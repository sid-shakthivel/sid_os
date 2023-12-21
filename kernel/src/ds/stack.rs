use super::list::{List, ListNode};
use crate::memory::allocator::{self, kmalloc};

#[derive(Debug)]
pub struct Stack<T: 'static> {
    list: List<T>,
}

impl<T: Clone> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack { list: List::new() }
    }

    // Remove from top of list
    pub fn pop(&mut self) -> T {
        self.list.remove_at(0).expect("Value expected when popping")
    }

    // Add to top of list
    pub fn push(&mut self, payload: T) {
        let addr = kmalloc(core::mem::size_of::<T>()) as usize;
        self.list.push_front(payload, addr);
    }
}
