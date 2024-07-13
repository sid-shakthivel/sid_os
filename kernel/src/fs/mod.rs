use vfs::VFS;

use crate::{
    memory::{allocator::kmalloc, page_frame_allocator::PAGE_FRAME_ALLOCATOR},
    print_serial,
};

mod fat;
pub mod vfs;

pub fn init(start_addr: usize) {
    let values = fat::init(start_addr);
    VFS.lock().init(values);
    VFS.free();

    // print_serial!("attempting to find\n");

    let new_file = VFS.lock().open("/a.txt");
    VFS.free();

    let buffer = kmalloc(new_file.size) as *mut u8;
    VFS.lock().read_file(&new_file, buffer, new_file.size, 0);
    VFS.free();
    print_serial!("{:?}", crate::utils::string::get_string_from_ptr(buffer));
}
