ENTRY(start)

SECTIONS {
    . = 1M;

    .rodata :
    {
        /* ensure that the multiboot header is at the beginning */
        KEEP(*(.multiboot_header))
        *(.rodata .rodata.*)
        . = ALIGN(4K);
    }

    .text : ALIGN(4K)
    {
        *(.text .text.*)
        . = ALIGN(4K);
    }

	.bss : ALIGN(4K)
    {
        *(.bss .bss.*)
        . = ALIGN(4K);
    }

    .data : ALIGN(4K)
    {
        *(.data .data.*)
        . = ALIGN(4K);
    }

    .data.rel.ro : ALIGN(4K)
    {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
        . = ALIGN(4K);
    }

	/*
    /DISCARD/ :
    {
        *(.debug*)
    }
    */
}