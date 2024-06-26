use super::list::{List, ListIterator, ListNode};
use super::vec::DynamicArray;
use crate::memory::{
    allocator::{kfree, kmalloc},
    page_frame_allocator::PAGE_FRAME_ALLOCATOR,
    paging::PAGE_SIZE,
};
use crate::print_serial;
use core::cmp::Ordering;

// A wrapper to add priority to any type
#[derive(Clone, Copy)]
struct PriorityWrapper<T> {
    priority: usize,
    value: T,
}

impl<T> Ord for PriorityWrapper<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority)
    }
}

impl<T> PartialOrd for PriorityWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Eq for PriorityWrapper<T> {}

impl<T> PartialEq for PriorityWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl<T> PriorityWrapper<T> {
    pub fn new(payload: T, priority: usize) -> PriorityWrapper<T> {
        PriorityWrapper {
            priority,
            value: payload,
        }
    }
}

/*
    Priority Queue uses a binary heap (min-heap)
    This ensures every parent node is less than or equal to its child nodes
*/
pub struct PriorityQueue<T: 'static> {
    pub nodes: DynamicArray<PriorityWrapper<T>>,
}

impl<T: Clone> PriorityQueue<T> {
    pub const fn new() -> PriorityQueue<T> {
        PriorityQueue {
            nodes: DynamicArray::new(),
        }
    }

    pub fn init(&mut self) {
        self.nodes.init();
    }

    pub fn enqueue(&mut self, payload: T, priority: usize) {
        let priority_wrapped_node = PriorityWrapper::new(payload, priority);
        self.nodes.push(priority_wrapped_node);
        self.swim();
    }

    pub fn swim(&mut self) {
        let mut index = self.nodes.length() - 1;
        while index > 0 {
            let parent = (index - 1) / 2;
            if self.nodes.get(index) > self.nodes.get(parent) {
                self.nodes.swap(index, parent);
                index = parent;
            } else {
                break;
            }
        }
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if self.nodes.is_empty() {
            return None;
        }

        self.nodes.swap(0, self.nodes.length() - 1);
        let wrapper = self.nodes.pop().expect("Priority Queue is empty");
        self.sink(0);

        return Some(wrapper.value);
    }

    pub fn sink(&mut self, mut index: usize) {
        let len = self.nodes.length();
        let mut left = 2 * index + 1;

        while left < len {
            let right = left + 1;
            let largest = if right < len && self.nodes.get(right) > self.nodes.get(left) {
                right
            } else {
                left
            };

            if self.nodes.get(largest) > self.nodes.get(0) {
                self.nodes.swap(largest, index);
                index = largest;
                left = 2 * index + 1;
            } else {
                break;
            }
        }
    }

    pub fn peek(&mut self) -> &mut T {
        let node = self.nodes.get(0).expect("Priority Queue is empty") as *mut PriorityWrapper<T>;

        unsafe {
            let node_value_mut = &mut ((*node).value) as *mut T;
            return &mut *node_value_mut;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.length()
    }
}

pub struct Queue<T: 'static> {
    list: List<T>,
}

impl<T: Clone> Queue<T> {
    pub const fn new() -> Queue<T> {
        Queue { list: List::new() }
    }

    pub fn enqueue(&mut self, payload: T) {
        let addr = kmalloc(core::mem::size_of::<ListNode<T>>()) as usize;
        self.list.push_back(payload, addr);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.list.remove_at(0).map(|node| {
            kfree(node.1);
            node.0
        })
    }

    pub fn empty(&mut self) {
        while self.dequeue().is_some() {}
    }

    pub fn get_element(&mut self, index: usize) -> &mut T {
        return self
            .list
            .get_at(index)
            .expect("ERROR: Queue does not contain element at specified index");
    }

    pub fn peek(&mut self) -> &mut T {
        let value = self.list.head.expect("ERROR: Queue is empty");
        return unsafe { &mut (*value).payload };
    }
}
