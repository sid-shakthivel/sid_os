/*
    PS/2 (Personal System 2) controller is part of AIP which is linked to the 8042 chip
    Green/purple ports which connect directly to keyboards and mice
    Has 2 buffers for data (one for data recieved and one for data written before it's sent)
    - Data port (0x60) which is used to read/write from PS/2 device/controller
    - Command/Status register (0x64) used to send commands
    - Writing a value to 0x64 sends a command byte whilst reading gets the status byte
*/

use super::keyboard::{Keyboard, KEYBOARD};
use super::mouse::MOUSE;
use crate::utils::ports::{inb, outb};
use crate::{print_serial, CONSOLE};

const PS2_DATA: u16 = 0x60; // Data port
const PS2_STATUS: u16 = 0x64;
const PS2_CMD: u16 = 0x64; // Command port
const TIMEOUT: i16 = 10000;

const GET_STATUS_BYTE: u8 = 0x20;
const SET_STATUS_BYTE: u8 = 0x60;

#[repr(u8)]
enum ControllerRegisterFlags {
    KeyboardInterruptEnable = 0b00000001,
    MouseInterruptEnable = 0b00000010,
    SystemFlag = 0b00000100,
    IgnoreKeyboardLock = 0b00001000,
    KeyboardEnable = 0b00010000,
    MouseEnable = 0b00100000,
    KeyboardTranlation = 0b01000000,
    IsUnused = 0b10000000,
}

#[derive(PartialEq, Debug)]
pub enum PS2Device {
    PS2Mouse,
    PS2MouseScrollWheel,
    PS2MouseFiveButtons,
    MF2Keyboard,
    MF2KeyboardTranslation,
}

fn remove_bit(value: u8, bitmask: u8) -> u8 {
    value & !bitmask
}

fn set_bit(value: u8, bitmask: u8) -> u8 {
    value | bitmask
}

pub fn init() -> Result<(), &'static str> {
    // Disable devices so ps2 devices can't mess up initialisation
    write(PS2_CMD, 0xAD)?;
    write(PS2_CMD, 0xA7)?;

    // Flush output buffer as data can be stuck in PS2 controller buffer
    inb(PS2_DATA);

    write(PS2_CMD, GET_STATUS_BYTE)?;
    let mut controller_config = read(PS2_DATA)?;

    controller_config = remove_bit(
        controller_config,
        ControllerRegisterFlags::KeyboardInterruptEnable as u8,
    );

    controller_config = remove_bit(
        controller_config,
        ControllerRegisterFlags::MouseInterruptEnable as u8,
    );

    controller_config = remove_bit(
        controller_config,
        ControllerRegisterFlags::KeyboardTranlation as u8,
    );

    write(PS2_CMD, SET_STATUS_BYTE)?;
    write(PS2_DATA, controller_config)?;

    // Perform controller self test
    write(PS2_CMD, 0xAA)?; // Test controller
    if read(PS2_DATA)? != 0x55 {
        panic!("Controller self test failed\n");
    }

    write(PS2_CMD, 0xA8)?; // Enable second PS2 port
    write(PS2_CMD, GET_STATUS_BYTE)?;

    controller_config = read(PS2_DATA)?;

    if (controller_config & ControllerRegisterFlags::MouseEnable as u8) > 0 {
        panic!("Not dual channel???\n");
    } else {
        write(PS2_CMD, 0xA7)?;
    }

    // Perform interface tests to test both ports
    write(PS2_CMD, 0xAB)?;
    if read(PS2_DATA)? != 0x00 {
        panic!("Interface test failed\n");
    }

    write(PS2_CMD, 0xA9)?;
    if read(PS2_DATA)? != 0x00 {
        panic!("Interface test failed\n");
    }

    // Enable both PS2 ports
    write(PS2_CMD, 0xAE)?;
    write(PS2_CMD, 0xA8)?;

    // Enable interrupts
    write(PS2_CMD, GET_STATUS_BYTE)?;
    controller_config = read(PS2_DATA)?;

    controller_config = set_bit(
        controller_config,
        ControllerRegisterFlags::KeyboardInterruptEnable as u8,
    );

    controller_config = set_bit(
        controller_config,
        ControllerRegisterFlags::MouseInterruptEnable as u8,
    );

    controller_config = set_bit(
        controller_config,
        ControllerRegisterFlags::KeyboardTranlation as u8,
    );

    write(PS2_CMD, SET_STATUS_BYTE)?;
    write(PS2_DATA, controller_config)?;

    // Reset devices
    for i in 0..2 {
        write_to_device(i, 0xFF)?;
        let response = read(PS2_DATA)?;

        if response != 0xFA || read(PS2_DATA)? != 0xAA {
            panic!("Reading device {} failed with {:x}", i, response);
        }

        // Mouse can send an extra 0x00 byte
        if (inb(PS2_STATUS) & 1) != 0 {
            read(PS2_DATA)?;
        }
    }

    // Identify devices and initialise them appropriately
    for i in 0..2 {
        match identify_device_type(i).unwrap() {
            PS2Device::MF2KeyboardTranslation => {
                KEYBOARD.lock().init();
                KEYBOARD.free();
            }
            PS2Device::PS2Mouse => {
                MOUSE.lock().init();
                MOUSE.free();
            }
            _ => panic!("Unknown device"),
        }
    }

    print_serial!("here\n");

    Ok(())
}

