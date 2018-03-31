bits 64

global high_start

section .text
high_start:
    mov rax, 0
    mov rcx, stack_top - 32
    mov r8, 0xdeaddeaddeaddead

.stack_init_loop:
    cmp rax, 4
    jge .fini
    mov [rcx, rax*8], r8
    inc rax
    jmp .stack_init_loop

.fini:
    mov rsp, stack_top - 32
    mov rbp, stack_top - 32

    extern osiris_main
    jmp osiris_main

section .bss
stack_bottom:
    resb 4096*4
stack_top:
