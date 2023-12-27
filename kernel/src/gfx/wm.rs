use super::psf::{self, Font};
use super::rect::{self, Rect};
use super::window::Window;
use crate::ds::queue::Queue;
use crate::ds::stack::Stack;
use crate::memory::allocator::print_memory_list;
use crate::print_serial;
use crate::utils::spinlock::Lock;

const SCREEN_WIDTH: u16 = 1024;
const SCREEN_HEIGHT: u16 = 768;

const BACKGROUND_COLOUR: u32 = 0x3499fe;
const WIN_BACKGROUND_COLOUR: u32 = 0xFFBBBBBB;

// const WINDOW_BORDER_COLOUR: u32 = 0xFF000000;
const WINDOW_BORDER_COLOUR: u32 = 0xFF000000;
const WINDOW_TITLE_COLOUR: u32 = 0x7092be;
const WINDOW_TITLE_HEIGHT: u16 = 20;

/*
    Keep a stack of all windows
    Keep a reference to the current window (useful for mouse input)
*/
pub struct WindowManager<'a> {
    windows: Stack<Window>,
    current_window: Option<Window>,
    drag_offset: (u16, u16),
    mouse_coords: (u16, u16),
    fb_address: usize,
    current_wid: i16,
    font: Option<Font>,
    marker: core::marker::PhantomData<&'a Window>,
}

impl<'a> WindowManager<'a> {
    pub const fn new() -> WindowManager<'a> {
        WindowManager {
            windows: Stack::<Window>::new(),
            current_window: None,
            fb_address: 0,
            current_wid: 0,
            mouse_coords: (384, 512),
            drag_offset: (0, 0),
            font: None,
            marker: core::marker::PhantomData,
        }
    }

    pub fn set_fb_address(&mut self, address: usize) {
        self.fb_address = address;
    }

    pub fn set_font(&mut self) {
        let (font_start, font_ptr) = psf::get_font_data();
        self.font = Some(Font {
            metadata: unsafe { &*(font_ptr) },
            start_addr: font_start,
        });
    }

    pub fn add_window(&mut self, mut window: Window) -> i16 {
        window.wid = self.current_wid;
        self.current_wid += 1;
        self.windows.push(window);
        self.current_wid - 1
    }

    pub fn handle_mouse_event(&mut self, new_coords: (i16, i16), is_left_click: bool) {}

    pub fn paint(&self) {
        // self.paint_background();
        // self.paint_windows();
    }
}

pub static WM: Lock<WindowManager> = Lock::new(WindowManager::new());
