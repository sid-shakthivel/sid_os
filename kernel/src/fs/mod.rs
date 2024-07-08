use vfs::VFS;

use crate::print_serial;

mod fat;
pub mod vfs;

pub fn init(start_addr: usize) {
    let values = fat::init(start_addr);
    VFS.lock().init(values);
    VFS.free();

    // VFS.lock().print();
    // VFS.free();

    // print_serial!("attempting to find\n");

    VFS.lock().open("/TESTDIR/B");
    VFS.free();
}
