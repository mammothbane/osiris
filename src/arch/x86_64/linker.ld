ENTRY(start)

SECTIONS {
    . = 1M;

    .rodata.boot :
    {
        /* make sure multiboot is at the beginning */
        KEEP(*(.multiboot_header))
        *(.boot_rodata .boot_rodata.*)
        . = ALIGN(4K);
    }

    .text.boot :
    {
        *(.boot_text .boot_text.*)
        . = ALIGN(4K);
    }

    .data.boot :
    {
        *(.boot_data .boot_data.*)
        . = ALIGN(4K);
    }

    .bss.boot :
    {
        *(.boot_bss .boot_bss.*)
        . = ALIGN(4K);
    }

    .got.boot :
    {
        *(.boot_got)
        . = ALIGN(4K);
    }

    .got.plt.boot :
    {
        *(.boot_got.plt)
        . = ALIGN(4K);
    }

    .data.rel.ro.boot : ALIGN(4K)
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
