mod fat;
pub mod vfs;

pub fn init(start_addr: usize) {
    fat::init(start_addr);
}
