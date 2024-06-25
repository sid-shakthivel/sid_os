/*
    Preemptive multitasking is when the CPU splits up its time between various processes to give the illusion they are happening simultaneously
    Interprocess communication is way processes communicate with each other
    Message passing model - processes communicate through kernel by sending and recieving messages through kernel without sharing same address space (can syncrynse actions)
    Messages can be fixed or variable length
    Communication link must exist between 2 processes like buffering, synchronisation,
*/

use core::hash::Hash;
use core::usize;

use crate::ds::queue::PriorityQueue;
use crate::memory::allocator::{kfree, kmalloc};
use crate::memory::gdt::TSS;
use crate::memory::page_frame_allocator::PAGE_FRAME_ALLOCATOR;
use crate::memory::paging::{self, PageTable}; 
use crate::print_serial;
use crate::utils::spinlock::Lock;
use crate::CONSOLE;

mod elf;
pub mod syscalls;

// The entrypoint for each user mode process
pub static USER_PROCESS_START_ADDRESS: usize = 0x8000000;

/*
    Processes are running programs with an individual address space, stack and data which run in userspace
    Processes will be selected based on  priority
    Procesess are mapped into a specific address space
*/
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Process {
    pub pid: usize,
    pub rsp: *const usize,
    pub process_priority: ProcessPriority,
    pub time_taken: usize,
    pub p4: usize,
}

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

    pub fn add_process(
        &mut self,
        priority: ProcessPriority,
        pid: usize,
        multiboot_start_addr: usize,
    ) {
        let converted_priority = ProcessPriority::convert_to_value(priority);
        let process = Process::init(priority, pid, multiboot_start_addr);

        self.tasks.enqueue(process, converted_priority);
    }

    pub fn switch_process(&mut self, old_rsp: usize) -> usize {
        let current_process = self.tasks.get_head();

        /*
           Coming from the kernel:
           - Update TSS to have a clean stack when coming from user to kernel
        */
        if (self.is_from_kernel) {
            self.is_from_kernel = false;
            unsafe {
                TSS.privilege_stack_table[0] = old_rsp;
            }
        } else {
            current_process.rsp = old_rsp as *const usize;
        }

        return current_process.rsp as usize;
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ProcessPriority {
    High,
    Low,
}

impl ProcessPriority {
    pub fn convert_to_value(process_priority: ProcessPriority) -> usize {
        match process_priority {
            Self::High => 10,
            Self::Low => 5,
        }
    }
}

// multiboot data defines the address of the process followed by its size
impl Process {
    pub fn init(priority: ProcessPriority, pid: usize, multiboot_start_addr: usize) -> Process {
        // Allocate a page of memory for the stack
        // let mut rsp: *mut usize = kmalloc(paging::PAGE_SIZE);
        let mut rsp = PAGE_FRAME_ALLOCATOR.lock().alloc_page_frame().unwrap();
        PAGE_FRAME_ALLOCATOR.free();

        let mut p4 = paging::deep_clone() as usize;

        elf::parse(multiboot_start_addr, p4);

        print_serial!("Parsed process successfully\n");

        unsafe {
            rsp = rsp.offset(511);
            let stack_top: usize = rsp as usize;

            /*
               When interrupt is called the following registers are pushed as follows: SS -> RSP -> RFLAGS -> CS -> RIP
               These registers are then pushed: RAX -> RBX -> RBC -> RDX -> RSI -> RDI -> R8..R15
            */
            *rsp.offset(-1) = 0x20 | 0x3; // SS
                                          // *rsp.offset(-1) = 0x10; // SS
            *rsp.offset(-2) = stack_top; // RSP
            *rsp.offset(-3) = 0x202; // RFLAGS which enable interrupts
            *rsp.offset(-4) = 0x18 | 0x3; // CS
                                          // *rsp.offset(-4) = 0x08; // CS
            *rsp.offset(-5) = USER_PROCESS_START_ADDRESS; // RIP
            *rsp.offset(-6) = 0x00; // RAX
            *rsp.offset(-7) = 0x00; // RBX
            *rsp.offset(-8) = 0x00; // RCX
            *rsp.offset(-9) = 0x00; // RDX
            *rsp.offset(-10) = 0; // RBP
            *rsp.offset(-11) = 0; // RDI (argv)
            *rsp.offset(-12) = 0; // RSI (argc)
            *rsp.offset(-13) = 0; // R8
            *rsp.offset(-14) = 0; // R9
            *rsp.offset(-15) = 0; // R10
            *rsp.offset(-16) = 0; // R11
            *rsp.offset(-17) = 0; // R12
            *rsp.offset(-18) = 0; // R13
            *rsp.offset(-19) = 0; // R14
            *rsp.offset(-20) = 0; // R15
            *rsp.offset(-21) = p4; // CR3
            rsp = rsp.offset(-21);
        }

        Process {
            pid,
            rsp,
            process_priority: priority,
            time_taken: 0,
            p4,
        }
    }
}

pub static PROCESS_MANAGER: Lock<ProcessManager> = Lock::new(ProcessManager::new());
