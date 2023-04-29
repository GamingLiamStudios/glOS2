use core::arch::asm;

pub struct SegmentSelector {
    index: u16,
    use_ldt: bool,
    privl: u8, // 0-3
}

pub const KERNEL_CS: SegmentSelector = SegmentSelector {
    index: 1,
    use_ldt: false,
    privl: 0,
};

pub const KERNEL_DS: SegmentSelector = SegmentSelector {
    index: 2,
    use_ldt: false,
    privl: 0,
};

pub const USER_CS: SegmentSelector = SegmentSelector {
    index: 3,
    use_ldt: false,
    privl: 3,
};

pub const USER_DS: SegmentSelector = SegmentSelector {
    index: 4,
    use_ldt: false,
    privl: 3,
};

impl From<SegmentSelector> for u16 {
    fn from(ss: SegmentSelector) -> Self {
        let mut ret = ss.index << 3;
        ret |= (ss.use_ldt as u16) << 2;
        ret |= ss.privl as u16;
        ret
    }
}

#[repr(u8)]
enum GdtDescriptorAccess {
    Present = 1 << 7,
    Descriptor = 1 << 4,
    Executable = 1 << 3,
    // Note: Conforming bit(DCbit) allows User to execute code in Kernel's Memory, but not vice versa.
    // However the Kernel can fully access the User's Memory regardless.
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

struct GdtDescriptor {
    base: u32,
    limit: u32, // 0-1MiB:1B, 1-4GiB:4K
    index: SegmentSelector,
    access: u8,
    flags: u8, // flags << 4
}

impl GdtDescriptor {
    fn new_data(idx: u16, base: u32, mut limit: u32, privl: u8) -> Self {
        assert!(privl < 4);

        let mut flags = GdtDescriptorFlags::Size as u8;
        if limit > 0xFFFFF {
            limit >>= 12;
            flags |= GdtDescriptorFlags::Granularity as u8;
        }

        let mut access = privl << 5;
        access |= GdtDescriptorAccess::Present as u8;
        access |= GdtDescriptorAccess::Descriptor as u8;
        access |= GdtDescriptorAccess::RWbit as u8;
        //access |= GdtDescriptorAccess::Accessed as u8;

        let index = SegmentSelector {
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

    fn new_code(idx: u16, base: u32, mut limit: u32, privl: u8) -> Self {
        assert!(privl < 4);

        let mut flags = GdtDescriptorFlags::Size as u8;
        if limit > 0xFFFFF {
            limit >>= 12;
            flags |= GdtDescriptorFlags::Granularity as u8;
        }

        let mut access = privl << 5;
        access |= GdtDescriptorAccess::Present as u8;
        access |= GdtDescriptorAccess::Descriptor as u8;
        access |= GdtDescriptorAccess::RWbit as u8;
        access |= GdtDescriptorAccess::Executable as u8;
        //access |= GdtDescriptorAccess::Accessed as u8;

        let index = SegmentSelector {
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

    // Should technically be new_sys with r#type, but we won't be using LDTs so it's probably fine.
    fn new_tss(idx: u16, base: u32, mut limit: u32, privl: u8) -> Self {
        assert!(privl < 4);

        let mut flags = 0; //GdtDescriptorFlags::Size as u8;
        if limit > 0xFFFFF {
            flags |= GdtDescriptorFlags::Granularity as u8;
            limit >>= 12;
        }

        let mut access = privl << 5;
        access |= GdtDescriptorAccess::Present as u8;
        access |= 0x9; // 32-bit TSS (Available)

        let index = SegmentSelector {
            index: idx,
            use_ldt: false,
            privl,
        };

        Self {
            base,
            limit, // Limit should *never* be over 0xFFFFF
            index,
            access,
            flags,
        }
    }

    fn write(&self, gdt: &mut [u8]) {
        // Encode Limit
        gdt[0] = self.limit as u8;
        gdt[1] = (self.limit >> 8) as u8;
        gdt[6] = (self.limit >> 16) as u8;

        // Encode Base
        gdt[2] = self.base as u8;
        gdt[3] = (self.base >> 8) as u8;
        gdt[4] = (self.base >> 16) as u8;
        gdt[7] = (self.base >> 24) as u8;

        // Finish
        gdt[5] = self.access;
        gdt[6] |= self.flags;
    }
}

// TSS. Legacy method of Task Switching for Multitasking/Async. Still useful for some of the
// other infomation it provides, but shouldn't really be used for it's original purpose as
// AMD64 yeets it. Linux has 1 TSS per core, so I'll probably do the same, but ig that's a problem
// for future me whenever I end up implementing Multitasking.
pub static mut TSS: [u8; 0x68] = [0; 0x68];

// GDT. Mostly just legacy reasons why this is here, not really needed at all aside from
// telling the CPU where the TSSs are. Used to be used to Segment memory & manage permissions
// but now that's all done through Paging - the superior method.
// Currently only specifies Code/Data entries for Ring 0 & 3 + 1 TSS entry.
// NOTE: Add entries for other Rings if they are ever used.
pub static mut GDT: [u8; 8 * 6] = [0; 8 * 6];

#[no_mangle]
#[inline(never)]
pub fn init_gdt() {
    unsafe { asm!("cli") };

    let kern_code = GdtDescriptor::new_code(1, 0, 0xFFFFFFFF, 0);
    let kern_data = GdtDescriptor::new_data(2, 0, 0xFFFFFFFF, 0);
    let user_code = GdtDescriptor::new_code(3, 0, 0xFFFFFFFF, 3);
    let user_data = GdtDescriptor::new_data(4, 0, 0xFFFFFFFF, 3);

    unsafe {
        let tss = GdtDescriptor::new_tss(5, &TSS as *const _ as u32, TSS.len() as u32, 0);

        // Null Segment
        GDT[0..8].copy_from_slice(&[0; 8]);

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
            (GDT.as_ptr() as u32).to_ne_bytes().as_ptr(),
            gdt[2..].as_mut_ptr(),
            4,
        );
        core::ptr::copy(GDT.len().to_ne_bytes().as_ptr(), gdt[..2].as_mut_ptr(), 2);

        asm!("lgdt [{}]", in(reg) &gdt);

        // 0x08 = Kernel Code Segment (idx 0, ring0 perms)
        asm!("ljmp $0x08, $1f", "1:", options(att_syntax));

        asm!(
            "mov   ds, ax",
            "mov   es, ax",
            "mov   fs, ax",
            "mov   gs, ax",
            "mov   ss, ax",
            in("ax") core::convert::Into::<u16>::into(kern_data.index),
        );
    }
}
