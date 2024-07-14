use crate::{
    ds::{hashmap::HashMap, queue::Queue},
    either,
    fs::vfs::File,
    memory::{page_frame_allocator::PAGE_FRAME_ALLOCATOR, paging},
    multitask::elf,
    print_serial,
};

// The entrypoint for each user mode process
pub static USER_PROCESS_START_ADDRESS: usize = 0x8000000;

#[derive(Debug, Copy, Clone)]
pub struct Message {
    pub sender_pid: usize,
    pub receiver_pid: usize,
    pub message: *const u8,
    pub length: usize,
    pub m_type: usize,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ProcessState {
    Running,
    Blocked,
    Terminated,
}

/*
    Processes are running programs with an individual address space, stack and data which run in userspace
    Processes will be selected based on  priority
    Procesess are mapped into a specific address space
*/
#[derive(Copy, Clone, Debug)]
pub struct Process {
    pub pid: usize,
    pub rsp: *const usize,
    pub priority: ProcessPriority,
    p4: usize,
    pub fdt: HashMap<*mut File>,
    pub state: ProcessState,
    pub messages: Queue<Message>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ProcessPriority {
    High,
    Low,
}

impl ProcessPriority {
    pub fn convert(process_priority: ProcessPriority) -> usize {
        match process_priority {
            Self::High => 5,
            Self::Low => 10,
        }
    }
}

// multiboot data defines the address of the process followed by its size
impl Process {
    pub fn init(is_user: bool, pid: usize, start_addr: usize) -> Process {
        // Allocate a page of memory for the stack
        // Use PFA for safety
        let mut rsp = PAGE_FRAME_ALLOCATOR.lock().alloc_page_frame().unwrap();
        PAGE_FRAME_ALLOCATOR.free();

        let mut p4 = paging::deep_clone() as usize;

        elf::parse(start_addr, p4);

        print_serial!("Parsed process successfully\n");

        unsafe {
            rsp = rsp.offset(511);
            let stack_top: usize = rsp as usize;

            /*
               When interrupt is called the following registers are pushed as follows: SS -> RSP -> RFLAGS -> CS -> RIP
               These registers are then pushed: RAX -> RBX -> RBC -> RDX -> RSI -> RDI -> R8..R15
            */
            *rsp.offset(-1) = either!(is_user => 0x20 | 0x3; 0x10); // SS
            *rsp.offset(-2) = stack_top; // RSP
            *rsp.offset(-3) = 0x202; // RFLAGS which enable interrupts
            *rsp.offset(-4) = either!(is_user=> 0x18 | 0x3; 0x08); // CS
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

        let fdt = HashMap::<*mut File>::new();

        Process {
            pid,
            rsp,
            priority: either!(is_user => ProcessPriority::Low; ProcessPriority::High),
            p4,
            fdt,
            state: ProcessState::Running,
            messages: Queue::<Message>::new(),
        }
    }

    pub fn block(&mut self) {
        self.state = ProcessState::Blocked;
    }

    pub fn unblock(&mut self) {
        self.state = ProcessState::Running;
    }
}
