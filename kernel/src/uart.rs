/*
    Physical serial ports provide a connector to attach devices (trasmits 1 byte at a time through a single channel)
    Serial ports are bi-directional (half duplex) and are controlled by uart (chip which encodes and decodes data)
    Must supply speed used for sending data (baud rate), error checking, data bits
*/

const PORT: u16 = 0x3F8; // COM1

use crate::output::Output;
use crate::ports::{inb, outb};
use crate::spinlock::Lock;
use core::fmt;

pub struct Console {
    port: u16,
}

impl Console {
    pub fn init(&self) {
        outb(PORT + 1, 0x00); // Disable all interrupts
        outb(PORT + 3, 0x80); // Enable DLAB (set baud rate divisor)
        outb(PORT + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
        outb(PORT + 1, 0x00); //                  (hi byte)
        outb(PORT + 3, 0x03); // 8 bits, no parity, one stop bit
        outb(PORT + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
        outb(PORT + 4, 0x0B); // IRQs enabled, RTS/DSR set
        outb(PORT + 4, 0x1E); // Set in loopback mode, test the serial chip
        outb(PORT + 0, 0xAE); // Test serial chip (send byte 0xAE and check if serial returns same byte)

        // Check if serial is faulty (i.e: not same byte as sent)
        if inb(PORT + 0) != 0xAE {}

        // If serial is not faulty set it in normal operation mode
        // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
        outb(PORT + 4, 0x0F);
    }

    fn read_serial(&self) -> char {
        while self.has_serial_received() == 0 {}
        return inb(self.port) as char;
    }

    fn has_serial_received(&self) -> u8 {
        return inb(self.port + 5) & 1;
    }

    fn write_e9_hack(&self, character: char) {
        while self.is_transmit_empty() == 0 {}
        outb(0xe9, character as u8);
    }

    fn is_transmit_empty(&self) -> u8 {
        return inb(self.port + 5) & 0x20;
    }
}

impl Output for Console {
    fn put_char(&mut self, character: char) {
        while self.is_transmit_empty() == 0 {}
        outb(self.port, character as u8);

        self.write_e9_hack(character);
    }
}

pub static CONSOLE: Lock<Console> = Lock::new(Console { port: PORT });

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print_serial {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        CONSOLE.lock().write_fmt(format_args!($($arg)*)).unwrap();
        CONSOLE.free();
    });
}
