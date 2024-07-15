use super::{bitwise, spinlock::Lock};
use crate::{memory::allocator::kmalloc, print_serial};
use core::mem::size_of;

#[repr(u8)]
enum EventFlags {
    KeyPressed = 0b00000001,
    MouseLeftClicked = 0b00000010,
    MouseRightClicked = 0b00000100,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Event {
    flags: u8,
    scancode: u8,
    character: u8,
    mouse_x: usize,
    mouse_y: usize,
}

impl Event {
    pub fn clear(&mut self) {
        self.flags = 0;
        self.scancode = 0;
        self.character = 0;
        self.mouse_x = 0;
        self.mouse_y = 0;
    }
}

pub struct EventManager {
    event_addr: *mut Event,
    event_rtn_addr: *mut Event,
}

impl EventManager {
    pub fn init(&mut self) {
        let event_addr = kmalloc(size_of::<Event>()) as *mut Event;
        let event_rtn_addr = kmalloc(size_of::<Event>()) as *mut Event;

        self.event_addr = event_addr;
        self.event_rtn_addr = event_rtn_addr;

        let event_ref = unsafe { &mut *self.event_addr };
        event_ref.clear();
    }

    pub fn update_key_event(&mut self, scancode: u8, character: char) {
        let event_ref = unsafe { &mut *self.event_addr };

        event_ref.scancode = scancode;
        event_ref.character = character as u8;
        event_ref.flags = bitwise::set_bit(event_ref.flags, EventFlags::KeyPressed as u8);
    }

    pub fn update_mouse_event(
        &mut self,
        x: usize,
        y: usize,
        is_left_click: bool,
        is_right_click: bool,
    ) {
        let event_ref = unsafe { &mut *self.event_addr };
        event_ref.mouse_x = x;
        event_ref.mouse_y = y;

        if is_left_click {
            event_ref.flags = bitwise::set_bit(event_ref.flags, EventFlags::MouseLeftClicked as u8);
        }

        if is_right_click {
            event_ref.flags =
                bitwise::set_bit(event_ref.flags, EventFlags::MouseRightClicked as u8);
        }
    }

    pub fn get_event(&mut self) -> *mut Event {
        let event_ref = unsafe { &mut *self.event_addr };

        // if (event_ref.scancode != 0) {
        //     print_serial!("event: {:?}\n", event_ref);
        // }

        unsafe {
            core::ptr::write(self.event_rtn_addr, *event_ref);
        }

        event_ref.clear();

        self.event_rtn_addr
    }
}

pub static EVENT_MANAGER: Lock<EventManager> = Lock::new(EventManager {
    event_addr: core::ptr::null_mut(),
    event_rtn_addr: core::ptr::null_mut(),
});
