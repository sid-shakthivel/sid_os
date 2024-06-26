use crate::{
    memory::{page_frame_allocator::PAGE_FRAME_ALLOCATOR, paging},
    multitask::elf,
    print_serial,
};

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
    pub priority: ProcessPriority,
    pub p4: usize,
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

#[macro_export]
macro_rules! either {
    ($test:expr => $true_expr:expr; $false_expr:expr) => {
        if $test {
            $true_expr
        } else {
            $false_expr
        }
    };
}

// multiboot data defines the address of the process followed by its size
impl Process {
    pub fn init(is_user: bool, pid: usize, start_addr: usize) -> Process {
        // Allocate a page of memory for the stack
        // let mut rsp: *mut usize = kmalloc(paging::PAGE_SIZE);
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

        Process {
            pid,
            rsp,
            priority: either!(is_user => ProcessPriority::Low; ProcessPriority::High),
            p4,
        }
    }
}
