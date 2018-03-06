ENTRY(start)

SECTIONS {
    . = 1M;

    .boot.rodata :
    {
        /* make sure multiboot is at the beginning */
        KEEP(*(.multiboot_header))
        *(.rodata_32 .rodata_32.*)
        . = ALIGN(4K);
    }

    .boot.text :
    {
        *(.text_32 .text_32.*)
        . = ALIGN(4K);
    }

    .boot.bss :
    {
        *(.bss_32 .bss_32.*)
        . = ALIGN(4K);
    }

    . = 0xffff800000000000;

    .rodata :
    {
        *(.rodata .rodata.*)
        . = ALIGN(4K);
    }

    .text :
    {
        *(.text .text.*)
        . = ALIGN(4K);
    }

    .bss :
    {
        *(.bss .bss.*)
        . = ALIGN(4K);
    }

    .data :
    {
        *(.data .data.*)
        . = ALIGN(4K);
    }

    .got :
    {
        *(.got)
        . = ALIGN(4K);
    }

    .got.plt :
    {
        *(.got.plt)
        . = ALIGN(4K);
    }

    .data.rel.ro : ALIGN(4K)
    {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
        . = ALIGN(4K);
    }

    .gcc_except_table : ALIGN(4K)
    {
        *(.gcc_except_table)
        . = ALIGN(4K);
    }
}
