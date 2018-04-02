bits 64
global long_mode_start

section .boot progbits alloc exec write
align 16
bits 64
long_mode_start:
    mov ax, 0

    ; zero all data segment registers
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; point to new stack
    mov rsp, stack_top

    ; invalidate relevant stack locations
    mov rbp, 0xdeaddeaddeaddead
    mov rax, stack_top
    mov [rax], rbp

    extern osiris_main
    mov rax, osiris_main
    jmp rax

section .bss
bits 64
align 4096
    stack_bottom:
        resb 4096*4
    stack_top:
        resq 1
