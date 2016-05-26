        global start            ; global here exports the label and makes it pub. entrypoint start

        section .text           ; default section of executable code
        bits 32                 ; tells that the following instr are 32bit

start:  
        mov dword [0xb8000], 0x2f4b2f4f ; move constant to memory location [location]
        hlt
        
