global start
extern long_mode_start

section .text
bits 32
start:
    ; sets the stack pointer to our reserved memory.
    mov esp, stack_top
    
    ; move multiboot info pointer to edi
    mov edi, ebx

    ; tests some features
    call test_multiboot
    call test_cpuid
    call test_long_mode

    ; enabling paging
    call setup_page_tables

    ; maps P4 recursively
    mov eax, p4_table
    or eax, 0b11 ; present + writable
    mov [p4_table + 511 * 8], eax

    call enable_paging

    ; loads the 64-bit GDT
    lgdt [gdt64.pointer]

    ; updates selectors
    ; it's 16 because code segment starts at byte 8 of the GDT,
    ; and the data segment at byte 16 (but we're gonna use gdt64.data, which is the same)
    mov ax, gdt64.data
    mov ss, ax ; stack selector
    mov ds, ax ; data selector
    mov es, ax ; extra selector


    ; does a far jump to load cs (code selector) into 64-bit
    jmp gdt64.code:long_mode_start
    hlt

; prints `ERR: ` and the given error code to screen and hangs.
; parameter: error code (in ascii) in al.
error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte  [0xb800a], al
    hlt

; checks whether the kernel was loaded by a
; multiboot compliant bootloader.
; the bootloader must write the magic value 0x36d76289 to
; the eax register.
test_multiboot:
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret
.no_multiboot:
    mov al, "0"
    jmp error

; checks whether cpuid is available.
test_cpuid:
    pushfd               ; Store the FLAGS-register.
    pop eax              ; Restore the A-register.
    mov ecx, eax         ; Set the C-register to the A-register.
    xor eax, 1 << 21     ; Flip the ID-bit, which is bit 21.
    push eax             ; Store the A-register.
    popfd                ; Restore the FLAGS-register.
    pushfd               ; Store the FLAGS-register.
    pop eax              ; Restore the A-register.
    push ecx             ; Store the C-register.
    popfd                ; Restore the FLAGS-register.
    xor eax, ecx         ; Do a XOR-operation on the A-register and the C-register.
    jz .no_cpuid         ; The zero flag is set, no CPUID.
    ret                  ; CPUID is available for use.
.no_cpuid:
    mov al, "1"
    jmp error

; checks whether long mode can be used.
test_long_mode:
    mov eax, 0x80000000    ; Set the A-register to 0x80000000.
    cpuid                  ; CPU identification.
    cmp eax, 0x80000001    ; Compare the A-register with 0x80000001.
    jb .no_long_mode       ; It is less, there is no long mode.
    mov eax, 0x80000001    ; Set the A-register to 0x80000001.
    cpuid                  ; CPU identification.
    test edx, 1 << 29      ; Test if the LM-bit, which is bit 29, is set in the D-register.
    jz .no_long_mode       ; They aren't, there is no long mode.
    ret
.no_long_mode:
    mov al, "2"
    jmp error

; setups page tables
setup_page_tables:
    ; map first P4 entry to P3 table
    mov eax, p3_table
    or eax, 0b11 ; present + writable
    mov [p4_table], eax

    ; map first P3 entry to P2 table
    mov eax, p2_table
    or eax, 0b11 ; present + writable
    mov [p3_table], eax

    ; map each P2 entry to a huge 2MiB page
    mov ecx, 0

.map_p2_table:
    ; map ecx-th P2 entry to a huge page starting at address 2MiB * ecx
    mov eax, 0x200000  ; 2MiB
    mul ecx            ; start address of ecx-th page
    or eax, 0b10000011 ; present + writable + huge
    mov [p2_table + ecx * 8], eax ; map ecx-th entry

    inc ecx            ; increase counter
    cmp ecx, 512       ; if counter == 512, the whole P2 table is mapped
    jne .map_p2_table  ; else, map the next entry
    ret

; enables paging
enable_paging:
    ; load P4 to cr3 register (cpu uses this to access the P4 table)
    mov eax, p4_table
    mov cr3, eax

    ; enable PAE-flag in cr4 (Physical Address Extension)
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; set the long mode bit in the EFER MSR (model specific register)
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; enable paging in the cr0 register
    mov eax, cr0
    or eax, 1 << 31
    or eax, 1 << 16
    mov cr0, eax

    ret


; this sets the GDT (Global Descriptor Table)
; we chose the .rodata section here because it's initialized read-only data.
section .rodata
gdt64:
    ; In Long mode, it's not possible to actually use the GDT entries
    ; for Segmentation and thus the base and limit fields must be 0.
    dq 0 ; zero entry
.code: equ $ - gdt64 ; sets .code label to be at (here - gdt64)
    dq (1<<44) | (1<<47) | (1<<41) | (1<<43) | (1<<53) ; code segment
.data: equ $ - gdt64 ; sets ,data label to be at (here - gdt64)
    dq (1<<44) | (1<<47) | (1<<41) ; data segment

; loads GDT, passing this memory location to the ldgt (load GDT).
.pointer:
    ; $ = current address
    dw $ - gdt64 - 1 ; .pointer - gdt64 - 1 (length)
    dq gdt64

section .bss
; paging
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096
stack_bottom:
    ; reserves 4096 * 2 (one page) bytes to our stack
    resb 4096 * 2
stack_top:

