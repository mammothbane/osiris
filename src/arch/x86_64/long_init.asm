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

    mov [multiboot_ptr], rdi

    extern remap_page_tables
    call remap_page_tables

    mov rdi, [multiboot_ptr]

    extern high_start
    mov rax, high_start
    call rax

section .boot_bss nobits alloc noexec write align=4
multiboot_ptr:
    resb 8
