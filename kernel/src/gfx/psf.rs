/*
    PSF(PC Screen Font) fonts consist of header, font, and unicode information
    Glyphs are bitmaps of 8*16
*/

const PSF_MAGIC: u32 = 0x864ab572;

#[derive(Copy, Clone, Debug)]
struct PsfFont {
    magic: u32,
    version: u32,         // Usually 0
    header_size: u32,     // Offset of bitmaps
    flags: u32,           // 0 If there isn't a unicode table
    glymph_num: u32,      // Number of glyghs
    bytes_per_glyph: u32, // Size of each glygh
    height: u32,          // In pixels
    width: u32,           // In pixels
}

struct Font {
    metadata: &'static PsfFont,
    start_address: u32,
}

impl PsfFont {
    pub fn verify(&self) {
        assert!(
            self.magic == PSF_MAGIC,
            "PsfFont magic is not {}",
            PSF_MAGIC
        );

        assert!(self.version == 0, "PsfFont version is not 0");

        assert!(
            self.bytes_per_glyph == 16,
            "PsfFont bytes per glyph is not 16"
        );

        assert!(self.height == 16, "PsfFont has not height of 16");

        assert!(self.width == 8, "PsfFont has not width of 8");
    }
}

// pub fn draw_string(string: &str, mut x_base: u64, y_base: u64) {
//     let buffer_p = self.buffer as *mut u32;

//     for character in string.as_bytes() {
//         unsafe {
//             if let Some(font) = FONT {
//                 let glyph_address = (FONT_START
//                     + font.header_size
//                     + (font.bytes_per_glyph * (character.clone() as u32)))
//                     as *mut u8;

//                 for cy in 0..16 {
//                     let mut index = 8;
//                     for cx in 0..8 {
//                         let adjusted_x = cx + x_base;
//                         let adjusted_y = cy + y_base;

//                         // Load correct bitmap for glyph
//                         let glyph_offset: u16 =
//                             (*glyph_address.offset(cy as isize) as u16) & (1 << index);
//                         if glyph_offset > 0 {
//                             *buffer_p.offset((adjusted_y * 4096 + adjusted_x) as isize) = 0x01;
//                         }
//                         index -= 1;
//                     }
//                 }

//                 x_base += 8;
//             }
//         }
//     }
// }

pub fn init() {
    // Setup font
    let font_end = unsafe { &_binary_font_psf_end as *const _ as u32 };
    let font_size = unsafe { &_binary_font_psf_size as *const _ as u32 };
    let font_start = font_end - font_size;

    unsafe {
        FONT_TEST = Some(*(font_start as *const PsfFont));
        FONT_START = font_start;
        FONT_TEST.unwrap().verify();
    }
}

extern "C" {
    pub(crate) static _binary_font_psf_end: usize;
    pub(crate) static _binary_font_psf_size: usize;
}

static mut FONT_TEST: Option<PsfFont> = None;
static mut FONT_START: u32 = 0;

// pub static FONT
