ENTRY(start)

SECTIONS {
    . = 1M;

    .boot.rodata :
    {
        /* make sure multiboot is at the beginning */
        KEEP(*(.multiboot_header))
        *(.boot_rodata .boot_rodata.*)
        . = ALIGN(4K);
    }

    .boot.text :
    {
        *(.boot_text .boot_text.*)
        . = ALIGN(4K);
    }

    .boot.data :
    {
        *(.boot_data .boot_data.*)
        . = ALIGN(4K);
    }

    .boot.bss :
    {
        *(.boot_bss .boot_bss.*)
        . = ALIGN(4K);
    }

    .boot.got :
    {
        *(.boot_got)
        . = ALIGN(4K);
    }

    .boot.got.plt :
    {
        *(.boot_got.plt)
        . = ALIGN(4K);
    }

    .boot.data.rel.ro : ALIGN(4K)
    {
        *(.boot_data.rel.ro.local*) *(.boot_data.rel.ro .boot_data.rel.ro.*)
        . = ALIGN(4K);
    }


    /* main kernel */

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

    .data :
    {
        *(.data .data.*)
        . = ALIGN(4K);
    }

    .bss :
    {
        *(.bss .bss.*)
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
