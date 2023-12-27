use crate::print_serial;
use crate::CONSOLE;

// Each node stores a reference to the next/previous node within the list along with a payload
#[derive(Debug, Copy, Clone)]
pub struct ListNode<T: 'static> {
    pub payload: T,
    pub next: Option<*mut ListNode<T>>,
    pub prev: Option<*mut ListNode<T>>,
}

impl<T> ListNode<T> {
    pub fn init(&mut self, payload: T) {
        self.payload = payload;
        self.prev = None;
        self.next = None;
    }
}

#[derive(Debug)]
pub struct List<T: 'static> {
    pub head: Option<*mut ListNode<T>>,
    pub tail: Option<*mut ListNode<T>>,
    pub length: usize,
}

impl<T> ListNode<T> {
    pub fn get_mut_ref(list_node: Option<*mut ListNode<T>>) -> &'static mut ListNode<T> {
        unsafe {
            list_node
                .expect("ListNode is undefined")
                .as_mut()
                .expect("ListNode is undefined")
        }
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

    // Create a new node at the front of the list
    pub fn push_front(&mut self, payload: T, addr: usize) {
        // Create a new node directly at a specific memory location
        let new_node = unsafe { &mut *(addr as *mut ListNode<T>) };
        new_node.init(payload);

        if self.head.is_some() {
            let head = ListNode::get_mut_ref(self.head);
            new_node.next = Some(head);
            head.prev = Some(new_node);
        }

        self.head = Some(new_node);

        self.length += 1;
    }

    // Create a new node at the end of the list
    pub fn push_back(&mut self, payload: T, addr: usize) {
        // Create a new node directly at a specific memory location
        let new_node = unsafe { &mut *(addr as *mut ListNode<T>) };
        new_node.init(payload);

        // Always adds to the end of the list
        match self.length {
            0 => {
                self.head = Some(new_node);
                self.tail = Some(new_node);
            }
            _ => {
                let tail = ListNode::get_mut_ref(self.tail);

                new_node.prev = self.tail;
                tail.next = Some(new_node);

                self.tail = Some(new_node);
            }
        }

        self.length += 1;
    }

    pub fn remove_at(&mut self, index: usize) -> Option<(T, *mut usize)> {
        if index < 0 || index > self.length {
            panic!("List-Remove: Index Out of Bounds");
        }

        // print_serial!("Removing at index {}\n", index);

        let length = self.length;

        match index {
            0 => unsafe {
                if (self.head.is_some()) {
                    let head = ListNode::get_mut_ref(self.head);
                    let address = self.head.unwrap() as *mut usize;

                    if (head.next.is_some()) {
                        let head_next = ListNode::get_mut_ref(head.next);
                        head_next.prev = None;
                    }

                    self.head = head.next;
                    let payload = head.payload.clone();

                    self.length -= 1;

                    return Some((payload, address));
                }
            },
            length => unsafe {
                if (self.tail.is_some()) {
                    let tail = ListNode::get_mut_ref(self.tail);

                    if (tail.prev.is_some()) {
                        let tail_prev = ListNode::get_mut_ref(tail.prev);
                        tail_prev.next = None;
                    }

                    let address = self.tail.unwrap() as *mut usize;

                    self.tail = tail.prev;

                    let payload = tail.payload.clone();
                    tail.prev = None;

                    self.length -= 1;

                    return Some((payload, address));
                }
            },
            _ => {
                // Implement for any other index (through looping)
                panic!("Must implement this!!\n");
            }
        };

        return None;
    }
}

impl<'a, T> IntoIterator for &'a List<T> {
    type Item = Option<&'a ListNode<T>>;
    type IntoIter = ListIntoIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ListIntoIterator {
            current: match self.head {
                Some(head) => unsafe { Some(&*head) },
                _ => None,
            },
        }
    }
}

/// Iterator for the List
pub struct ListIntoIterator<'a, T: 'static> {
    current: Option<&'a ListNode<T>>,
}

impl<'a, T> Iterator for ListIntoIterator<'a, T> {
    type Item = Option<&'a ListNode<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Some(node) => {
                let saved_current = self.current;

                self.current = match node.next {
                    Some(value) => unsafe { Some(&*value) },
                    None => None,
                };

                return Some(saved_current);
            }
            None => None,
        }
    }
}
