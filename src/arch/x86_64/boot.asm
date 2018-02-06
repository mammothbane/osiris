global start
extern long_mode_start

section .text
bits 32

;;; Error table
;;;
;;; CODE    DESC
;;; 0       Multiboot magic number not found
;;; 1       CPUID not supported
;;; 2       Long mode not supported

start:
    mov esp, stack_top
    mov edi, ebx

    call check_multiboot
    call check_cpuid
    call check_long_mode

    call setup_pagetable
    call enable_paging

    lgdt [gdt64.pointer]  ; load 64-bit gdt

    jmp gdt64.code:long_mode_start


check_multiboot:
    cmp eax, 0x36d76289  ; look for multiboot magic number
    jne .no_multiboot
    ret

.no_multiboot:
    mov al, "0"
    jmp error


check_cpuid:
    ;; check if cpuid supported by trying to flip flag bit
    pushfd
    pop eax

    mov ecx, eax
    xor eax, 1 << 21

    push eax
    popfd

    pushfd
    pop eax

    push ecx
    popfd

    cmp eax, ecx
    je .no_cpuid
    ret

.no_cpuid:
    mov al, "1"
    jmp error


check_long_mode:
    ;; check if extended processor info is available
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb .no_long_mode

    ;; test for long mode
    mov eax, 0x80000001
    cpuid
    test edx, 1 << 29
    jz .no_long_mode
    ret

.no_long_mode:
    mov al, "2"
    jmp error


setup_pagetable:
    ; recursive map p4 table
    mov eax, p4_table
    or eax, 0b11  ; present and writable
    mov [p4_table + 511 * 8], eax

    ; map the rest of the tables in
    mov eax, p3_table
    or eax, 0b11
    mov [p4_table], eax

    mov eax, p2_table
    or eax, 0b11
    mov [p3_table], eax

    mov ecx, 0

    ; map each p2 entry to a huge page
.map_p2_table:
    mov eax, 0x200000
    mul ecx
    or eax, 0b10000011 ; present, writable, huge
    mov [p2_table + ecx * 8], eax

    inc ecx
    cmp ecx, 512
    jne .map_p2_table

    ret


enable_paging:
    mov eax, p4_table
    mov cr3, eax

    ; enable pae
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; write long mode to EFER msr
    mov ecx, 0xc0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; enable paging
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ret


error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte  [0xb800a], al
    hlt


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
