use crate::{
    ds::queue::{PriorityQueue, PriorityWrapper},
    memory::{allocator::kmalloc, gdt::TSS},
    print_serial,
};

use core::mem::size_of;

use super::process::{Message, Process, ProcessPriority, ProcessState};

pub struct ProcessManager {
    pub tasks: PriorityQueue<Process>,
    pub current_process_id: usize,
    pub is_from_kernel: bool,
}

fn find_process(node: &PriorityWrapper<Process>, pid: usize) -> bool {
    return node.value.pid == pid;
}

impl ProcessManager {
    pub const fn new() -> ProcessManager {
        ProcessManager {
            tasks: PriorityQueue::<Process>::new(),
            current_process_id: 0,
            is_from_kernel: true,
        }
    }

    pub fn init(&mut self) {
        self.tasks.init();
    }

    pub fn add_process(&mut self, is_user: bool, pid: usize, multiboot_start_addr: usize) {
        let process = Process::init(is_user, pid, multiboot_start_addr);
        let converted_priority = ProcessPriority::convert(process.priority);
        self.tasks.enqueue(process, converted_priority);
    }

    pub fn get_current_process(&mut self) -> &mut Process {
        self.tasks.peek()
    }

    pub fn remove_process(&mut self) {
        // Mark process for termination
        let current_process = self.tasks.peek();
        current_process.state = ProcessState::Terminated;
    }

    pub fn send_message(&mut self, message: *mut Message) {
        let message_ref = unsafe { &mut *message };

        let current_process = self.tasks.peek();

        message_ref.sender_pid = current_process.pid;

        let receiver_process = self
            .tasks
            .nodes
            .find_where(&find_process, message_ref.receiver_pid);

        if let Some(process_index) = receiver_process {
            let receiver_process = self
                .tasks
                .nodes
                .get_mut(process_index)
                .expect("Process not found");

            receiver_process.value.messages.enqueue(*message_ref);

            if receiver_process.value.state == ProcessState::Blocked {
                receiver_process.value.unblock();
            }
        }
    }

    pub fn receive_message(&mut self) -> Option<*mut Message> {
        let process = self.tasks.peek();

        if let Some(message) = process.messages.dequeue() {
            let message_addr = kmalloc(size_of::<Message>()) as *mut Message;

            unsafe {
                core::ptr::write(message_addr, message);
            }

            return Some(message_addr as *mut Message);
        }

        process.block();
        None
    }

    fn is_all_process_blocked(&self) -> bool {
        for process in self.tasks.nodes.iter() {
            if process.value.state == ProcessState::Running {
                return false;
            }
        }
        return true;
    }

    pub fn switch_process(&mut self, old_rsp: usize) -> usize {
        if self.is_from_kernel {
            self.is_from_kernel = false;
            unsafe {
                TSS.privilege_stack_table[0] = old_rsp;
            }
        } else {
            if let Some(mut process) = self.tasks.dequeue() {
                // If the process is not terminated, then remove
                if process.state != ProcessState::Terminated {
                    let converted_priority = ProcessPriority::convert(process.priority);
                    process.rsp = old_rsp as *const usize;
                    self.tasks.enqueue(process, converted_priority);
                }
            }
        }

        if self.tasks.is_empty() {
            self.is_from_kernel = true;
            return old_rsp;
        } else if self.is_all_process_blocked() {
            self.is_from_kernel = true;
            return old_rsp;
        } else {
            let next_process = self.tasks.peek();

            return next_process.rsp as usize;
        }
    }
}
