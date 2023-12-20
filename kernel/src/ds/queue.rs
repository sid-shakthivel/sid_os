// /// Node for the priority queue
// struct PriorityNode<T> {
//     priority: usize,
//     value: T,
//     next: Option<*mut PriorityNode>,
// }

// /// Priority Queue data structure
// struct PriorityQueue<T> {
//     front: Option<*mut PriorityNode>,
//     rear: Option<*mut PriorityNode>,
// }

// impl PriorityQueue {
//     fn new() -> Self {
//         PriorityQueue {
//             front: None,
//             rear: None,
//         }
//     }

//     fn enqueue(&mut self, priority: usize, value: usize) {
//         let new_node = PriorityNode {
//             priority,
//             value,
//             next: None
//         };

//         let new_node_ptr = Box::into_raw(new_node);

//         if let Some(rear) = self.rear {
//             unsafe {
//                 (*rear).next = Some(new_node_ptr);
//             }
//         } else {
//             self.front = Some(unsafe { Box::from_raw(new_node_ptr) });
//         }

//         self.rear = Some(new_node_ptr);
//     }

//     fn dequeue(&mut self) -> Option<(usize, usize)> {
//         self.front.take().map(|node| {
//             let priority = node.priority;
//             let value = node.value;
//             self.front = node.next;
//             if self.front.is_none() {
//                 self.rear = None;
//             }
//             (priority, value)
//         })
//     }
// }
