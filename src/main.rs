#![no_std]
#![no_main]

mod kernel;
mod vga;

use core::arch::asm;
use core::ffi::c_void;
use core::panic::PanicInfo;
use core::sync::atomic;
use core::sync::atomic::Ordering;

#[no_mangle]
#[link_section = ".multiboot"]
pub static MULTIBOOT_HEADER: [u8; include_bytes!(concat!(env!("OUT_DIR"), "/multiboot.bin"))
    .len()] = *include_bytes!(concat!(env!("OUT_DIR"), "/multiboot.bin"));

extern "C" {
    static STACK_TOP: *const c_void; // ptr size
}

#[no_mangle]
extern "C" fn _start() -> ! {
    unsafe { asm!("mov {0}, esp", in(reg) STACK_TOP) };

    // GDT + IDT not initalized as specified by Multiboot2 2.0

    kernel::main();

    vga::clear_screen();

    let mut state = vga::VGAState::default();
    vga::sprint(&mut state, "Hello, world!\n");

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
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}
