const DEFAULT_SIZE: usize = 50;

use crate::memory::allocator::{kfree, kmalloc};

pub struct DynamicArray<T> {
    data: *mut T,
    capacity: usize, // Maximum capacity
    length: usize,   // Current length
}

impl<T> DynamicArray<T> {
    pub fn new() -> DynamicArray<T> {
        DynamicArray::<T> {
            data: kmalloc(DynamicArray::<T>::calculate_capacity(DEFAULT_SIZE)) as *mut T,
            capacity: DEFAULT_SIZE,
            length: 0,
        }
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

    pub fn empty(&mut self) {
        unsafe {
            core::ptr::write_bytes(
                self.data as *mut u8,
                0,
                DynamicArray::<T>::calculate_capacity(self.length),
            );
        }
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn length(&self) -> usize {
        self.length
    }

    fn calculate_capacity(size: usize) -> usize {
        core::mem::size_of::<T>() * size
    }

    pub fn get(&self, index: usize) -> Option<&mut T> {
        if index < self.length {
            unsafe { Some(&mut *self.data.add(index)) }
        } else {
            None
        }
    }
}

