// src/hashmap.rs

/*
    Data structure which stores elements in key value pairs with very fast access times
    Hash function is used to map data of arbitrary size to fixed size values and these returns hashes
    Collisions occur when hash function generates same index for multiple keys
    Separate chaining is a method in which linked lists are created for items with same hash
*/

use crate::{ds::queue, fs::vfs::File, memory::allocator::kmalloc, print_serial};

use super::{list::ListNode, queue::Queue};

use core::mem::size_of;

const CAPACITY: usize = 10;

#[derive(Copy, Clone, Debug)]
pub struct HashMap<T: 'static> {
    items: [Option<HashItem<T>>; CAPACITY],
}

#[derive(Copy, Clone, Debug)]
pub struct HashItem<T: 'static> {
    key: usize,
    value: T,
    values: *mut Queue<HashItem<T>>,
}

fn find_item<T>(node: &ListNode<HashItem<T>>, key: usize) -> bool {
    let hashitem = &node.payload;
    return hashitem.key == key;
}

impl<T: Clone> HashItem<T> {
    pub fn new(key: usize, value: T) -> HashItem<T> {
        let addr = kmalloc(core::mem::size_of::<Queue<HashItem<T>>>()) as *mut Queue<HashItem<T>>;

        HashItem {
            key,
            value,
            values: addr,
        }
    }
}

impl<T: Copy> HashMap<T> {
    pub fn new() -> HashMap<T> {
        HashMap {
            items: [None; CAPACITY],
        }
    }

    // Insert a new key value pair into the hashmap
    pub fn set(&mut self, key: usize, value: T) {
        // Create item based on the key value pair
        let new_item = HashItem::new(key, value);

        // Compute the index based on the hash function
        let index = self.hash(key);

        if index > CAPACITY {
            return;
        }

        // Check if the index is already occupied or not
        match self.items[index] {
            Some(existing_item) => {
                let queue = unsafe { &mut *existing_item.values };

                if let Some(queue_index) = queue.find_where(&find_item, key) {
                    queue.list.remove_at(queue_index);
                }

                queue.enqueue(new_item);
            }
            None => {
                self.items[index] = Some(new_item);
            }
        }
    }

    // Gets an element from hashmap
    pub fn get(&self, key: usize) -> Option<T> {
        let index = self.hash(key);
        if index > CAPACITY {
            return None;
        }

        if let Some(hashitem) = self.items[index] {
            if hashitem.key == key {
                return Some(hashitem.value);
            }

            let queue = unsafe { &mut *hashitem.values };

            if let Some(queue_index) = queue.find_where(&find_item, key) {
                return Some(queue.get_element(index).value);
            }
        }

        None
    }

    pub fn get_mut_ref(&self, key: usize) -> Option<*mut T> {
        let index = self.hash(key);
        if index > CAPACITY {
            return None;
        }

        match self.items[index] {
            Some(mut hashitem) => {
                // Check if the item required is the node
                if hashitem.key == key {
                    return Some(&mut hashitem.value as *mut T);
                }

                let queue = unsafe { &mut *hashitem.values };

                if let Some(queue_index) = queue.find_where(&find_item, key) {
                    return Some(&mut queue.get_element(queue_index).value as *mut T);
                }

                None
            }
            None => None,
        }
    }

    // Removes an element from the hashmap
    pub fn delete(&mut self, key: usize) {
        let index = self.hash(key);
        if index > CAPACITY {
            return;
        }

        if let Some(hashitem) = self.items[index] {
            if hashitem.key == key {
                self.items[index] = None;
                return;
            }

            let queue = unsafe { &mut *hashitem.values };

            if let Some(queue_index) = queue.find_where(&find_item, key) {
                queue.list.remove_at(queue_index);
            }
        }
    }

    fn hash(&self, key: usize) -> usize {
        key % CAPACITY
    }
}
