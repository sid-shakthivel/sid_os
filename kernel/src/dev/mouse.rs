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
use crate::utils::bitwise;
use crate::utils::spinlock::Lock;
use crate::{panic, print_serial};

#[repr(u8)]
enum GenericPacketBits {
    LeftBtnClicked = 0b00000001,
    RightBtnClicked = 0b00000010,
    MidBtnClicked = 0b00000100,
    IsFour = 0b00001000,
    XSignBit = 0b00010000,
    YSignBit = 0b00100000,
    XOverflow = 0b01000000,
    YOverflow = 0b10000000,
}

#[repr(u8)]
enum Button5MouseZAxisBits {
    Value = 0x0F,
    AdditionalButtonA = 0b00010000,
    AdditionalButtonB = 0b00100000,
}

pub struct Mouse {
    x: usize,
    y: usize,
    z: usize,
    flags: u8,
    current_byte: usize,
    variety: ps2::PS2Device,
}

impl Mouse {
    /*
       It should be noted that placing a self.get_type() after self.enable_scanning();
       Will result in the mouse not working at all
       Do not do this
    */
    pub fn init(&mut self) {
        self.enable_z_axis();
        // self.enable_5_buttons();

        assert!(
            self.get_type() == ps2::PS2Device::PS2MouseScrollWheel,
            "Error: Mouse does not have scroll wheel"
        );

        self.enable_scanning();
    }

    pub fn handle_mouse_interrupt(&mut self) {
        if !ps2::is_from_mouse() {
            return;
        }

        let byte = ps2::read(0x60).unwrap();

        match self.current_byte {
            0 => {
                if !bitwise::contains_bit(byte, GenericPacketBits::IsFour as u8) {
                    return;
                }

                if bitwise::contains_bit(byte, GenericPacketBits::LeftBtnClicked as u8) {
                    print_serial!("Left Click\n");
                }

                if bitwise::contains_bit(byte, GenericPacketBits::RightBtnClicked as u8) {
                    print_serial!("Right Click\n");
                }

                self.flags = byte;
            }
            1 => {
                if bitwise::contains_bit(self.flags, GenericPacketBits::XOverflow as u8) {
                    return;
                }

                if bitwise::contains_bit(self.flags, GenericPacketBits::XSignBit as u8) {
                    self.x = self.x.wrapping_add(self.sign_extend(byte) as usize);
                } else {
                    self.x = self.x.wrapping_add(byte as usize);
                }
            }
            2 => {
                if bitwise::contains_bit(self.flags, GenericPacketBits::YOverflow as u8) {
                    return;
                }

                if bitwise::contains_bit(self.flags, GenericPacketBits::YSignBit as u8) {
                    self.y = self.y.wrapping_add((self.sign_extend(byte) * -1) as usize);
                } else {
                    self.y = self.y.wrapping_add((byte as i16 * -1) as usize);
                }
            }
            3 => match self.variety {
                ps2::PS2Device::PS2Mouse => panic!("PS2 Mouse"),
                ps2::PS2Device::PS2MouseFiveButtons => {
                    self.z = (Button5MouseZAxisBits::Value as i16 & byte as i16) as usize;

                    if bitwise::contains_bit(byte, Button5MouseZAxisBits::AdditionalButtonA as u8) {
                        print_serial!("Button A Pressed\n");
                    }

                    if bitwise::contains_bit(byte, Button5MouseZAxisBits::AdditionalButtonB as u8) {
                        print_serial!("Button B Pressed\n");
                    }
                }
                ps2::PS2Device::PS2MouseScrollWheel => {
                    self.z = self.z.wrapping_add((byte as i16) as usize);
                }
                _ => panic!("Unknown mouse type"),
            },
            _ => {}
        }

        self.current_byte = (self.current_byte + 1) % 4;
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
    }

    // Uses a magic sequence
    fn enable_5_buttons(&mut self) {
        self.set_mouse_rate(200);
        self.set_mouse_rate(200);
        self.set_mouse_rate(80);
    }

    fn get_type(&mut self) -> ps2::PS2Device {
        self.variety = ps2::identify_device_type(1).unwrap();
        return self.variety;
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
    x: 512,
    y: 384,
    z: 0,
    flags: 0,
    current_byte: 0,
    variety: ps2::PS2Device::PS2Mouse,
});
