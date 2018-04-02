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

    lidt [idt_info]

    extern high_start
    mov rax, high_start
    jmp rax

page_fault:

    hlt

section .boot_bss nobits alloc noexec write align=4
multiboot_ptr:
    resb 8

section .boot_rodata progbits alloc noexec nowrite align=4
idt_info:
    dw idt_end - idt
    dq idt

idt:
    resb 16*8

    dw page_fault & 0xffff ; offset_low
    dw 0x8 ; 0-1 (rpl); 2 (gdt: 0/ldt: 1); 3-15 table index
    db 0 ; IST index (0 is unused)
    db 10001111b ; attr/type (Present, DPL (2 bit), 0, 4-bit type)
    dw (page_fault >> 16) & 0xffff ; offset_middle
    dd (page_fault >> 32) & 0xffffffff ; offset_high
    dd 0 ; reserved
idt_end:
