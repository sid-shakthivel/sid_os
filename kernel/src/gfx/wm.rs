use super::psf::{self, Font};
use super::rect::{self, Rect};
use super::window::Window;
use crate::ds::queue::Queue;
use crate::ds::stack::Stack;
use crate::memory::allocator::{kfree, print_memory_list};
use crate::print_serial;
use crate::utils::spinlock::Lock;

use super::{BACKGROUND_COLOUR, BPP, PITCH, SCREEN_HEIGHT, SCREEN_WIDTH};

const MOUSE_WIDTH: u16 = 5;
const MOUSE_HEIGHT: u16 = 5;

/*
    Keep a stack of all windows
    Keep a reference to the current window (useful for mouse input)
*/
pub struct WindowManager<'a> {
    windows: Stack<Window>,
    selected_window: Option<Window>,
    drag_offset: (u16, u16),
    mouse_coords: (u16, u16),
    fb_addr: usize,
    current_wid: i16,
    font: Option<Font>,
    area: Rect,
    current_state: WMState,
    marker: core::marker::PhantomData<&'a Window>,
}

/*
    Idle => Select
    Select => Drag
    Drag => Idle
*/
enum WMState {
    Idle,
    Select,
    Drag,
}

impl<'a> WindowManager<'a> {
    pub const fn new() -> WindowManager<'a> {
        let rect = Rect::new(0, SCREEN_HEIGHT, SCREEN_WIDTH, 0);

