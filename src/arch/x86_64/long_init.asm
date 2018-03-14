global long_mode_start
global alloc_frame

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

    extern osiris_init
    mov rax, osiris_init
    call rax

;; params:
;;   page index (rdi)
;;   frame index (rsi)
alloc_frame:
    push rbx
    push r10

    call p4_table
    mov rbx, rax ;; rbx gets table base then entry

    mov rax, rdi
    call p4_index
    shl rax, 3
    add rbx, rax

    cmp qword [rbx], 0 ;; if entry is 0, allocate a new page and fill it
    jne .pt3

    mov rax, [frame_ctr]
    inc qword [frame_ctr]
    call entry_for_frame
    mov [rbx], rax

.pt3:
    mov rax, rdi
    call p3_table
    mov rbx, rax

    mov rax, rdi
    call p3_index
    shl rax, 3
    add rbx, rax

    cmp qword [rbx], 0
    jne .pt2

    mov rax, [frame_ctr]
    inc qword [frame_ctr]
    call entry_for_frame
    mov [rbx], rax

.pt2:
    mov rax, rdi
    call p2_table
    mov rbx, rax

    mov rax, rdi
    call p2_index
    shl rax, 8
    add rbx, rax

    cmp qword [rbx], 0
    jne .pt1

    mov rax, [frame_ctr]
    inc qword [frame_ctr]
    call entry_for_frame
    mov [rbx], rax

.pt1:
    mov rax, rdi
    call p1_table
    mov rbx, rax

    mov rax, rdi
    call p1_index
    shl rax, 8
    add rbx, rax

    ; cmp [rbx], 0
    ; jne .over_1

    mov rax, rsi
    call entry_for_frame
    mov [rbx], rax

    pop r10
    pop rbx
    ret

;; ad hoc calling convention used in this file (below this point): rax is only parameter and return. i'm managing
;; registers so there's no overlap. it's not great but i don't want to have to care about the stack at all right now.

;; param: frame index
entry_for_frame:
    shl rax, 12
    or rax, 0x3
    ret

p4_index:
    shr rax, 27
    and rax, 0x1ff
    ret

p3_index:
    shr rax, 18
    and rax, 0x1ff
    ret

p2_index:
    shr rax, 9
    and rax, 0x1ff
    ret

p1_index:
    ; shr rax, 0
    and rax, 0x1ff
    ret

p4_table:
    mov rax, 0xfffffffffffff000
    ret

;; regs: r8, r9
p3_table:
    mov r9, rax

    call p4_table ; get last table, lshift, append
    shl rax, 9
    mov r8, rax

    mov rax, r9
    call p4_index
    shl rax, 12

    or rax, r8

    ret

;; regs: rdx, rcx
p2_table:
    mov rdx, rax

    call p3_table
    shl rax, 9
    mov rcx, rax

    mov rax, rdx
    call p3_index
    shl rax, 12

    or rax, rcx

    ret

;; regs: r10, r11
p1_table:
    mov r10, rax

    call p2_table
    shl rax, 9
    mov r11, rax

    mov rax, r10
    call p2_index
    shl rax, 12

    or rax, r11

    ret


section .boot_data progbits alloc noexec write align=4
frame_ctr:
    align 8
    dq 0xf0000

section .boot_bss nobits alloc noexec write align=4
multiboot_ptr:
    resb 8
