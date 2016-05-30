        global start            ; global here exports the label and makes it pub. entrypoint start
        extern long_mode_start
        section .text           ; default section of executable code
        bits 32                 ; tells that the following instr are 32bit

start:
        mov esp, stack_top      ; Update stack pointer reg at start.

        call check_multiboot
        call check_cpuid
        call check_long_mode

        call set_up_page_tables
        call enable_paging

        call set_up_SSE         ; Enable SSE extension
        
        ;;  load the 64-bit GDT
        lgdt [gdt64.pointer]

        ;;  update selectors
        mov ax, gdt64.data
        mov ss, ax              ; stack selector
        mov ds, ax              ; data selector
        mov es, ax              ; extra selector

        jmp gdt64.code:long_mode_start

set_up_page_tables:
        ;; Map the first P4 entry to P3 table
        mov eax, p3_table
        or eax, 0b11            ; Present + Writable bits set
        mov [p4_table], eax

        ;; Map the first P3 entry to P2 table
        mov eax, p2_table
        or eax, 0b11
        mov [p3_table], eax

        ;; 512 entries in P1 and each entry in P1 can address page 4KiB in size, therefore 2MiB
        ;; map each P2 entry to huge 2MiB P1 pages
        mov ecx, 0              ; Counter variable

.map_p2_table:
        ;;  map ecx-th P2 entry to a huge page that starts at address 2MiB*ecx
        mov eax, 0x200000       ; 2MiB
        mul ecx                 ; start address of ecx-th page
        or eax, 0b10000011      ; present + writable + huge
        mov [p2_table + ecx * 8], eax ; map ecx-th entry

        inc ecx                 ; increase counter
        cmp ecx, 512            ; if counter == 512, the whole P2 table is mapped
        jne .map_p2_table       ; else map the next entry

        ret                     ; First GB mapped as 512 entries in P2 * 2MiB is 1 GB

enable_paging:
        ;; load P4 to cr3 register
        mov eax, p4_table
        mov cr3, eax

        ;; enable PAE-flag in cr4
        mov eax, cr4
        or eax, 1 << 5
        mov cr4, eax

        ;;  set the long mode bit in the EFER MSR (model specific register)
        mov ecx, 0xC0000080
        rdmsr
        or eax, 1 << 8
        wrmsr

        ;; Enable paging in cr0 register
        mov eax, cr0
        or eax, 1 << 31
        mov cr0, eax

        ret

;; For printing error codes on any error (parameter al)
error:
        mov dword [0xb8000], 0x4f524f45
        mov dword [0xb8004], 0x4f3a4f52
        mov dword [0xb8008], 0x4f204f20
        mov byte  [0xb800a], al
        hlt


check_multiboot:
        cmp eax, 0x36d76289     ; To check if bootloader has loaded our kernel. Check the magic number.
        jne .no_multiboot       ; If not equal jump to no_multiboot label
        ret

.no_multiboot:
        mov al, "0"
        jmp error

;; This instr is used to get various information about CPU, (Copied from OSDev)
check_cpuid:
        ;; Check if CPUID is supported by attempting to flip the ID bit (bit 21)
        ;; in the FLAGS register. If we can flip it, CPUID is available.

        ;; Copy FLAGS in to EAX via stack
        pushfd
        pop eax

        ;; Copy to ECX as well for comparing later on
        mov ecx, eax

        ;; Flip the ID bit
        xor eax, 1 << 21

        ;; Copy EAX to FLAGS via the stack
        push eax
        popfd

        ;; Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
        pushfd
        pop eax

        ;; Restore FLAGS from the old version stored in ECX (i.e. flipping the
        ;; ID bit back if it was ever flipped).
        push ecx
        popfd

        ;;  Compare EAX and ECX. If they are equal then that means the bit
        ;;  wasn't flipped, and CPUID isn't supported.
        cmp eax, ecx
        je .no_cpuid
        ret
.no_cpuid:
        mov al, "1"
        jmp error

;; Now CPUID can be used to check if long-mode is supported by the CPU. 
check_long_mode:
        ;;  test if extended processor info in available
        mov eax, 0x80000000     ; implicit argument for cpuid
        cpuid                   ; get highest supported argument
        cmp eax, 0x80000001     ; it needs to be at least 0x80000001
        jb .no_long_mode        ; if it's less, the CPU is too old for long mode

        ;;  use extended info to test if long mode is available
        mov eax, 0x80000001     ; argument for extended processor info
        cpuid                   ; returns various feature bits in ecx and edx
        test edx, 1 << 29       ; test if the LM-bit is set in the D-register
        jz .no_long_mode        ; If it's not set, there is no long mode
        ret

.no_long_mode:
        mov al, "2"
        jmp error


set_up_SSE:
        ;;  check for SSE
        mov eax, 0x1
        cpuid
        test edx, 1<<25
        jz .no_SSE

        ;;  enable SSE
        mov eax, cr0
        and ax, 0xFFFB          ; clear coprocessor emulation CR0.EM
        or ax, 0x2              ; set coprocessor monitoring  CR0.MP
        mov cr0, eax
        mov eax, cr4
        or ax, 3 << 9           ; set CR4.OSFXSR and CR4.OSXMMEXCPT at the same time
        mov cr4, eax

        ret

.no_SSE:
        mov al, "a"
        jmp error


section .bss
align 4096
p4_table:
        resb 4096               ; reserves the ammount without initializing them, 8KiB doesn't need to be executable
p3_table:
        resb 4096
p2_table:
        resb 4096
stack_bottom:
        resb 64                 ; Reserve 64 bytes for stact at bss section to avoid having unnecessary large file.
stack_top:


        section .rodata
gdt64:
        dq 0                    ; zero entry we don't need segmentation
.code:  equ $ - gdt64
        dq (1<<44) | (1<<47) | (1<<41) | (1<<43) | (1<<53) ; code segment
.data:  equ $ - gdt64
        dq (1<<44) | (1<<47) | (1<<41)                     ; data segment
.pointer:
        dw $ - gdt64 - 1        ; .pointer - gdt64 - 1 is length (2 bytes)
        dq gdt64                ; finally the address (8 bytes)
