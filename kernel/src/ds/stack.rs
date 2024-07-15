use super::list::{List, ListIterator, ListNode, MutListIterator};
use crate::{
    memory::{
        allocator::{self, kfree, kmalloc, print_memory_list},
        page_frame_allocator::PAGE_FRAME_ALLOCATOR,
    },
    print_serial,
};

#[derive(Debug)]
pub struct Stack<T: 'static> {
    list: List<T>,
}

impl<T: Clone> Stack<T> {
    pub const fn new() -> Stack<T> {
        Stack { list: List::new() }
    }

    // Remove from top of list
    pub fn pop(&mut self) -> Option<T> {
        self.list.remove(0).map(|node| {
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

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        return self
            .list
            .get_mut(index)
            .expect("Undefined node at specifies index");
    }

    pub fn peek(&mut self) -> &mut T {
        let value = self.list.head.expect("ERROR: Stack is empty");
        return unsafe { &mut (*value).payload };
    }

    pub fn remove(&mut self, index: usize) -> Option<(T, *mut usize)> {
        self.list.remove(index)
    }

    pub fn iter_mut(&mut self) -> MutListIterator<T> {
        self.list.iter_mut()
    }

    pub fn iter(&self) -> ListIterator<T> {
        self.list.iter()
    }

    pub fn find_where<F>(&self, func: &F, key: usize) -> Option<usize>
    where
        F: Fn(&T, usize) -> bool,
    {
        for (i, node) in self.list.iter().enumerate() {
            if func(node, key) {
                return Some(i);
            }
        }

        None
    }
}
