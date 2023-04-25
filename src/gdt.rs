use core::arch::asm;

use crate::vga;

#[repr(u8)]
enum GdtDescriptorAccess {
    Present = 1 << 7,
    Descriptor = 1 << 4,
    Executable = 1 << 3,
    DCbit = 1 << 2,
    RWbit = 1 << 1,
    Accessed = 1 << 0,
}

#[repr(u8)]
enum GdtDescriptorFlags {
    Granularity = 1 << 7,
    Size = 1 << 6,
    Long = 1 << 5,
}

struct GdtDescriptorIndex {
    index: u16,
    use_ldt: bool,
    privl: u8, // 0-3
}

struct GdtDescriptor {
    base: u32,
    limit: u32, // 0-1MiB:1, 1-4GiB:4K
    index: GdtDescriptorIndex,
    access: u8,
    flags: u8, // flags << 4
}

impl GdtDescriptor {
    fn create(idx: u16, base: u32, mut limit: u32, mut access: u8, privl: u8) -> Self {
        assert!(privl < 4);

        let mut flags = GdtDescriptorFlags::Size as u8;
        if limit > 0xFFFFF {
            limit >>= 12;
            flags |= GdtDescriptorFlags::Granularity as u8;
        }

        access &= 0b1001_1111;
        access |= privl << 5;

        access |= GdtDescriptorAccess::Present as u8;
        access |= GdtDescriptorAccess::Descriptor as u8;
        //access |= GdtDescriptorAccess::Accessed as u8;

        let index = GdtDescriptorIndex {
            index: idx,
            use_ldt: false,
            privl,
        };

        Self {
            base,
            limit,
            index,
            access,
            flags,
        }
    }

    fn write(&self, gdt: &mut [u8]) {
        // Encode Limit
        gdt[0] = (self.limit & 0xFF) as u8;
        gdt[1] = ((self.limit >> 8) & 0xFF) as u8;
        gdt[6] = ((self.limit >> 16) & 0xF) as u8;

        // Encode Base
        gdt[2] = (self.base & 0xFF) as u8;
        gdt[3] = ((self.base >> 8) & 0xFF) as u8;
        gdt[4] = ((self.base >> 16) & 0xFF) as u8;
        gdt[7] = ((self.base >> 24) & 0xFF) as u8;

        // Finish
        gdt[5] = self.access;
        gdt[6] |= self.flags;
    }
}

pub static mut TSS: [u8; 0x68] = [0; 0x68];
pub static mut GDT: [u8; 8 * 6] = [0; 8 * 6];

#[no_mangle]
#[inline(never)]
pub fn init_gdt() {
    unsafe { asm!("cli") };

    // Conforming bit(DCbit) allows User to execute code in Kernel's Memory, but not vice versa.
    // However the Kernel can fully access the User's Memory.

    let null = GdtDescriptor::create(0, 0, 0, 0, 0);
    let kern_code = GdtDescriptor::create(
        1,
        0,
        0xFFFFFFFF,
        GdtDescriptorAccess::Executable as u8 | GdtDescriptorAccess::RWbit as u8,
        0,
    );
    let kern_data = GdtDescriptor::create(2, 0, 0xFFFFFFFF, GdtDescriptorAccess::RWbit as u8, 0);
    let user_code = GdtDescriptor::create(
        3,
        0,
        0xFFFFFFFF,
        GdtDescriptorAccess::Executable as u8 | GdtDescriptorAccess::RWbit as u8,
        3,
    );
    let user_data = GdtDescriptor::create(4, 0, 0xFFFFFFFF, GdtDescriptorAccess::RWbit as u8, 3);

    unsafe {
        let tss =
            GdtDescriptor::create(5, &TSS as *const _ as u32, TSS.len() as u32, 0b1000_1001, 0);

        // Null Segment
        null.write(&mut GDT[0..8]);

        // Kernel (Ring 0) Segments
        kern_code.write(&mut GDT[8..16]);
        kern_data.write(&mut GDT[16..24]);

        // User (Ring 3) Segments
        user_code.write(&mut GDT[24..32]);
        user_data.write(&mut GDT[32..40]);

        // TSS
        tss.write(&mut GDT[40..48]);

        // Load GDT
        let mut gdt = [0u8; 6];
        core::ptr::copy(
            (GDT.as_ptr() as u32).to_le_bytes().as_ptr(),
            gdt[2..].as_mut_ptr(),
            4,
        );
        core::ptr::copy(GDT.len().to_le_bytes().as_ptr(), gdt[..2].as_mut_ptr(), 2);

        asm!("lgdt [{}]", in(reg) &gdt);

        // 0x08 = Kernel Code Segment (idx 0, ring0 perms)
        asm!(".code32", "ljmp $0x08, $1f", "1:", options(att_syntax));

        asm!(
            "mov   ds, ax",
            "mov   es, ax",
            "mov   fs, ax",
            "mov   gs, ax",
            "mov   ss, ax",
            in("ax") 0x10,
        )
    }
}
