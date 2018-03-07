global relocate_kernel

bits 64

section .text

;;;  params
;;;    rdi - new PTL4
;;;    rsi - kernel base
relocate_kernel:
    ; set up our stack for uniformity
    push rbp
    mov rbp, rsp
    mov rsp, rbp

    ; activate new page table
    mov cr3, rdi

    ; bump up stack to new location
    add rsp, rsi
    add rbp, rsi

    ; rax stores traced back frame pointers.
    ; rcx stores last (higher-stack, lower-mem) frame pointer to compare to see if we should continue tracing back

    ; load current rbp into rax and rcx
    mov rax, rbp
    mov rcx, rax

    ; rdx stores invalid memory location to invalidate main return address and stack
    mov rdx, 0x400000000000dead

.loop:
    ; adjust return addr and traced frame ptr to remapped kernel location
    add [rax + 8], rsi
    add [rax], rsi

    ; follow 1 stack frame back
    mov rcx, rax
    mov rax, [rax]

    ; pre: original saved BP was pointing to 0 -- this needs to be ensured during the boot sequence
    ; if we're at the bottom of the stack, we're done. otherwise, keep stepping back through it
    cmp rax, rsi
    je .fini
    jmp .loop

.fini:
    ; invalidate osiris_main's return address
    mov [rcx], rdx
    mov [rcx + 8], rdx

    mov rsp, rbp
    pop rbp
    ret
