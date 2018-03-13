global long_mode_start

section .boot_text progbits alloc exec nowrite align=16
bits 64

long_mode_start:
    ; zero all data segment registers
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    extern remap_page_tables
    call remap_page_tables

    extern osiris_init
    mov rax, osiris_init
    call rax
