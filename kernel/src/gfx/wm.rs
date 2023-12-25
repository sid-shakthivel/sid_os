use super::rect::{self, Rect};
use super::window::Window;
use crate::ds::queue::Queue;
use crate::ds::stack::Stack;

const SCREEN_WIDTH: u16 = 1024;
const SCREEN_HEIGHT: u16 = 768;

const BACKGROUND_COLOUR: u32 = 0x3499fe;
const WIN_BACKGROUND_COLOUR: u32 = 0xFFBBBBBB;

/*
    Keep a stack of all windows
    Keep a reference to the current window (useful for mouse input)
*/
pub struct WindowManager<'a> {
    windows: Stack<Window>,
    current_window: Option<&'a mut Window>,
    pub fb_address: usize,
}

impl<'a> WindowManager<'a> {
    pub const fn new() -> WindowManager<'a> {
        WindowManager {
            windows: Stack::<Window>::new(),
            current_window: None,
            fb_address: 0,
        }
    }

    pub fn set_fb_address(&mut self, address: usize) {
        self.fb_address = address;
    }

    pub fn add_window(&mut self, window: Window) {
        self.windows.push(window);
    }

    pub fn paint(&self) {
        self.paint_background();
        self.paint_windows();
    }

    fn paint_background(&self) {
        // Create a new rectangle of the entire screen
        let mut rect = Rect::new(0, SCREEN_HEIGHT, SCREEN_WIDTH, 0);
        let mut splitted_rects = Queue::<Rect>::new();
        splitted_rects.enqueue(rect);

        // Punch out spaces for the windows
        for w_window in self.windows.list.into_iter() {
            let window = &w_window.expect("Window expected").payload;
            let splitting_rect = window.generate_rect();
            rect::split_rects(&mut splitted_rects, &splitting_rect);
        }

        // Paint the remaining area
        for rect in splitted_rects.list.into_iter() {
            let rect = rect.unwrap().payload;
            rect.paint(BACKGROUND_COLOUR, self.fb_address);
        }
    }

    fn paint_windows(&self) {
        for (index, window) in self.windows.list.into_iter().enumerate() {
            
            let windows_above = self.get_above_windows(index);

            for window in windows_above.list.into_iter() {
                let rect = window.unwrap().payload.clone().generate_rect();
                rect.paint(WIN_BACKGROUND_COLOUR, self.fb_address);
            }
        }
    }

    fn get_above_windows(&self, target_index: usize) -> Queue<Window> {
        let mut windows_above = Queue::<Window>::new();

        for (index, window) in self.windows.list.into_iter().enumerate() {
            if (index == target_index) {
                break;
            }

            windows_above.enqueue(window.unwrap().payload.clone());
        }

        windows_above
    }
}
