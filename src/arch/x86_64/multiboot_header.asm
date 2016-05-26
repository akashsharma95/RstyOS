section .multiboot_header
header_start:
        dd 0xe85250d6                ; magic number (multiboot 2)
        dd 0                         ; arch 0 is i386(32bit protected mode)
        dd header_end - header_start ; total header size
        ;; claculate checksum
        dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

        ;; required end tags
        dw 0                    ; type
        dw 0                    ; flags
        dd 8                    ; size
header_end:
