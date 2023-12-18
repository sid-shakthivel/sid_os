// src/vga_text.rs

/*
VGA buffer is located at 0xb8000 (maps to VRAM)
Allows characters to be displayed upon the screen.
Screen has 25 rows of 80 length

Buffer entry format:
+---------------------------------------------+
|         | 15 | 12-14 | 8-11| 0-7 |          |
+---------------------------------------------+
| | Blink | Background | Foreground | ASCII | |
+---------------------------------------------+
*/

use crate::output::output::Output;

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
const VGA_BUFFER_ADDRESS: usize = 0xB8000;

pub struct Screen {
    current_row: usize,
    current_col: usize,
    buffer: &'static mut [[u16; VGA_WIDTH]; VGA_HEIGHT],
    colour_pair: ColourCode,
}

impl Screen {
    pub fn new() -> Screen {
        return Screen {
            current_row: 0,
            current_col: 0,
            buffer: unsafe { &mut *(VGA_BUFFER_ADDRESS as *mut [[u16; VGA_WIDTH]; VGA_HEIGHT]) },
            colour_pair: ColourCode::new((VgaColours::Black as u8, VgaColours::White as u8)),
        };
    }
}

#[allow(dead_code)]
enum VgaColours {
    Black = 0,
    Bue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGrey = 7,
    DarkGrey = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    LightBrown = 14,
    White = 15,
}

struct ColourCode(u8);

impl ColourCode {
    // Background followed by foregound
    pub fn new(colour_pair: (u8, u8)) -> ColourCode {
        let combined_colour = (colour_pair.0 as u8) << 4 | (colour_pair.1 as u8);
        return ColourCode(combined_colour);
    }

    pub fn generate_vga_entry(&self, character: char) -> u16 {
        return ((self.0 as u16) << 8) | (character as u16);
    }
}

impl Output for Screen {
    fn put_char(&mut self, character: char) {
        match character {
            '\n' => self.new_line(),
            _ => {
                let entry: u16 = self.colour_pair.generate_vga_entry(character);
                self.buffer[self.current_row][self.current_col] = entry;
                self.current_col += 1;

                if self.current_col >= VGA_WIDTH {
                    self.new_line();
                }
            }
        }
    }

    fn new_line(&mut self) {
        self.current_col = 0;
        self.current_row += 1;
    }

    fn clear(&mut self) {
        self.current_col = 0;
        self.current_row = 0;

        for _i in 0..VGA_HEIGHT {
            for _j in 0..VGA_WIDTH {
                self.put_char(' ');
            }
        }

        self.current_col = 0;
        self.current_row = 0;
    }
}

// #[macro_export]
// macro_rules! print_vga {
//     ($($arg:tt)*) => {{
//         use core::fmt::Write;
//         // TERMINAL.lock().write_fmt(format_args!($($arg)*)).unwrap();
//     }};
// }

// impl fmt::Write for Screen {
//     // To support the rust formatting system and use the write! macro, the write_str method must be supported
//     fn write_str(&mut self, s: &str) -> fmt::Result {
//         self.write_string(s);
//         Ok(())
//     }
// }
