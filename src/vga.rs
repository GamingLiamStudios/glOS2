// SPDX-License-Identifier: LGPL-2.1-only

const VGA_BUFFER: *mut u16 = 0xb8000 as *mut u16;

#[derive(Default)]
pub struct VGAState {
    pub x: usize,
    pub y: usize,
}

pub fn clear_screen() {
    for i in 0..80 * 25 {
        unsafe {
            VGA_BUFFER.add(i).write_volatile(0x1f00 | b' ' as u16);
        }
    }
}

fn write_char(state: &mut VGAState, c: char) {
    if !c.is_ascii() {
        return;
    }

    match c {
        '\n' => {
            state.x = 0;
            state.y += 1;
        }
        _ => {
            unsafe {
                VGA_BUFFER
                    .add(state.y * 80 + state.x)
                    .write_volatile(0x1f00 | c as u16);
            }
            state.x += 1;
        }
    }
}

pub fn sprint(state: &mut VGAState, s: &str) {
    s.chars().for_each(|c| write_char(state, c));
}
