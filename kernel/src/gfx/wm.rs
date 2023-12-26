use super::rect::{self, Rect};
use super::window::Window;
use crate::ds::queue::Queue;
use crate::ds::stack::Stack;
use crate::print_serial;
use crate::utils::spinlock::Lock;

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
    current_window: Option<Window>,
    drag_offset: (u16, u16),
    mouse_coords: (u16, u16),
    pub fb_address: usize,
    marker: core::marker::PhantomData<&'a Window>,
}

impl<'a> WindowManager<'a> {
    pub const fn new() -> WindowManager<'a> {
        WindowManager {
            windows: Stack::<Window>::new(),
            current_window: None,
            fb_address: 0,
            mouse_coords: (384, 512),
            drag_offset: (0, 0),
            marker: core::marker::PhantomData,
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

    pub fn handle_mouse_event(&mut self, new_coords: (i16, i16), is_left_click: bool) {
        let old_mouse_x = self.mouse_coords.1;
        let old_mouse_y = self.mouse_coords.0;

        self.mouse_coords.0 = ((self.mouse_coords.0 as i16) + new_coords.0) as u16;
        self.mouse_coords.1 = ((self.mouse_coords.1 as i16) + new_coords.1) as u16;

        // LOOK VERY CAREFULLY
        let y_start = self.mouse_coords.0;
        let x_start = self.mouse_coords.1;

        // If left click => raise
        if is_left_click {
            let index = self.find_mouse_over_window();

            if (index > -1) {
                // self.windows.list.remove_at(index as usize);
                // let current_window = self.current_window.unwrap();
                // self.windows.push(current_window.clone());
            }
        }

        // print_serial!("NEW mouse coords {} {}\n", x_start, y_start);

        if self.current_window.is_some() && is_left_click {
            let current_window = self.current_window.unwrap();
            let new_x = self.mouse_coords.1 - self.drag_offset.1;
            let new_y = self.mouse_coords.0 - self.drag_offset.0;

            // new clipping rect
            let clipping_rect = Rect::new(
                new_y,
                new_y + current_window.height,
                new_x + current_window.width,
                new_x,
            );

            let mut test_rect = Rect::new(
                current_window.y,
                new_y + current_window.height,
                new_x + current_window.width,
                current_window.x,
            );

            if (new_x < current_window.x) {
                test_rect.left = new_x;
                test_rect.right = current_window.x + current_window.width;
            }

            if (new_y < current_window.y) {
                test_rect.top = new_y;
                test_rect.bottom = current_window.y + current_window.height;
            }

            // These should be the dirty rects
            let mut splitted_rects = Queue::<Rect>::new();
            splitted_rects.enqueue(test_rect);

            rect::split_rects(&mut splitted_rects, &clipping_rect);

            if (new_x != 200 && new_y != 125) {
                print_serial!("{} {}\n", current_window.x, current_window.y);
                print_serial!("{} {}\n", new_x, new_y);
                print_serial!(
                    "Updated properly and these are the rects which need to be updated\n"
                );
                for rect in splitted_rects.list.into_iter() {
                    let mut rect = rect.unwrap().payload;
                    print_serial!("{:?}\n", rect);
                    rect.paint(BACKGROUND_COLOUR, self.fb_address);
                }
            }

            let window = self.windows.list.head.unwrap();
            unsafe {
                (*window).payload.x = new_x;
                (*window).payload.y = new_y;

                let rect = (*window).payload.generate_rect();

                rect.paint(WIN_BACKGROUND_COLOUR, self.fb_address);
            }

            // self.paint_dirty_background(&mut splitted_rects);
        }

        // Dirty rect the mouse
        let dirty_rect = Rect::new(old_mouse_y, old_mouse_y + 5, old_mouse_x + 5, old_mouse_x);
        let mut splitted_rects = Queue::<Rect>::new();
        splitted_rects.enqueue(dirty_rect);

        self.paint_dirty_background(&mut splitted_rects);
        self.paint_dirty_windows(1, &mut splitted_rects, true);

        for y in y_start..(y_start + 5) {
            for x in x_start..(x_start + 5) {
                let offset = ((self.fb_address as u32) + (y as u32 * 4096) + ((x as u32 * 32) / 8))
                    as *mut u32;
                unsafe {
                    *offset = 0x00;
                }
            }
        }
    }

    fn paint_dirty_background(&self, split_rects: &mut Queue<Rect>) {
        // Punch out spaces for the windows
        // for w_window in self.windows.list.into_iter() {
        //     let window = &w_window.expect("Window expected").payload;
        //     let splitting_rect = window.generate_rect();
        //     rect::split_rects(split_rects, &splitting_rect);
        // }

        // Paint the remaining area
        for rect in split_rects.list.into_iter() {
            let rect = rect.unwrap().payload;
            // print_serial!("{:?}\n", rect);
            rect.paint(BACKGROUND_COLOUR, self.fb_address);
        }
    }

    fn paint_dirty_windows(&self, index: usize, split_rects: &mut Queue<Rect>, is_mouse: bool) {
        // Will break with multiple windows
        let windows_below = self.get_below_windows(index);

        if is_mouse {
            for window in self.windows.list.into_iter() {
                let window_rect = window.unwrap().payload.clone().generate_rect();
                let mouse_rect = split_rects.list.head.unwrap();
                unsafe {
                    let test_rect = (*mouse_rect).payload.clone();

                    if (window_rect.does_intersect(&test_rect)) {
                        test_rect.paint(WIN_BACKGROUND_COLOUR, self.fb_address);
                    }
                }
            }
        } else {
            print_serial!("New painting dirty windows session:\n");
            for window in windows_below.list.into_iter() {
                let splitting_rect = window.unwrap().payload.clone().generate_rect();
                print_serial!("the splitting rect{:?}\n", splitting_rect);
                rect::split_rects(split_rects, &splitting_rect);

                for rect in split_rects.list.into_iter() {
                    let rect = rect.unwrap().payload;
                    rect.paint(WIN_BACKGROUND_COLOUR, self.fb_address);
                }
            }
        }
    }

    fn find_mouse_over_window(&mut self) -> isize {
        for (index, window) in self.windows.list.into_iter().enumerate() {
            let temp_win = window.unwrap().payload;

            if self.mouse_coords.1 >= temp_win.x
                && self.mouse_coords.1 <= (temp_win.x + temp_win.width)
                && self.mouse_coords.0 >= temp_win.y
                && self.mouse_coords.0 <= (temp_win.y + temp_win.height)
            {
                self.current_window = Some(temp_win);
                self.drag_offset.1 = self.mouse_coords.1 - temp_win.x;
                self.drag_offset.0 = self.mouse_coords.0 - temp_win.y;

                return index as isize;
            }
        }
        -1
    }

    fn paint_background(&self) {
        // Create a new rectangle of the entire screen
        let mut rect = Rect::new(0, SCREEN_HEIGHT, SCREEN_WIDTH, 0);
        let mut splitted_rects = Queue::<Rect>::new();
        splitted_rects.enqueue(rect);

        // // Punch out spaces for the windows
        for w_window in self.windows.list.into_iter() {
            let window = &w_window.expect("Window expected").payload;
            let splitting_rect = window.generate_rect();
            rect::split_rects(&mut splitted_rects, &splitting_rect);

            // print_serial!("New set of rectangles\n");
            for rect in splitted_rects.list.into_iter() {
                let rect = rect.unwrap().payload;
                // print_serial!("{:?}\n", rect);
            }
        }

        // Paint the remaining area
        for rect in splitted_rects.list.into_iter() {
            let rect = rect.unwrap().payload;
            // print_serial!("{:?}\n", rect);
            rect.paint(BACKGROUND_COLOUR, self.fb_address);
        }
    }

    fn paint_windows(&self) {
        for (index, window) in self.windows.list.into_iter().enumerate() {
            let current_window = &window.unwrap().payload;
            let windows_above = self.get_above_windows(index);

            let mut rect = current_window.generate_rect();
            let mut splitted_rects = Queue::<Rect>::new();
            splitted_rects.enqueue(rect);

            for window in windows_above.list.into_iter() {
                let splitting_rect = window.unwrap().payload.clone().generate_rect();
                rect::split_rects(&mut splitted_rects, &splitting_rect);
            }

            for rect in splitted_rects.list.into_iter() {
                let rect = rect.unwrap().payload;
                // print_serial!("{:?}\n", rect);
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

    fn get_below_windows(&self, target_index: usize) -> Queue<Window> {
        let mut windows_below = Queue::<Window>::new();

        for (index, window) in self.windows.list.into_iter().enumerate() {
            if (index > target_index) {
                windows_below.enqueue(window.unwrap().payload.clone());
            }
        }

        windows_below
    }
}

pub static WM: Lock<WindowManager> = Lock::new(WindowManager::new());
