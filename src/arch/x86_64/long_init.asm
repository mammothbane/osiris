global long_mode_start

section .boot_text
bits 64

long_mode_start:
    ; zero all data segment registers
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    extern osiris_init
    call osiris_init
