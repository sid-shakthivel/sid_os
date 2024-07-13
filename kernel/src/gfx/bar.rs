use crate::utils::{
    rtc::{self, DateTime},
    spinlock::Lock,
};

use super::{
    psf::{FONT_HEIGHT, FONT_WIDTH},
    rect::Rect,
    SCREEN_WIDTH,
};

const TOP_BAR_HEIGHT: u16 = 30;
const TOP_BAR_COLOUR: u32 = 0x1E1E2E;
const BAR_TEXT_OFFSET: u16 = 15;

pub struct TopBar {
    rect: Rect,
    title: &'static str,
    colour: u32,
}

static mut TIME: [u8; 8] = [0; 8];
static mut DATE: [u8; 10] = [0; 10];

impl TopBar {
    pub const fn new() -> TopBar {
        TopBar {
            rect: Rect::new(0, TOP_BAR_HEIGHT, SCREEN_WIDTH, 0),
            title: "SidOS",
            colour: TOP_BAR_COLOUR,
        }
    }

    pub fn paint(&mut self, fb_addr: usize) {
        self.rect.paint(self.colour, fb_addr);
        self.paint_title(fb_addr);

        let datetime = rtc::get_current_datetime();
        self.paint_date(&datetime, fb_addr);
        self.paint_time(&datetime, fb_addr);
    }

    pub fn update_time(&mut self, fb_addr: usize) {
        let datetime = rtc::get_current_datetime();
        self.paint_time(&datetime, fb_addr);
    }

    fn paint_title(&self, fb_addr: usize) {
        let title_x = (SCREEN_WIDTH / 2) - ((self.title.as_bytes().len() as u16 * FONT_WIDTH) / 2);
        let title_y = (TOP_BAR_HEIGHT - FONT_HEIGHT) / 2;

        self.rect
            .paint_text(self.title, title_x, title_y, fb_addr, 0xffffff);
    }

    fn paint_date(&mut self, datetime: &DateTime, fb_addr: usize) {
        unsafe {
            DATE = datetime.format_date();
            let formatted_date = core::str::from_utf8(&DATE).unwrap();

            let start_x = BAR_TEXT_OFFSET;
            let title_y = (TOP_BAR_HEIGHT - FONT_HEIGHT) / 2;

            self.rect
                .paint_text(formatted_date, start_x, title_y, fb_addr, 0xffffff);
        }
    }

    fn paint_time(&mut self, datetime: &DateTime, fb_addr: usize) {
        unsafe {
            TIME = datetime.format_time();
            let formatted_time = core::str::from_utf8(&TIME).unwrap();

            let start_x =
                SCREEN_WIDTH - BAR_TEXT_OFFSET - (formatted_time.len() as u16 * FONT_WIDTH);
            let title_y = (TOP_BAR_HEIGHT - FONT_HEIGHT) / 2;

            self.rect
                .paint_text(formatted_time, start_x, title_y, fb_addr, 0xffffff);
        }
    }
}

pub static TOP_BAR: Lock<TopBar> = Lock::new(TopBar::new());
