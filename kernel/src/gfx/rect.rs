use crate::ds::{list::List, queue::Queue};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rect {
    top: u16,
    bottom: u16,
    right: u16,
    left: u16,
}

impl Rect {
    pub const fn new(top: u16, bottom: u16, right: u16, left: u16) -> Rect {
        Rect {
            top,
            bottom,
            right,
            left,
        }
    }

    pub fn paint(&self, colour: u32, fb_address: usize) {
        for y in self.top..self.bottom {
            for x in self.left..self.right {
                let offset =
                    ((fb_address as u32) + (y as u32 * 4096) + ((x as u32 * 32) / 8)) as *mut u32;
                unsafe {
                    *offset = colour;
                }
            }
        }
    }
}

/*
    Split a rectangle (which may be a window) by a splitting rectangle
*/
fn split_rect(rect: &mut Rect, split_rect: &Rect) -> Queue<Rect> {
    let mut splitted_rects: Queue<Rect> = Queue::<Rect>::new();

    // Split by left edge
    if (split_rect.left >= rect.left && split_rect.left <= rect.right) {
        splitted_rects.enqueue(Rect::new(
            rect.top,
            rect.bottom,
            split_rect.left - 1,
            split_rect.left,
        ));
        rect.left = split_rect.left;
    }

    // Split by top edge
    if (split_rect.top >= rect.top && split_rect.top <= rect.bottom) {
        splitted_rects.enqueue(Rect::new(
            rect.top,
            split_rect.bottom - 1,
            rect.right,
            rect.left,
        ));
        rect.top = split_rect.top;
    }

    // Split by right edge
    if (split_rect.right >= rect.left && split_rect.right <= rect.left) {
        splitted_rects.enqueue(Rect::new(
            rect.top,
            rect.bottom,
            rect.right,
            split_rect.right + 1,
        ));
        rect.right = split_rect.right;
    }

    // Split by bottom edge
    if (split_rect.bottom >= rect.top && split_rect.bottom <= rect.bottom) {
        splitted_rects.enqueue(Rect::new(
            split_rect.bottom + 1,
            rect.bottom,
            rect.right,
            rect.left,
        ));
        rect.bottom = split_rect.bottom;
    }

    splitted_rects
}

/*
    Splits a list of already splitted rectangles by another splitting rectangle
    MEMORY: NEED TO FREE!
*/
pub fn split_rects(rects: &mut Queue<Rect>, splitting_rect: &Rect) {
    // for (index, rect) in rects.list.into_iter().enumerate() {
    //     let mut working_rect = rect.expect("Rect: Expected rect").payload;
    //     let mut splitted_rects = split_rect(&mut working_rect, splitting_rect);
    //     rects.list.remove_at(index);

    //     for rect in splitted_rects.list.into_iter() {
    //         rects.enqueue(rect.unwrap().payload);
    //     }
    // }

    for i in 0..rects.list.length {
        let mut working_rect = rects.get_element(i).unwrap();
        rects.list.remove_at(working_rect.0);
        let mut splitted_rects = split_rect(&mut working_rect.1, splitting_rect);

        for rect in splitted_rects.list.into_iter() {
            rects.enqueue(rect.unwrap().payload);
        }
    }
}
