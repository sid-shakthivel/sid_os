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
const MOUSE_COLOUR: u32 = 0x00;

/*
    Keep a stack of all windows
    Keep a reference to the current window (useful for mouse input)
*/
pub struct WindowManager<'a> {
    windows: Stack<Window>,
    dr_windows: Queue<Rect>,
    dr_background: Queue<Rect>,
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
            dr_windows: Queue::<Rect>::new(),
            dr_background: Queue::<Rect>::new(),
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

    pub fn handle_mouse_event(&mut self, new_mouse_coords: (i16, i16), is_left_click: bool) {
        // Ensure mouse is within the screen
        let new_x = new_mouse_coords.0 as u16;
        let new_y = new_mouse_coords.1 as u16;

        if !(new_x < SCREEN_WIDTH && new_x > 0 && new_y < SCREEN_HEIGHT && new_y > 0) {
            return;
        }

        let old_mouse_rect = self.generate_mouse_rect();

        // self.dr_windows.enqueue(old_mouse_rect);

        match self.current_state {
            WMState::Idle => {
                if is_left_click {
                    let index = self.find_window_under_mouse();

                    if (index > -1) {
                        self.current_state = WMState::Select;
                        self.raise(index);
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
                    self.move_window();
                } else {
                    self.current_state = WMState::Idle;
                }
            }
        }

        self.mouse_coords.0 = new_mouse_coords.0 as u16;
        self.mouse_coords.1 = new_mouse_coords.1 as u16;

        let mouse_rect = self.generate_mouse_rect();

        let mouse_rect = mouse_rect.paint(MOUSE_COLOUR, self.fb_addr);

        self.dr_windows.empty();
    }

    fn move_window(&mut self) {
        if let Some(current_window) = self.selected_window {
            let new_x = (self.mouse_coords.0).wrapping_sub(self.drag_offset.0);
            let new_y = (self.mouse_coords.1).wrapping_sub(self.drag_offset.1);

            /*
               The affected area when dragging a window the area of:
               the frame of the old window
               the frame of the new window
               this total area needs to be clipped from
            */
            let drag_total_area = self.generate_drag_total_area(&current_window, new_x, new_y);

            // self.dr_windows.enqueue(drag_total_area);
            // Rect::split_rects(clipping_rects, &clipping_rect);

            let window_ptr = unsafe { &mut *(self.windows.list.head.unwrap()) };
            window_ptr.payload.x = new_x;
            window_ptr.payload.y = new_y;
        }
    }

    fn paint_dirty_windows(&mut self) {
        // Paint dirty window against self
    }

    fn generate_mouse_rect(&self) -> Rect {
        rect::Rect::new(
            self.mouse_coords.1,
            self.mouse_coords.1 + MOUSE_HEIGHT,
            self.mouse_coords.0 + MOUSE_WIDTH,
            self.mouse_coords.0,
        )
    }

    fn find_window_under_mouse(&mut self) -> isize {
        for (index, window) in self.windows.list.into_iter().enumerate() {
            let win = window.payload;

            if win
                .generate_rect()
                .coord_does_intersect((self.mouse_coords.0, self.mouse_coords.1))
            {
                self.selected_window = Some(window.payload);
                self.drag_offset.0 = self.mouse_coords.0 - win.x;
                self.drag_offset.1 = self.mouse_coords.1 - win.y;

                return index as isize;
            }
        }
        -1
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

    fn raise(&mut self, index: isize) {
        // Only need to raise window if not already head
        if (index > 0) {
            let remove_data = self.windows.list.remove_at(index as usize).unwrap();
            let current_window = self.selected_window.unwrap();
            kfree(remove_data.1);
            self.windows.push(current_window.clone());
        }
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

    fn generate_drag_total_area(&self, window: &Window, new_x: u16, new_y: u16) -> Rect {
        let top = new_y.min(window.y);
        let bottom = (new_y + window.height).max(window.y + window.height);
        let left = new_x.min(window.x);
        let right = (new_x + window.width).max(window.x + window.width);

        Rect::new(top, bottom, left, right)
    }

    pub fn paint(&mut self) {
        self.dr_windows.enqueue(self.area);
        self.paint_background();
        self.dr_windows.empty();

        self.paint_windows();
    }

    /*
       These two methods are called stricly for initialisation
       Initially there are no dirty rectangles
    */
    fn paint_windows(&mut self) {
        // Go through each window and repaint the dirty regions appropriately
        for (index, w_window) in self.windows.list.into_iter().enumerate() {
            let window = w_window.payload;

            self.dr_windows.enqueue(window.generate_rect());

            let windows_above = self.get_above_windows(index);
            for window in windows_above.list.into_iter() {
                let mut clipping_rect = window.payload.generate_rect();
                Rect::split_rect_list(&mut clipping_rect, &mut self.dr_windows);
            }

            for rect in self.dr_windows.list.into_iter() {
                rect.payload.paint(0xFFBBBBBB, self.fb_addr);
            }

            self.dr_windows.empty();
        }
    }

    fn paint_background(&mut self) {
        // Go through each window and split against to find appropriate sections
        for w_window in self.windows.list.into_iter() {
            let window = w_window.payload;
            let mut splitting_rect = window.generate_rect();
            Rect::split_rect_list(&mut splitting_rect, &mut self.dr_windows);
        }

        for rect in self.dr_windows.list.into_iter() {
            let rect = rect.payload;
            rect.paint(BACKGROUND_COLOUR, self.fb_addr);
        }
    }
}

pub static WM: Lock<WindowManager> = Lock::new(WindowManager::new());
