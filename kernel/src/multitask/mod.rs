/*
    Preemptive multitasking is when the CPU splits up its time between various processes to give the illusion they are happening simultaneously
    Interprocess communication is way processes communicate with each other
    Message passing model - processes communicate through kernel by sending and recieving messages through kernel without sharing same address space (can syncrynse actions)
    Messages can be fixed or variable length
    Communication link must exist between 2 processes like buffering, synchronisation,
*/

mod elf;
pub mod process;
mod process_manager;
pub mod syscalls;

use crate::utils::spinlock::Lock;
use process_manager::ProcessManager;

pub static PROCESS_MANAGER: Lock<ProcessManager> = Lock::new(ProcessManager::new());
