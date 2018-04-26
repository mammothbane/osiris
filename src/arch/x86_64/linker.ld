ENTRY(osiris_main)
OUTPUT_FORMAT(elf64-x86-64)

SECTIONS {
    /* main kernel */
    . = 0xffff800000000000;

    .text :
    {
        *(.text .text.*)
        . = ALIGN(4K);
    }

    .rodata :
    {
        *(.rodata .rodata.*)
        . = ALIGN(4K);
    }

    .data :
    {
        *(.data .data.*)
        . = ALIGN(4K);
    }

    .data.rel.ro : ALIGN(4K)
    {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
        . = ALIGN(4K);
    }

    .bss :
    {
        *(.bss .bss.*)
        . = ALIGN(4K);
    }


    /* DISCARD */

    .gcc_except_table :
    {
        *(.gcc_except_table)
        . = .;
    }

    .got :
    {
        *(.got)
        . = .;
    }

    .got.plt :
    {
        *(.got.plt)
        . = .;
    }
}
