global long_mode_start

section .text
bits 64

long_mode_start:
    ; zero all data segment registers
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    call setup_pagetable
    lgdt [gdt64.pointer]  ; load 64-bit gdt

    extern osiris_init
    call osiris_init


setup_pagetable:
    ; recursive map p4 table
    mov rax, p4_table
    or rax, 0b11  ; present and writable
    mov [p4_table + 511 * 8], rax

    ; map the rest of the tables in
    mov rax, p3_table
    or rax, 0b11
    mov [p4_table], rax

    mov rax, p2_table
    or rax, 0b11
    mov [p3_table], rax

    mov rcx, 0

    ; map each p2 entry to a huge page
.map_p2_table:
    mov rax, 0x200000
    mul rcx
    or rax, 0b10000011 ; present, writable, huge
    mov [p2_table + rcx * 8], eax

    inc rcx
    cmp rcx, 512
    jne .map_p2_table

    ret


section .bss
align 4096

p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096

stack_bottom:
    resb 4096*4
stack_top:


section .rodata
gdt64:
    dq 0 ; zero entry

.code: equ $ - gdt64
    ; code segment
    ; executable, code/data desc., present, 64-bit
    dq (1 << 43) | (1 << 44) | (1 << 47) | (1 << 53)

.pointer:
    dw $ - gdt64 - 1
    dq gdt64

