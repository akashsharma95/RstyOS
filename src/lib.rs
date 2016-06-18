#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn, unique)]
#![feature(alloc, collections)]
#![feature(asm)]
#![feature(naked_functions)]
#![no_std]

extern crate rlibc;
extern crate spin;
extern crate multiboot2;
#[macro_use]
extern crate bitflags;
extern crate x86;
#[macro_use]
extern crate once;
extern crate bit_field;

extern crate hole_list_allocator;
extern crate alloc;
#[macro_use]
extern crate collections;
extern crate cpuio;

#[macro_use]
mod vga;
mod memory;

mod interrupts;
mod pic;
mod keyboard;
mod pci;
mod console;


#[no_mangle]
pub extern "C" fn rust_main(multiboot_info_address: usize) {
    let boot_info = unsafe { multiboot2::load(multiboot_info_address) };
    enable_nxe_bit();
    enable_write_protect_bit();
    pic::remap_pic();
    vga::initialize();
    memory::init(boot_info);
    interrupts::init();
    unsafe {
        x86::irq::enable();
    }

    vga::clear_console();
    console::shell(multiboot_info_address);
}

fn enable_nxe_bit() {
    use x86::msr::{IA32_EFER, rdmsr, wrmsr};

    let nxe_bit = 1 << 11;
    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | nxe_bit);
    }
}

fn enable_write_protect_bit() {
    use x86::controlregs::{cr0, cr0_write};

    let wp_bit = 1 << 16;
    unsafe { cr0_write(cr0() | wp_bit) };
}

#[cfg(not(test))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[cfg(not(test))]
#[lang = "panic_fmt"]
extern "C" fn panic_fmt(fmt: ::core::fmt::Arguments, filen: &str, line_no: u32) -> ! {
    kprintln!("\n\nPanicked in {} at line number {}: ", filen, line_no);
    kprintln!("    {}", fmt);
    loop {}
}
