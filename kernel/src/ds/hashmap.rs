// src/hashmap.rs

/*
    Data structure which stores elements in key value pairs with very fast access times
    Hash function is used to map data of arbitrary size to fixed size values and these returns hashes
    Collisions occur when hash function generates same index for multiple keys
    Separate chaining is a method in which linked lists are created for items with same hash
*/

use crate::memory::allocator::kmalloc;

use super::queue::Queue;
use core::fmt::Debug;

const CAPACITY: usize = 5;

pub struct HashMap<T: 'static> {
    items: [Option<HashItem<T>>; CAPACITY],
}

#[derive(Copy, Clone, Debug)]
pub struct HashItem<T: 'static> {
    key: usize,
    values: *mut Queue<HashItem<T>>,
}

impl<T: Clone> HashItem<T> {
    pub fn new(key: usize, value: T) -> HashItem<T> {
        let addr = kmalloc(core::mem::size_of::<Queue<HashItem<T>>>()) as *mut Queue<HashItem<T>>;

        HashItem {
            key,
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
        let item = HashItem::new(key, value);

        // Compute the index based on the hash function
        let index = self.hash(key);  

        // Check if the index is already occupied or not
        match self.items[index] {
            Some(mut existing_item) => {


                // match existing_item.list {
                //     Some(mut list) => {
                //         let mut index_to_be_removed = 0;

                //         // If list exists, attempt to find the item and update it accordingly
                //         for (_i, item) in list.into_iter().enumerate() {
                //             if item.unwrap().payload.key == key {
                //                 index_to_be_removed = key;
                //             }
                //         }

                //         if index_to_be_removed != 0 {
                //             list.remove_at(index_to_be_removed);
                //         }

                //         // If cannot find item, add it to the list
                //         list.push(item);
                //     }
                //     None => {
                //         // Check if this item is being updated, and update it
                //         if existing_item.key == key {
                //             self.items[index] = Some(item);
                //             return;
                //         }

                //         // If linked list does not exist, we must create one and add the element to the list
                //         // existing_item.list = Some(Stack::<HashItem<T>>::new());
                //         existing_item.list.unwrap().push(item);
                //     }
                // }
            }
            None => {
                // Set item
                self.items[index] = Some(item);
            }
        }
    }

    // Gets an element from hashmap
    pub fn get(&self, key: usize) -> Option<T> {
        let index = self.hash(key);
        if index > CAPACITY {
            return None;
        }

        match self.items[index] {
            Some(item) => {
                // Check if the item required is the node
                // if item.key == key {
                //     return Some(item.value);
                // }

                // if let Some(list) = item.list {
                //     for (_i, item) in list.into_iter().enumerate() {
                //         let unwrapped = item.unwrap().payload.clone();
                //         if unwrapped.key == key {
                //             return Some(unwrapped.value);
                //         }
                //     }
                // }

                None
            }
            None => None,
        }
    }

    pub fn get_mut_ref(&self, key: usize) -> Option<*mut T> {
        let index = self.hash(key);
        if index > CAPACITY {
            return None;
        }

        match self.items[index] {
            Some(item) => {
                // Check if the item required is the node
                if item.key == key {
                    let const_ptr = &item as *const HashItem<T>;
                    let mut_ptr = const_ptr as *mut HashItem<T>;
                    // let best = unsafe { &mut (*mut_ptr).value as *mut T };

                    // return Some(best);
                }

                // if let Some(list) = item.list {
                //     for (_i, item) in list.into_iter().enumerate() {
                //         let const_ptr = &item.unwrap().payload as *const HashItem<T>;
                //         let mut_ptr = const_ptr as *mut HashItem<T>;

                //         unsafe {
                //             let best = &mut (*mut_ptr).value as *mut T;

                //             if (*mut_ptr).key == key {
                //                 return Some(best);
                //             }
                //         }
                //     }
                // }

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

        if let Some(existing_item) = self.items[index] {
            if existing_item.key == key {
                self.items[index] = None;
                return;
            }

            let mut index_to_be_removed = 0;

            // for (i, item) in existing_item.list.unwrap().into_iter().enumerate() {
            //     let unwrapped = item.unwrap().payload.clone();
            //     if unwrapped.key == key {
            //         index_to_be_removed = i;
            //     }
            // }

            // existing_item.list.unwrap().remove_at(index_to_be_removed);
        }
    }

    fn hash(&self, key: usize) -> usize {
        key % CAPACITY
    }
}
