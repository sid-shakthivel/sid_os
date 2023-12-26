/*
    PS2 Mouse communicates with the PS2 controller using serial communication
    Mouse is enabled on PS2 bys, (0xFA means acknowledge)
    Mouse sends 3/4 byte packets to communicate movement at port 0x60 (bit 5 on status register indicates if from mouse)
    Packets are generated at a rate (100 packets a second) and if mouse is pressed/released
    Byte 1:
    +------------+------------+-------------+------------+-------+------------+-----------+----------+
    |   Bit 0    |   Bit 1    |    Bit 2    |   Bit 3    | Bit 4 |   Bit 5    |  Bit 6    |  Bit 7   |
    +------------+------------+-------------+------------+-------+------------+-----------+----------+
    | Y Overflow | X Overflow | Y Sign Bit  | X Sign Bit |     1 | Middle Btn | Right Btn | Left Btn |
    +------------+------------+-------------+------------+-------+------------+-----------+----------+
    Byte 2: X Movement
    Byte 3: Y Movement
*/

use super::ps2;
use crate::gfx::wm::WM;
use crate::print_serial;
use crate::utils::spinlock::Lock;

#[repr(u8)]
enum MouseByte1Bits {
    LeftBtnClicked = 0b00000001,
    RightBtnClicked = 0b00000010,
    MidBtnClicked = 0b00000100,
    IsFour = 0b00001000,
    XSignBit = 0b00010000,
    YSignBit = 0b00100000,
    XOverflow = 0b01000000,
    YOverflow = 0b10000000,
}

#[derive(PartialEq)]
pub enum MouseState {
    Up,
    Down,
    Immobile,
}

pub struct Mouse {
    mouse_x: usize,
    mouse_y: usize,
    mouse_packets: [u8; 4],
    current_byte: usize,
    variety: ps2::PS2Device,
    mouse_state: MouseState,
}

impl Mouse {
    pub fn init(&mut self) {
        // self.enable_z_axis();
        // self.enable_5_buttons();
        // self.enable_scanning();

        // print_serial!("Mouse Type: {:?}\n", self.get_type());

        // assert!(
        //     self.get_type() == ps2::PS2Device::PS2MouseFiveButtons,
        //     "Mouse: Is not PS2 MouseFiveButtons"
        // );
    }

    pub fn handle_mouse_interrupt(&mut self) {
        if ps2::is_from_mouse() {
            let byte = ps2::read(0x60).unwrap();

            self.mouse_packets[self.current_byte] = byte;

            self.current_byte = (self.current_byte + 1) % 4;

            if self.current_byte == 0 {
                self.handle_mouse_packets();
            }
        }
    }

    fn contains(&self, value: u8, bitmask: u8) -> bool {
        (value & bitmask) > 0
    }

    fn handle_mouse_packets(&mut self) {
        let mut is_left_clicked = false;

        // Check overflows, if set, discard packet
        if self.mouse_packets[0] & (1 << 7) >= 0x80 || self.mouse_packets[0] & (1 << 6) >= 0x40 {
            return;
        }

        // Bit 3 verifies packet alignment (if wrong, should return error)
        if self.mouse_packets[0] & (1 << 3) != 0x08 {
            return;
        }

        // Left button pressed
        if self.mouse_packets[0] & (1 << 0) == 1 {
            print_serial!("Left button pressed\n");
            is_left_clicked = true;
            self.mouse_state = MouseState::Down;
        } else {
            self.mouse_state = MouseState::Up;
        }

        // Right button pressed
        // if self.mouse_packets[0] & (1 << 1) == 2 {
        //     return;
        // }

        // X movement and Y movement values must be read as a 9 bit or greater SIGNED value if bit is enabled
        if self.mouse_packets[0] & (1 << 4) == 0x10 {
            self.mouse_x = self
                .mouse_x
                .wrapping_add(self.sign_extend(self.mouse_packets[1]) as usize);
        } else {
            self.mouse_x = self.mouse_x.wrapping_add(self.mouse_packets[1] as usize);
        }

        if self.mouse_packets[0] & (1 << 5) == 0x20 {
            let adjusted_y = self.sign_extend(self.mouse_packets[2]) * -1;
            self.mouse_y = self.mouse_y.wrapping_add(adjusted_y as usize);
        } else {
            let adjusted_y = (self.mouse_packets[2] as i16) * -1;
            self.mouse_y = self.mouse_y.wrapping_add(adjusted_y as usize);
        }

        // WM.lock()
        //     .handle_mouse_event((self.mouse_x as u16, self.mouse_y as u16), is_left_clicked);
        // WM.free();
    }

    fn enable_scanning(&self) {
        ps2::write_to_device(1, 0xF4).unwrap(); // Set sample rate command
        ps2::wait_ack().unwrap();
    }

    fn disable_scanning(&self) {
        ps2::write_to_device(1, 0xF5).unwrap(); // Set sample rate command
        ps2::wait_ack().unwrap();
    }

    // Uses a magic sequence
    fn enable_z_axis(&mut self) {
        self.set_mouse_rate(200);
        self.set_mouse_rate(100);
        self.set_mouse_rate(80);
        if self.get_type() != ps2::PS2Device::PS2MouseScrollWheel {
            panic!("Scroll wheel failed");
        } else {
            self.variety = self.get_type();
        }
    }

    // Uses a magic sequence
    fn enable_5_buttons(&mut self) {
        self.set_mouse_rate(200);
        self.set_mouse_rate(200);
        self.set_mouse_rate(80);
    }

    fn get_type(&self) -> ps2::PS2Device {
        return ps2::identify_device_type(1).unwrap();
    }

    fn sign_extend(&self, packet: u8) -> i16 {
        ((packet as u16) | 0xFF00) as i16
    }

    fn set_mouse_rate(&self, sample_rate: u8) {
        ps2::write_to_device(1, 0xF3).unwrap(); // Set sample rate command
        ps2::wait_ack().unwrap();
        ps2::write_to_device(1, sample_rate).unwrap();
        ps2::wait_ack().unwrap();
    }
}

pub static MOUSE: Lock<Mouse> = Lock::new(Mouse {
    mouse_x: 512,
    mouse_y: 384,
    mouse_packets: [0; 4],
    current_byte: 0,
    variety: ps2::PS2Device::PS2Mouse,
    mouse_state: MouseState::Immobile,
});
