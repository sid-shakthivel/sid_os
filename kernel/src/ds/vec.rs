const DEFAULT_SIZE: usize = 5;

use crate::{
    memory::allocator::{kfree, kmalloc},
    print_serial,
};

#[derive(Debug, Clone, Copy)]
pub struct DynamicArray<T> {
    data: *mut T,
    capacity: usize, // Maximum capacity
    length: usize,   // Current length
}

impl<T> DynamicArray<T> {
    pub const fn new() -> DynamicArray<T> {
        DynamicArray::<T> {
            data: core::ptr::null_mut(),
            capacity: DEFAULT_SIZE,
            length: 0,
        }
    }

    pub fn init(&mut self) {
        self.data = kmalloc(DynamicArray::<T>::calculate_capacity(DEFAULT_SIZE)) as *mut T;
    }

    pub fn push(&mut self, element: T) {
        // Resize array
        if self.length >= self.capacity {
            let new_capacity = self.capacity * 2;

            let new_data = kmalloc(DynamicArray::<T>::calculate_capacity(DEFAULT_SIZE)) as *mut T;

            unsafe {
                core::ptr::copy(new_data, self.data, self.length);
            }

            kfree(self.data as *mut usize);

            self.data = new_data;
            self.capacity = 0;
        }

        unsafe {
            core::ptr::write(self.data.add(self.length), element);
        }

        self.length += 1;
    }

    pub fn swap(&mut self, index1: usize, index2: usize) {
        unsafe {
            let temp = core::ptr::read(self.data.add(index1));
            core::ptr::write(
                self.data.add(index1),
                core::ptr::read(self.data.add(index2)),
            );
            core::ptr::write(self.data.add(index2), temp);
        }
    }

    pub fn empty(&mut self) {
        unsafe {
            core::ptr::write_bytes(
                self.data as *mut u8,
                0,
                DynamicArray::<T>::calculate_capacity(self.length),
            );
        }
    }

    pub fn free(&mut self) {
        kfree(self.data as *mut usize);
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.length > 0 {
            self.length -= 1;
            unsafe { Some(core::ptr::read(self.data.add(self.length))) }
        } else {
            None
        }
    }

    fn calculate_capacity(size: usize) -> usize {
        core::mem::size_of::<T>() * size
    }

    pub fn get_last_mut(&self) -> Option<&mut T> {
        self.get_mut(self.length - 1)
    }

    pub fn get_mut(&self, index: usize) -> Option<&mut T> {
        if index < self.length {
            unsafe { Some(&mut *self.data.add(index)) }
        } else {
            None
        }
    }

    pub fn find_where<F>(&self, func: &F, key: usize) -> Option<usize>
    where
        F: Fn(&T, usize) -> bool,
    {
        for (i, node) in self.iter().enumerate() {
            if func(node, key) {
                return Some(i);
            }
        }

        None
    }

    pub fn iter(&self) -> DynamicArrayIter<'_, T> {
        DynamicArrayIter {
            data: self.data,
            length: self.length,
            index: 0,
            _marker: core::marker::PhantomData,
        }
    }
}

pub struct DynamicArrayIter<'a, T> {
    data: *const T,
    length: usize,
    index: usize,
    _marker: core::marker::PhantomData<&'a T>,
}

impl<'a, T> Iterator for DynamicArrayIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.length {
            let item = unsafe { &*self.data.add(self.index) };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}
