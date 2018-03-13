section .multiboot_header progbits alloc noexec nowrite

header_start:
    dd 0xe85250d6                   ; magic number
    dd 0                            ; arch 0 (i386 protected)
    dd header_end - header_start    ; header length
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

    ; end tag
    dw 0
    dw 0
    dd 8

header_end:
