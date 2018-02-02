ENTRY(start)

SECTIONS {
    . = 1M;

    .boot :
    {
        /* make sure multiboot is at the beginning */
        *(.multiboot_header)
    }

    .text :
    {
        *(.text)
    }

}
