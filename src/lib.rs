#![feature(lang_items)]
#![feature(const_fn, unique)]
#![no_std]

extern crate rlibc;
extern crate multiboot2;
extern crate spin;

#[macro_use]
mod vga_buffer;

#[no_mangle]
pub extern fn print_memory_areas(multiboot_info_addr: usize) {
    let boot_info = unsafe { multiboot2::load(multiboot_info_addr) };
    let memory_map_tag = boot_info.memory_map_tag()
        .expect("Memory map tag is required!");
    println!("Memory areas : ");
    for area in memory_map_tag.memory_areas() {
        println!("    start: 0x{:x}, length 0x{:x}",
                 area.base_addr, area.length);
    }
    let elf_sec_tags = boot_info.elf_sections_tag()
        .expect("ELF-sections tag required");
    println!("Kernel Sections: ");
    for sections in elf_sec_tags.sections() {
        println!("    addr: 0x{:x}, size 0x{:x}, flags: 0x{:x}",
                 sections.addr, sections.size, sections.flags);
    }
    let kernel_start = elf_sec_tags.sections()
        .map(|s| s.addr)
        .min()
        .unwrap();
    let kernel_end = elf_sec_tags.sections()
        .map(|s| s.addr)
        .max()
        .unwrap();
    let multiboot_start = multiboot_info_addr;
    let multiboot_end = multiboot_start + (boot_info.total_size as usize);
    println!("kernel start: 0x{:x}, kernel end: 0x{:x}",
             kernel_start, kernel_end);
    println!("multiboot_start: 0x{:x}, multiboot_end: 0x{:x}",
             multiboot_start, multiboot_end);
}

#[no_mangle]
pub extern fn rust_main(multiboot_info_address: usize) {
    // ATTENTION: we have a very small stack and no guard page
    vga_buffer::clear_screen();
    println!("RustyOS{}", "!");
    let boot_info = unsafe { multiboot2::load(multiboot_info_address) };
    let memory_map_tag = boot_info.memory_map_tag()
        .expect("Memory map tag is required!");
    println!("Memory areas : ");
    for area in memory_map_tag.memory_areas() {
        println!("    start: 0x{:x}, length 0x{:x}",
                 area.base_addr, area.length);
    }
    let elf_sec_tags = boot_info.elf_sections_tag()
        .expect("ELF-sections tag required");
    println!("Kernel Sections: ");
    for sections in elf_sec_tags.sections() {
        println!("    addr: 0x{:x}, size 0x{:x}, flags: 0x{:x}",
                 sections.addr, sections.size, sections.flags);
    }
    let kernel_start = elf_sec_tags.sections()
        .map(|s| s.addr)
        .min()
        .unwrap();
    let kernel_end = elf_sec_tags.sections()
        .map(|s| s.addr)
        .max()
        .unwrap();
    let multiboot_start = multiboot_info_address;
    let multiboot_end = multiboot_start + (boot_info.total_size as usize);
    println!("kernel start: 0x{:x}, kernel end: 0x{:x}",
             kernel_start, kernel_end);
    println!("multiboot_start: 0x{:x}, multiboot_end: 0x{:x}",
             multiboot_start, multiboot_end);
}

#[lang = "eh_personality"] extern fn eh_personality() {}

#[lang = "panic_fmt"]
extern fn panic_fmt(fmt: ::core::fmt::Arguments,
                    filen: &str, line_no: u32) -> ! {
    println!("\n\nPanicked in {} at line number {}: ", filen, line_no);
    println!("    {}", fmt);
    loop{}
}
