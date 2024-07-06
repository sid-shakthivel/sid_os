use super::list::{List, ListNode};
use crate::{
    memory::{
        allocator::{self, kfree, kmalloc, print_memory_list},
        page_frame_allocator::PAGE_FRAME_ALLOCATOR,
    },
    print_serial,
};

#[derive(Debug)]
pub struct Stack<T: 'static> {
    pub list: List<T>,
}

impl<T: Clone> Stack<T> {
    pub const fn new() -> Stack<T> {
        Stack { list: List::new() }
    }

    // Remove from top of list
    pub fn pop(&mut self) -> Option<T> {
        self.list.remove_at(0).map(|node| {
            kfree(node.1);
            node.0
        })
    }

    pub fn empty(&mut self) {
        while self.pop().is_some() {}
    }

    // Add to top of list
    pub fn push(&mut self, payload: T) {
        let addr = kmalloc(core::mem::size_of::<ListNode<T>>()) as usize;
        self.list.push_front(payload, addr);
    }

    pub fn get_at(&mut self, index: usize) -> &mut T {
        return self
            .list
            .get_at(index)
            .expect("Undefined node at specifies index");
    }

    pub fn peek(&mut self) -> &mut T {
        let value = self.list.head.expect("ERROR: Stack is empty");
        return unsafe { &mut (*value).payload };
    }
}
