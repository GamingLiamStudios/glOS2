// SPDX-License-Identifier: LGPL-2.1-only

#![no_std]
#![no_main]
#![feature(panic_info_message)]

/* Place this in the root of any crate that exposes functions
#![warn(missing_docs)]
#![warn(missing_crate_level_docs)]
#![warn(missing_doc_code_examples)]
*/

mod gdt;
mod idt;
mod kernel;
mod vga;

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use core::sync::atomic;

#[no_mangle]
#[link_section = ".multiboot"]
pub static MULTIBOOT_HEADER: [u8; include_bytes!(concat!(env!("OUT_DIR"), "/multiboot.bin"))
    .len()] = *include_bytes!(concat!(env!("OUT_DIR"), "/multiboot.bin"));

global_asm!(
    ".global _start",
    "_start:",
    "push ebx",
    "push eax",
    "mov STACK_TOP, esp",
    "call main"
);

#[no_mangle]
extern "C" fn main(eax: u32, _ebx: u32) -> ! {
    // Check if the bootloader is multiboot2
    if eax != 0x36d76289 {
        panic!("Bootloader is not multiboot2");
    }

    vga::clear_screen();

    idt::init_idt();
    gdt::init_gdt();

    // GDT + IDT not initalized as specified by Multiboot2 2.0

    kernel::main();

    unsafe {
        asm!("cli");
        loop {
            asm!("hlt");
        }
    }
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    vga::clear_screen();

    let mut state = vga::VGAState::default();
    vga::sprint(&mut state, "Kernel panic!");

    if let Some(s) = _info.message() {
        vga::sprint(&mut state, ": ");
        vga::sprint(&mut state, s.as_str().unwrap());
    }

    loop {
        atomic::compiler_fence(atomic::Ordering::SeqCst);
    }
}
