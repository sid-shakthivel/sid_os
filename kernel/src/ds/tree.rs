use super::vec::DynamicArray;
use crate::{memory::allocator::kmalloc, print_serial};

#[derive(Debug, Clone, Copy)]
pub struct TreeNode<T: core::fmt::Debug> {
    pub payload: *mut T,
    pub children: DynamicArray<TreeNode<T>>,
    has_parent: bool,
}

impl<T: core::fmt::Debug> TreeNode<T> {
    pub const fn new_const() -> TreeNode<T> {
        TreeNode {
            payload: core::ptr::null_mut(),
            children: DynamicArray::new(),
            has_parent: false,
        }
    }

    pub fn new(payload: T) -> TreeNode<T> {
        let payload_ptr = kmalloc(core::mem::size_of::<T>()) as *mut T;
        unsafe {
            core::ptr::write(payload_ptr, payload);
        }

        let mut new_node = TreeNode {
            payload: payload_ptr,
            children: DynamicArray::new(),
            has_parent: false,
        };

        new_node.children.init();

        new_node
    }

    pub fn add_child(&mut self, mut child: TreeNode<T>) {
        if !child.has_parent {
            print_serial!("ERROR: Child already has parent");
            return;
        }
        child.has_parent = true;
        self.children.push(child);
    }

    pub fn traverse<F>(&self, func: F)
    where
        F: Fn(&TreeNode<T>),
    {
        func(self);

        for child in self.children.iter() {
            child.traverse(&func);
        }
    }
}
