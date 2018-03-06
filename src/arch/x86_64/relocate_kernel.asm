global relocate_kernel

bits 64

section .text

;;;  params
;;;    rdi - new PTL4
;;;    rsi - kernel base
relocate_kernel:
    push rbp
    mov rbp, rsp
    mov rsp, rbp

    ; activate new page table
    mov cr3, rdi

    ; NOTE: we don't have our own stack here. rbp belongs to the frame below us, and [rsp] is the return address

    ; bump up stack to new location
    add rsp, rsi
    add rbp, rsi
    ; add [rsp], rsi
    ; add [rsp + 8], rsi

    ; rax stores traced back frame pointers.
    ; rcx stores last (higher-stack, lower-mem) frame pointer to compare to see if we should continue tracing back

    ; load current rbp into rax and rcx
    mov rax, rbp
    mov rcx, rax

    ; rdx stores invalid memory location to invalidate main return address and stack
    mov rdx, 1
    shl rdx, 63
    or  rdx, 0xdead

.loop:
    ; adjust return addr and traced frame ptr to remapped kernel location
    add [rax + 8], rsi
    add [rax], rsi

    ; follow 1 stack frame back
    mov rcx, rax
    mov rax, [rax]

    ; r8 gets the size of this stack frame
    ; mov r8, rax
    ; sub r8, rcx

    cmp rax, rsi
    je .fini
    jmp .loop

    ; if it's negative (our stack is mis-ordered), abort
    ; jl .fini

    ; otherwise, if the frame is "too big" to be a real frame (arbitrary, rule of thumb interpretation),
    ;   we've reached the bottom of the stack; exit.
    ; cmp r8, 1024*100
    ; jge .fini

    ; else loop again
    ; jmp .loop

.fini:
    ; invalidate osiris_main's return address
    mov [rcx], rdx
    mov [rcx + 8], rdx

    mov rsp, rbp
    pop rbp
    ret
