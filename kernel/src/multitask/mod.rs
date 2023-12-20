/*
    Preemptive multitasking is when the CPU splits up its time between various processes to give the illusion they are happening simultaneously
    Interprocess communication is way processes communicate with each other
    Message passing model - processes communicate through kernel by sending and recieving messages through kernel without sharing same address space (can syncrynse actions)
    Messages can be fixed or variable length
    Communication link must exist between 2 processes like buffering, synchronisation,
*/

use crate::memory::page_frame_allocator::PAGE_FRAME_ALLOCATOR;

pub const MAX_PROCESS_NUM: usize = 4;

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
    // pub cr3: *mut Table,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ProcessPriority {
    High,
    Low,
}

impl Process {
    // The entrypoint for each process is 0x800000 which has already been mapped into memory
    pub fn init(priority: ProcessPriority, pid: usize) -> Process {
        let mut rsp: *mut usize = PAGE_FRAME_ALLOCATOR.lock().alloc_page_frame().unwrap();
        PAGE_FRAME_ALLOCATOR.free();

        unsafe {
            rsp = rsp.offset(4095);
            let stack_top: usize = rsp as usize;

            /*
               When interrupt is called the following registers are pushed as follows: SS -> RSP -> RFLAGS -> CS -> RIP
               These registers are then pushed: RAX -> RBX -> RBC -> RDX -> RSI -> RDI -> R8..R15
            */

            *rsp.offset(-1) = 0x20 | 0x3; // SS
            *rsp.offset(-2) = stack_top; // RSP
            *rsp.offset(-3) = 0x202; // RFLAGS which enable interrupts
            *rsp.offset(-4) = 0x18 | 0x3; // CS
                                          // *rsp.offset(-5) = v_address; // RIP
            *rsp.offset(-6) = 0x00; // RAX
            *rsp.offset(-7) = 0x00; // RBX
            *rsp.offset(-8) = 0x00; // RCX
            *rsp.offset(-9) = 0x00; // RDX
            *rsp.offset(-10) = 0; // RBP
            *rsp.offset(-11) = 0; // RDI (argv)
            *rsp.offset(-12) = 0; // RSI (argc)
            *rsp.offset(-13) = 0; // R8
            *rsp.offset(-14) = 0; // R8
            *rsp.offset(-15) = 0; // R9
            *rsp.offset(-16) = 0; // R10
            *rsp.offset(-17) = 0; // R11
            *rsp.offset(-18) = 0; // R12
            *rsp.offset(-19) = 0; // R14
            *rsp.offset(-20) = 0; // R15
                                  // *rsp.offset(-21) = new_p4 as u64; // CR3
            rsp = rsp.offset(-21);
        }

        Process {
            pid: pid,
            rsp: rsp,
            process_priority: priority,
            time_taken: 0,
        }
    }
}
