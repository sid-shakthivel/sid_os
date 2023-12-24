pub mod keyboard;
mod ps2;

pub fn init() {
    ps2::init();
}
