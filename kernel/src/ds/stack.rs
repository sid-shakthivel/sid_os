// Each node stores a reference to the next/previous node within the list along with a payload
#[derive(Debug, Copy, Clone)]
pub struct ListNode<T: 'static> {
    pub payload: T,
    pub next: Option<*mut ListNode<T>>,
    pub prev: Option<*mut ListNode<T>>,
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

impl<T> List<T> {
    pub const fn new() -> List<T> {
        return List {
            head: None,
            tail: None,
            length: 0,
        };
    }

    pub fn push(&mut self, payload: T, addr: usize) {
        // Create a new node directly at a memory location
        let new_node = unsafe { &mut *(addr as *mut ListNode<T>) };
        new_node.payload = payload;
        new_node.next = None;
        new_node.prev = None;

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
    }

    pub fn remove_at(&mut self, index: usize) {
        if index < 0 || index > self.length {
            panic!("List-Remove: Index Out of Bounds");
        }

        let length = self.length;

        match index {
            0 => unsafe {
                let head = ListNode::get_mut_ref(self.head);
                let head_next = ListNode::get_mut_ref(head.next);

                self.head = head.next;
                let payload = &head.payload;
                head_next.prev = None;
            },
            length => unsafe {
                let tail = ListNode::get_mut_ref(self.tail);
                let tail_prev = ListNode::get_mut_ref(tail.prev);

                self.tail = tail.prev;

                let payload = &tail.payload;
                tail_prev.next = None;
                tail.prev = None;
            },
            _ => {
                // Implement this
            }
        }
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
    // type Item = T;

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
