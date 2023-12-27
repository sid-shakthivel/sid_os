/*
    PSF(PC Screen Font) fonts consist of header, font, and unicode information
    Glyphs are bitmaps of 8*16
*/

const PSF_MAGIC: u32 = 0x864ab572;

#[derive(Copy, Clone, Debug)]
pub struct PsfFont {
    magic: u32,
    version: u32,             // Usually 0
    pub header_size: u32,     // Offset of bitmaps
    flags: u32,               // 0 If there isn't a unicode table
    glymph_num: u32,          // Number of glyghs
    pub bytes_per_glyph: u32, // Size of each glygh
    height: u32,              // In pixels
    width: u32,               // In pixels
}

#[derive(Clone, Copy)]
pub struct Font {
    pub metadata: &'static PsfFont,
    pub start_addr: u32,
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

pub fn get_font_data() -> (u32, *const PsfFont) {
    // Setup font
    let font_end = unsafe { &_binary_font_psf_end as *const _ as u32 };
    let font_size = unsafe { &_binary_font_psf_size as *const _ as u32 };
    let font_start = font_end - font_size;

    (font_start, font_start as *const PsfFont)
}

extern "C" {
    pub(crate) static _binary_font_psf_end: usize;
    pub(crate) static _binary_font_psf_size: usize;
}
