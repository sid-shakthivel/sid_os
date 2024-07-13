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

    // VFS.lock().print();
    // VFS.free();

    // print_serial!("attempting to find\n");

    let test = VFS.lock().open("/a.txt");
    VFS.free();

    print_serial!("{:?}\n", test);

    PAGE_FRAME_ALLOCATOR.lock().alloc_page_frame();
    PAGE_FRAME_ALLOCATOR.free();

    let buffer = PAGE_FRAME_ALLOCATOR.lock().alloc_page_frames(2) as *mut u8;
    PAGE_FRAME_ALLOCATOR.free();

    // let buffer = kmalloc(test.size) as *mut u8;
    VFS.lock().read_file(&test, buffer, test.size);
    VFS.free();
    print_serial!("{:?}", crate::utils::string::get_string_from_ptr(buffer));
}
