use crate::{
    print_serial,
    utils::{bitwise, ports},
};

const CMOS_PORT_ADDR: u16 = 0x70;
const CMOS_PORT_DATA: u16 = 0x71;
const CURRENT_YEAR: u16 = 2024;
const CENTURY_REGISTER: u16 = 0x00;

#[derive(Copy, Clone, PartialEq)]
pub struct DateTime {
    second: u8,
    minute: u8,
    hour: u8,
    day: u8,
    month: u8,
    year: u16,
}

impl DateTime {
    pub const fn new() -> DateTime {
        DateTime {
            second: 0,
            minute: 0,
            hour: 0,
            day: 0,
            month: 0,
            year: 0,
        }
    }

    pub fn print(&self) {
        print_serial!(
            "Current Time: {:02}/{:02}/{:04} {:02}:{:02}:{:02}\n",
            self.day,
            self.month,
            self.year,
            self.hour,
            self.minute,
            self.second,
        );
    }

    pub fn copy_from(&mut self, other: &DateTime) {
        self.second = other.second;
        self.minute = other.minute;
        self.hour = other.hour;
        self.day = other.day;
        self.month = other.month;
        self.year = other.year;
    }

    pub fn get_time(&self) -> (u8, u8, u8) {
        (self.hour, self.minute, self.second)
    }

    pub fn get_date(&self) -> (u8, u8, u16) {
        (self.day, self.month, self.year)
    }

    pub fn calculate_year(&mut self) {
        self.year += ((CURRENT_YEAR / 100) * 100) as u16;
        if self.year < CURRENT_YEAR as u16 {
            self.year += 100;
        }
    }

    pub fn convert_to_binary(&mut self, register_b: u8) {
        if !bitwise::contains_bit(register_b, 0x04) {
            self.second = (self.second & 0x0F) + ((self.second / 16) * 10);
            self.minute = (self.minute & 0x0F) + ((self.minute / 16) * 10);
            self.hour = (self.hour & 0x0F) + ((self.hour / 16) * 10) | (self.hour & 0x80);
            self.day = (self.day & 0x0F) + ((self.day / 16) * 10);
            self.month = (self.month & 0x0F) + ((self.month / 16) * 10);
            self.year = (self.year & 0x0F) as u16 + (((self.year / 16) * 10) as u16);
        }
    }

    pub fn convert_to_bst(&mut self, register_b: u8) {
        // Account for British Summer Time
        self.hour += 1;
    }

    pub fn convert_to_24(&mut self, register_b: u8) {
        if !(bitwise::contains_bit(register_b, 0x02)) && bitwise::contains_bit(self.hour, 0x80) {
            self.hour = (((self.hour & 0x7F) + 12) % 24);
        }
    }

    pub fn update_values(&mut self) {
        self.second = get_rtc_register(0x00);
        self.minute = get_rtc_register(0x02);
        self.hour = get_rtc_register(0x04);
        self.day = get_rtc_register(0x07);
        self.month = get_rtc_register(0x08);
        self.year = get_rtc_register(0x09) as u16;
    }
}

fn get_update_in_progress_flag() -> bool {
    ports::outb(CMOS_PORT_ADDR, 0x0A);
    bitwise::contains_bit(ports::inb(CMOS_PORT_DATA), 0x80)
}

fn get_rtc_register(reg: u8) -> u8 {
    ports::outb(CMOS_PORT_ADDR, reg);
    ports::inb(CMOS_PORT_DATA)
}

pub fn get_current_datetime() -> DateTime {
    read_rtc();
    return unsafe { DATETIME };
}

fn read_rtc() {
    unsafe {
        let mut register_b: u8 = 0;

        let mut new_datetime = DateTime::new();

        // Read RTC values
        while get_update_in_progress_flag() {}

        DATETIME.update_values();

        // Ensure consistent values
        loop {
            new_datetime.copy_from(&DATETIME);

            while get_update_in_progress_flag() {}

            DATETIME.update_values();

            if new_datetime == DATETIME {
                break;
            }
        }

        register_b = get_rtc_register(0x0B);

        DATETIME.convert_to_binary(register_b);
        DATETIME.convert_to_24(register_b);
        DATETIME.convert_to_bst(register_b);
        DATETIME.calculate_year();
    }
}

static mut DATETIME: DateTime = DateTime::new();
