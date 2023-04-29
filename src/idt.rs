use core::arch::asm;

use crate::gdt::{SegmentSelector, KERNEL_CS};

enum IdtDescriptorType {
    InterruptGate = 0b1110,
    TrapGate = 0b1111,
}

struct IdtDescriptor {
    segsel: u16,
    offset: u32,
    flags: u8,
}

impl IdtDescriptor {
    fn new(segsel: SegmentSelector, offset: u32, r#type: IdtDescriptorType, privl: u8) -> Self {
        assert!(privl < 4);

        let mut flags = privl << 5;
        flags |= r#type as u8;
        flags |= 0b10000000; // Present bit

        Self {
            segsel: segsel.into(),
            offset,
            flags,
        }
    }

    fn write(&self, idt: &mut [u8]) {
        idt[0] = self.offset as u8;
        idt[1] = (self.offset >> 8) as u8;
        idt[6] = (self.offset >> 16) as u8;
        idt[7] = (self.offset >> 24) as u8;

        let ss: u16 = self.segsel;
        idt[2] = ss as u8;
        idt[3] = (ss >> 8) as u8;

        idt[4] = 0;
        idt[5] = self.flags;
    }
}

// IDT.
pub static mut IDT: [u8; 8 * 256] = [0; 8 * 256];

macro_rules! naked_function {
    ($name:ident { $($inner:tt)* }) => {
        #[naked]
        #[no_mangle]
        pub unsafe extern "C" fn $name() {
            asm!(
                "pushad",
                concat!("call ", stringify!($name), "_entry"),
                "popad",
                "iret",
                options(noreturn) // butternut is a master of psychological manipulation
            )
        }
        paste::paste! {
            #[no_mangle]
            fn [<$name _entry>]() {
                $($inner)*
            }
        }
    }
}

naked_function!(general_exception {
    panic!("Unimplemented Exception");
});

naked_function!(general_interrupt {
    panic!("Unimplemented Interrupt");
});

naked_function!(skip_interrupt {});

#[no_mangle]
#[inline(never)]
pub fn init_idt() {
    let ge = IdtDescriptor::new(
        KERNEL_CS,
        general_exception as usize as u32,
        IdtDescriptorType::TrapGate,
        0,
    );
    let gi = IdtDescriptor::new(
        KERNEL_CS,
        general_interrupt as usize as u32,
        IdtDescriptorType::InterruptGate,
        0,
    );
    let si = IdtDescriptor::new(
        KERNEL_CS,
        skip_interrupt as usize as u32,
        IdtDescriptorType::InterruptGate,
        0,
    );

    unsafe {
        // Panic on unknown interrupts
        for _ in 0..256 {
            gi.write(&mut IDT);
        }

        // Intel Reserved Exceptions
        for i in 0..32 {
            ge.write(&mut IDT[8 * i..]);
        }

        // According to the OSDev Wiki, the PICs remap IRQs 8-15 to 0x70-78.
        // but 8-15 overlap with the CPU exceptions? I'll just leave 'em out for now.

        let mut idt = [0u8; 6];
        core::ptr::copy(
            (IDT.as_ptr() as u32).to_ne_bytes().as_ptr(),
            idt[2..].as_mut_ptr(),
            4,
        );
        core::ptr::copy(IDT.len().to_ne_bytes().as_ptr(), idt[..2].as_mut_ptr(), 2);

        asm!("lidt [{}]", "sti", in(reg) &idt);
    }
}