        WindowManager {
            windows: Stack::<Window>::new(),
            selected_window: None,
            fb_addr: 0,
            current_wid: 0,
            mouse_coords: (512, 384),
            drag_offset: (0, 0),
            font: None,
            area: rect,
            current_state: WMState::Idle,
            marker: core::marker::PhantomData,
        }
    }

    pub fn set_fb_address(&mut self, address: usize) {
        self.fb_addr = address;
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

    pub fn handle_mouse_event(&mut self, delta_coords: (i16, i16), is_left_click: bool) {
        // Keep the old coordinates
        let old_mouse_x = self.mouse_coords.0;
        let old_mouse_y = self.mouse_coords.1;

        /*
           Ensure the mouse is within the screen
        */
        let new_x = delta_coords.0 as u16;
        let new_y = delta_coords.1 as u16;

        if !(new_x < SCREEN_WIDTH && new_x > 0 && new_y < SCREEN_HEIGHT && new_y > 0) {
            return;
        }

        self.mouse_coords.0 = new_x;
        self.mouse_coords.1 = new_y;

        let mut dirty_rects = Queue::<Rect>::new();

        let old_mouse: Rect = Rect::new(
            old_mouse_y,
            old_mouse_y + MOUSE_HEIGHT,
            old_mouse_x + MOUSE_WIDTH,
            old_mouse_x,
        );

        dirty_rects.enqueue(old_mouse);

        match self.current_state {
            WMState::Idle => {
                if is_left_click {
                    let index = self.find_window_under_mouse();

                    if (index > -1) {
                        self.raise(index);
                        self.current_state = WMState::Select;
                    }
                }
            }
            WMState::Select => {
                if is_left_click {
                    self.current_state = WMState::Drag;
                }
            }
            WMState::Drag => {
                if is_left_click {
                    self.move_window(&mut dirty_rects);
                } else {
                    self.current_state = WMState::Idle;
                }
            }
        }

        self.paint_dirty_windows(
            self.windows.list.length(),
            &mut dirty_rects,
            Some(old_mouse),
        );

        self.paint_dirty_background(&mut dirty_rects);

        dirty_rects.empty();

        self.paint_mouse();
    }

    fn paint_dirty_windows(
        &mut self,
        index: usize,
        dirty_rects: &mut Queue<Rect>,
        w_mouse_rect: Option<Rect>,
    ) {
        /*
           If repainting mouse (Some) =>
           iterate through each window and if the mouse intersects then update that region
           Else =>
           go through each window below the current window and repaint the dirty regions appropriately
        */
        match w_mouse_rect {
            Some(mouse_rect) => {
                let mut dirty_mouse_rects = Queue::<Rect>::new();
                dirty_mouse_rects.enqueue(mouse_rect);

                for window in self.windows.list.into_iter() {
                    let window = window.payload;
                    window.paint_dirty(&mut dirty_mouse_rects, self.fb_addr, &self.font.unwrap());
                }
                dirty_mouse_rects.empty();
            }
            None => {
                // This probably doesn't work TEST THIS

                panic!("This probably doesn't work TEST THIS?\n");

                let windows_below = self.get_below_windows(index);

                for w_window in windows_below.list.into_iter() {
                    let window = w_window.payload.clone();
                    let clipping_rect = window.generate_rect();
                    Rect::split_rects(dirty_rects, &clipping_rect);
                    window.paint_dirty(dirty_rects, self.fb_addr, &self.font.unwrap());
                }
            }
        }
    }

    fn paint_dirty_background(&mut self, dirty_rects: &mut Queue<Rect>) {
        for w_window in self.windows.list.into_iter() {
            let window = w_window.payload;
            let splitting_rect = window.generate_rect();
            Rect::split_rects(dirty_rects, &splitting_rect);
        }

        for rect in dirty_rects.list.into_iter() {
            let rect = rect.payload;
            rect.paint(BACKGROUND_COLOUR, self.fb_addr);
        }
    }

    fn move_window(&mut self, clipping_rects: &mut Queue<Rect>) {
        if let Some(current_window) = self.selected_window {
            let new_x = (self.mouse_coords.0).wrapping_sub(self.drag_offset.0);
            let new_y = (self.mouse_coords.1).wrapping_sub(self.drag_offset.1);

            /*
               The affected area when dragging a window the area of:
               the frame of the old window
               the frame of the new window
               this total area needs to be clipped from
            */
            let drag_total_area = self.get_drag_total_area(&current_window, new_x, new_y);
            let clipping_rect = self.get_drag_clipping_rect(&current_window, new_x, new_y);

            clipping_rects.enqueue(drag_total_area);
            Rect::split_rects(clipping_rects, &clipping_rect);

            let window_ptr = unsafe { &mut *(self.windows.list.head.unwrap()) };
            window_ptr.payload.x = new_x;
            window_ptr.payload.y = new_y;

            /*
               The current window is already at the top of the stack
               Therefore there are no windows above it (so don't need to empty)
            */
            let mut above_windows = Queue::<Window>::new();
            window_ptr
                .payload
                .paint(&mut above_windows, self.fb_addr, &self.font.unwrap());
        }
    }

    fn get_drag_clipping_rect(&self, window: &Window, new_x: u16, new_y: u16) -> Rect {
        Rect::new(new_y, new_y + window.height, new_x + window.width, new_x)
    }

    fn get_drag_total_area(&self, window: &Window, new_x: u16, new_y: u16) -> Rect {
        // Down + Right
        let mut total_area = Rect::new(
            window.y,
            new_y + window.height,
            new_x + window.width,
            window.x,
        );

        // Up
        if new_y < window.y {
            total_area.top = new_y;
            total_area.bottom = window.y + window.height;
        }

        // Left
        if new_x < window.x {
            total_area.left = new_x;
            total_area.right = window.x + window.width;
        }

        total_area
    }

    /*
       Moves a window to the top of the stack
       That window is selected
    */
    fn raise(&mut self, index: isize) {
        let remove_data = self.windows.list.remove_at(index as usize).unwrap();
        let current_window = self.selected_window.unwrap();
        kfree(remove_data.1);
        self.windows.push(current_window.clone());
    }

    fn paint_mouse(&mut self) {
        let x_start = self.mouse_coords.0;
        let y_start = self.mouse_coords.1;

        for y in y_start..(y_start + MOUSE_HEIGHT) {
            for x in x_start..(x_start + MOUSE_WIDTH) {
                let offset = ((self.fb_addr as u32)
                    + (y as u32 * PITCH as u32)
                    + ((x as u32 * BPP as u32) / 8)) as *mut u32;
                unsafe {
                    *offset = 0x00;
                }
            }
        }
    }

    pub fn paint(&self) {
        self.paint_background();
        self.paint_windows();
    }

    fn paint_windows(&self) {
        /*
           Windows must be clipped as to only paint visible regions
           This is done by clipping against windows above the current window
           Since windows is a stack, the selected window is at the top
        */
        for (index, wrapped_win) in self.windows.list.into_iter().enumerate() {
            let window = wrapped_win.payload;
            let mut windows_above = self.get_above_windows(index);
            window.paint(&mut windows_above, self.fb_addr, &self.font.unwrap());
            windows_above.empty();
        }
    }

    fn find_window_under_mouse(&mut self) -> isize {
        for (index, window) in self.windows.list.into_iter().enumerate() {
            let temp_win = window.payload;

            if self.mouse_coords.0 >= temp_win.x
                && self.mouse_coords.0 <= (temp_win.x + temp_win.width)
                && self.mouse_coords.1 >= temp_win.y
                && self.mouse_coords.1 <= (temp_win.y + temp_win.height)
            {
                self.selected_window = Some(temp_win);

                self.drag_offset.0 = self.mouse_coords.0 - temp_win.x;
                self.drag_offset.1 = self.mouse_coords.1 - temp_win.y;

                return index as isize;
            }
        }
        -1
    }

    fn paint_background(&self) {
        let mut clipped_rects = Queue::<Rect>::new();
        clipped_rects.enqueue(self.area);

        /*
           Iterate through all windows
           Perform rectangle clipping to punch out regions where windows are present
           Only visible regions of the screen should be updated
        */
        for wrapped_win in self.windows.list.into_iter() {
            let window = wrapped_win.payload;
            let splitting_rect = window.generate_rect();
            Rect::split_rects(&mut clipped_rects, &splitting_rect);
        }

        for rect in clipped_rects.list.into_iter() {
            let rect = rect.payload;
            rect.paint(BACKGROUND_COLOUR, self.fb_addr);
        }

        clipped_rects.empty();
    }

    fn get_above_windows(&self, target_index: usize) -> Queue<Window> {
        let mut windows_above = Queue::<Window>::new();

        for (index, window) in self.windows.list.into_iter().enumerate() {
            if (index == target_index) {
                break;
            }

            windows_above.enqueue(window.payload.clone());
        }

        windows_above
    }

    fn get_below_windows(&self, target_index: usize) -> Queue<Window> {
        let mut windows_below = Queue::<Window>::new();

        for (index, window) in self.windows.list.into_iter().enumerate() {
            if (index > target_index) {
                windows_below.enqueue(window.payload.clone());
            }
        }

        windows_below
    }
}

pub static WM: Lock<WindowManager> = Lock::new(WindowManager::new());
