use super::list::{List, ListNode};
use crate::memory::allocator::{self, kmalloc};

#[derive(Debug)]
pub struct Stack<T: 'static> {
    pub list: List<T>,
}

impl<T: Clone> Stack<T> {
    pub const fn new() -> Stack<T> {
        Stack { list: List::new() }
    }

    // Remove from top of list
    pub fn pop(&mut self) -> T {
        self.list.remove_at(0).expect("Value expected when popping")
    }

    // Add to top of list
    pub fn push(&mut self, payload: T) {
        let addr = kmalloc(core::mem::size_of::<ListNode<T>>()) as usize;
        self.list.push_front(payload, addr);
    }

    /*
       Purely for page frame allocator
       kmalloc relies on pfa and thus can't be for it
       Free stack is used
    */
    pub fn push_at_addr(&mut self, payload: T, addr: usize) {
        self.list.push_front(payload, addr);
    }
}
