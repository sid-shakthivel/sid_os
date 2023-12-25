pub mod keyboard;
pub mod mouse;
mod ps2;

pub fn init() {
    ps2::init();
}
