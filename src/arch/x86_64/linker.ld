ENTRY(start)

SECTIONS {
         . = 1M; /* sets the load address of the first section to 1 MiB. Hint: Below that is VGA buffer */
         .boot :
         {
                /* ensure that multiboot header is at the beginning */
                /* grub expects to find the multiboot header at the very beginning */
                KEEP(*(.multiboot_header))
         }
          .text :
         {
                *(.text .text.*)
         }

         .rodata : {
                *(.rodata .rodata.*)
         }

         .data.rel.ro : {
                *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
         }
}
