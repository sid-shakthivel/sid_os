/*
    Programmable interval timer is a chip which is used to implement a system clock as it sends interrupts on a regular basis
    Channel 0 (0x40) is connected to IRQ 0
    0x43 is command port
*/

use crate::utils::ports::outb;
use crate::utils::spinlock::Lock;

pub struct Pit {
    divisor: usize,
    ticks: usize,
}

const INPUT_CLOCK: usize = 1193180;
const FREQUENCY: usize = 100;

impl Pit {
    pub fn new(frequency: usize) -> Pit {
        Pit {
            ticks: 0,
            divisor: INPUT_CLOCK / frequency,
        }
    }

    pub fn init(&self) {
        // Set command byte (0x36)
        let mode = 0b00000000 | 0b00110000 | 0b00000000;
        outb(0x43, mode);
        self.set_frequency();
    }

    pub fn handle_timer(&mut self) {
        self.ticks += 1;
        self.set_frequency();
    }

    fn set_frequency(&self) {
        // To set a frequency, a divisor is sent in bits
        outb(0x40, (self.divisor & 0xFF) as u8);
        outb(0x40, (self.divisor >> 8) as u8);
    }
}
