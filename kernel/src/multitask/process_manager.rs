use crate::{ds::queue::PriorityQueue, memory::gdt::TSS, print_serial};

use super::process::{Process, ProcessPriority};

pub struct ProcessManager {
    pub tasks: PriorityQueue<Process>,
    pub current_process_id: usize,
    pub is_from_kernel: bool,
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

    // WARNING: This may not work
    pub fn remove_process(&mut self) {
        self.tasks.dequeue();
    }

    pub fn switch_process(&mut self, old_rsp: usize) -> usize {
        if self.is_from_kernel {
            self.is_from_kernel = false;
            unsafe {
                TSS.privilege_stack_table[0] = old_rsp;
            }
        } else {
            if let Some(mut process) = self.tasks.dequeue() {
                let converted_priority = ProcessPriority::convert(process.priority);
                process.rsp = old_rsp as *const usize;
                self.tasks.enqueue(process, converted_priority);
            }
        }

        if self.tasks.is_empty() {
            self.is_from_kernel = true;
            return old_rsp;
        } else {
            let next_process = self.tasks.peek();
            return next_process.rsp as usize;
        }
    }
}
