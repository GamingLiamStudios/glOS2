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

pub fn printhex_u8(state: &mut VGAState, mut n: u8) {
    let mut buf = [0u8; 2];
    for d in buf.iter_mut().rev() {
        let digit = n & 0xf;
        n >>= 4;
        *d = match digit {
            0..=9 => b'0' + digit,
            10..=15 => b'a' + (digit - 10),
            _ => unreachable!(),
        };
    }
    sprint(state, core::str::from_utf8(&buf).unwrap());
}

pub fn printhex_u32(state: &mut VGAState, n: u32) {
    n.to_be_bytes().iter().for_each(|b| printhex_u8(state, *b));
}
