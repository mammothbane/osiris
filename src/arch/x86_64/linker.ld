ENTRY(start)

SECTIONS {
    . = 1M;

    .boot :
    {
        /* make sure multiboot is at the beginning */
        KEEP(*(.multiboot_header))
    }

    .text :
    {
        *(.text)
    }

}
