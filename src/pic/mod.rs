use x86::io;
pub use x86::io::inb;
pub use x86::io::outb;

pub fn remap_pic() {
    unsafe {
        let pic1_mask = io::inb(0x21);
        let pic2_mask = io::inb(0xA1);

        // initialize both PICs
        io::outb(0x20, 0x11);
        io::outb(0xA0, 0x11);

        // set vector offset of pic1 to 0x20
        io::outb(0x21, 0x20);
        // set vector offset of pic2 to 0x28
        io::outb(0xA1, 0x28);

        // tell PIC1 about PIC2 at IRQ2 (0000 0100)
        io::outb(0x21, 4);

        // tell PIC2 its cascade identity (0000 0010)
        io::outb(0xA1, 2);

        // set both PICs to 8086 mode
        io::outb(0x21, 0x1);
        io::outb(0xA1, 0x1);

        // restore masks
        io::outb(0x21, pic1_mask);
        io::outb(0xA1, pic2_mask);
    }
}

pub fn eoi_for(interrupt_number: isize) {
    unsafe {
        match interrupt_number {
            i if i >= 40 => {
                outb(0xA0, 0x20);
                outb(0x20, 0x20);
            }
            32...40 => outb(0x20, 0x20),
            _ => {}
        }
    }
}
