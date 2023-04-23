use crate::vga;

pub fn main() {
    vga::clear_screen();

    let mut state = vga::VGAState::default();
    vga::sprint(&mut state, "Hello, world!\n");
}