fn write(port: u16, byte: u8) -> Result<u8, &'static str> {
    let mut timeout = TIMEOUT;
    while (inb(PS2_STATUS) & 2) > 0 {
        timeout -= 1;
        if timeout < 0 {
            print_serial!("PS2 WRITE FAILED\n");
            return Err("PS2 Write Failed");
        }
    }
    outb(port, byte);
    return Ok(0);
}

pub fn read(port: u16) -> Result<u8, &'static str> {
    let mut timeout = TIMEOUT;
    while (inb(PS2_STATUS) & 1) == 0 {
        timeout -= 1;
        if timeout < 0 {
            print_serial!("PS2 READ FAILED\n");
            return Err("PS2 Read Failed");
        }
    }

    return Ok(inb(port));
}

pub fn is_from_mouse() -> bool {
    return inb(PS2_STATUS) & (1 << 5) == 0x20;
}

pub fn write_to_device(device_num: u16, byte: u8) -> Result<u8, &'static str> {
    return match device_num {
        0 => {
            write(PS2_DATA, byte)?;
            return Ok(0);
        }
        1 => {
            write(PS2_CMD, 0xD4)?;
            write(PS2_DATA, byte)?;
            return Ok(0);
        }
        _ => Err("Unknown device"),
    };
}

// Must wait to recieve acknowledgement from device (0xFA)
pub fn wait_ack() -> Result<bool, &'static str> {
    while read(PS2_DATA)? != 0xFA {}
    return Ok(true);
}

pub fn identify_device_type(device_num: u16) -> Result<PS2Device, &'static str> {
    write_to_device(device_num, 0xF5)?; // Send disable scanning command
    wait_ack()?;

    write_to_device(device_num, 0xF2)?; // Send identify command
    wait_ack()?;

    let mut response = read(PS2_DATA)?;
    return match response {
        0x00 => Ok(PS2Device::PS2Mouse),
        0x03 => Ok(PS2Device::PS2MouseScrollWheel),
        0x04 => Ok(PS2Device::PS2MouseFiveButtons),
        0xAB => {
            response = read(PS2_DATA)?;
            return match response {
                0x41 | 0xC1 => Ok(PS2Device::MF2KeyboardTranslation),
                0x83 => Ok(PS2Device::MF2Keyboard),
                _ => Err("Unknown device"),
            };
        }
        _ => {
            print_serial!("Unknown device response: {:x}\n", response);
            Err("Unknown device\n")
        }
    };
}
