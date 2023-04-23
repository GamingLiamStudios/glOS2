/* The bootloader will look at this image and start execution at the symbol
   designated as the entry point. */
ENTRY(_start)

/* Tell where the various sections of the object files will be put in the final
   kernel image. */
SECTIONS
{
	/* Begin putting sections at 1 MiB, a conventional place for kernels to be
	   loaded at by the bootloader. */
	. = 1M;

	/* The multiboot header. */
    .rodata : ALIGN(4K)
    {
        KEEP(*(.multiboot))
    }
 
	/* The kernel code */
	.text : ALIGN(4K)
	{
		*(.text)
	}
 
	/* Read-only data. */
	.rodata : ALIGN(4K)
	{
		*(.rodata)
	}
 
	/* Read-write data (initialized) */
	.data : ALIGN(4K)
	{
		*(.data)
	}
 
	/* Read-write data (uninitialized) and stack */
	.bss : ALIGN(4K)
	{
        _BSS_START = .;
		*(COMMON)
		*(.bss)
	}

    STACK_TOP = _BSS_START + 16K;
 
	/* The compiler may produce other sections, by default it will put them in
	   a segment with the same name. Simply add stuff here as needed. */
    .comment : ALIGN(4K)
    {
        *(.comment)
    }
}