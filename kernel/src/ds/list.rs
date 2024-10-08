use crate::print_serial;

// Each node stores a reference to the next/previous node within the list along with a payload
#[derive(Debug, Copy, Clone)]
pub struct ListNode<T: 'static> {
    pub payload: T,
    pub next: Option<*mut ListNode<T>>,
    pub prev: Option<*mut ListNode<T>>,
}

impl<T> ListNode<T> {
    pub fn init(
        &mut self,
        payload: T,
        prev: Option<*mut ListNode<T>>,
        next: Option<*mut ListNode<T>>,
    ) {
        self.payload = payload;
        self.prev = prev;
        self.next = next
    }
}

#[derive(Debug, Copy, Clone)]
pub struct List<T: 'static> {
    pub head: Option<*mut ListNode<T>>,
    pub tail: Option<*mut ListNode<T>>,
    length: usize,
}

impl<T> ListNode<T> {
    pub fn get_mut_ref_optional(list_node: Option<*mut ListNode<T>>) -> &'static mut ListNode<T> {
        unsafe {
            list_node
                .expect("ListNode is undefined")
                .as_mut()
                .expect("ListNode is undefined")
        }
    }

    pub fn get_mut_ref(list_node: *mut ListNode<T>) -> &'static mut ListNode<T> {
        unsafe { list_node.as_mut().expect("ListNode is undefined") }
    }
}

impl<T: Clone> List<T> {
    pub const fn new() -> List<T> {
        return List {
            head: None,
            tail: None,
            length: 0,
        };
    }

    pub fn length(&self) -> usize {
        self.length
    }

    // Create a new node at the front of the list
    pub fn push_front(&mut self, payload: T, addr: usize) {
        // Create a new node directly at a specific memory location
        let new_node = unsafe { &mut *(addr as *mut ListNode<T>) };
        new_node.init(payload, None, self.head);

        // Check and set the current head to point to the new head
        if let Some(head) = self.head {
            ListNode::get_mut_ref(head).prev = Some(new_node);
        } else {
            self.tail = Some(new_node);
        }

        // Set the head to the new node
        self.head = Some(new_node);

        self.length += 1;
    }

    // Create a new node at the end of the list
    pub fn push_back(&mut self, payload: T, addr: usize) {
        // Create a new node directly at a specific memory location
        let new_node = unsafe { &mut *(addr as *mut ListNode<T>) };
        new_node.init(payload, self.tail, None);

        // Check and set the current tail to point to the new tail
        if let Some(tail) = self.tail {
            ListNode::get_mut_ref(tail).next = Some(new_node);
        } else {
            self.head = Some(new_node);
        }

        // Set the tail to the new node
        self.tail = Some(new_node);

        self.length += 1;
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let mut current = self.head?;
        let mut count = 0;

        while count < index {
            current = unsafe { (*current).next? };
            count += 1;
        }

        return unsafe { Some(&mut (*current).payload) };
    }

    pub fn remove(&mut self, index: usize) -> Option<(T, *mut usize)> {
        if index >= self.length || index < 0 {
            return None;
        }

        let mut current = self.head?;
        let mut count = 0;

        // Traverse to the node at the specified index
        while count < index {
            current = unsafe { (*current).next? };
            count += 1;
        }

        // Extract the node to be removed
        let node = unsafe { &mut *current };

        // Update the links
        match (node.prev, node.next) {
            (Some(prev), Some(next)) => unsafe {
                (*prev).next = Some(next);
                (*next).prev = Some(prev);
            },
            (Some(prev), None) => {
                unsafe {
                    (*prev).next = None;
                }
                self.tail = node.prev;
            }
            (None, Some(next)) => {
                unsafe {
                    (*next).prev = None;
                }
                self.head = Some(next);
            }
            (None, None) => {
                self.head = None;
                self.tail = None;
            }
        }

        self.length -= 1;

        // Clear the node's links
        node.prev = None;
        node.next = None;

        return Some((node.payload.clone(), current as *mut usize));
    }

    pub fn iter_mut(&self) -> MutListIterator<'_, T> {
        MutListIterator {
            current: self.head,
            _marker: core::marker::PhantomData,
        }
    }

    pub fn iter(&self) -> ListIterator<'_, T> {
        ListIterator {
            current: self.head,
            _marker: core::marker::PhantomData,
        }
    }
}

pub struct ListIterator<'a, T: 'static> {
    current: Option<*mut ListNode<T>>,
    _marker: core::marker::PhantomData<&'a T>,
}

impl<'a, T> Iterator for ListIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.map(|node| {
            self.current = unsafe { (*node).next };
            unsafe { &(*node).payload }
        })
    }
}

pub struct MutListIterator<'a, T: 'static> {
    current: Option<*mut ListNode<T>>,
    _marker: core::marker::PhantomData<&'a T>,
}

impl<'a, T> Iterator for MutListIterator<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.map(|node| {
            self.current = unsafe { (*node).next };
            unsafe { &mut (*node).payload }
        })
    }
}
