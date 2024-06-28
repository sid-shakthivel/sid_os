pub fn clear_bit(value: u8, bitmask: u8) -> u8 {
    value & !bitmask
}

pub fn set_bit(value: u8, bitmask: u8) -> u8 {
    value | bitmask
}

pub fn contains_bit(value: u8, bitmask: u8) -> bool {
    (value & bitmask) > 0
}

pub fn toggle_bit(value: u8, bitmask: u8) -> u8 {
    value ^ bitmask
}
