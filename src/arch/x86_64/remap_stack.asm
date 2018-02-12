global remap_stack

bits 64

section .text

remap_stack:
    ; increase return location to point into new kernel mapping
    add [rbp + 8], rdi

    ; invalidate main's return value and base, destroying old stack
    mov rax, [rbp]

    mov rsi, 1
    shl rsi, 63
    mov [rax],     rsi
    mov [rax + 8], rsi

    ret