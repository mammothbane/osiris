global relocate_kernel

bits 64

section .text

;;;  params
;;;    rdi - new PTL4
;;;    rsi - kernel base
relocate_kernel:
    ; activate new page table
    mov cr3, rdi

    ; bump up stack to new location
    add rsp, rsi
    add rbp, rsi

    ; rax stores traced back frame pointers.
    ; rcx stores last (higher-stack, lower-mem) frame pointer to compare to see if we should continue tracing back
    mov rax, rbp
    mov rcx, rax

    ; rdx stores invalid memory location to invalidate main return address and stack
    mov rdx, 1
    shl rdx, 62

.loop:
    ; adjust return addr and traced frame ptr to remapped kernel location
    add [rax + 8], rsi
    add [rax], rsi

    mov rcx, rax
    mov rax, [rax]
    mov r8, rax

    ; if our stack is mis-ordered, abort
    sub r8, rcx
    jl .fini

    ; otherwise, if the frame is "too small" or "too big" (arbitrary, rule-of-thumb interpretation)
    ; also abort
    cmp r8, 24
    jle .fini

    cmp r8, 1024
    jge .fini

    ; else loop again
    jmp .loop

.fini:
    ; invalidate osiris_main's return address
    mov [rax], rdx
    mov [rax + 8], rdx

    ret
