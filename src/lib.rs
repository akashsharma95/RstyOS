#![feature(lang_items)]
#![feature(const_fn, unique)]
#![feature(alloc, collections)]
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

#[macro_use]
mod vga_buffer;
mod memory;

mod interrupts;

#[no_mangle]
pub extern fn print_memory_areas(multiboot_info_addr: usize) {
    vga_buffer::clear_screen();
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
    let mut frame_allocator = memory::AreaFrameAllocator::new(
        kernel_start as usize, kernel_end as usize, multiboot_start,
        multiboot_end, memory_map_tag.memory_areas());
    for i in 0.. {
    	use memory::FrameAllocator;
        if let None = frame_allocator.allocate_frame() {
            println!("allocated {} frames", i);
            break;
        }
    }
}
#[no_mangle]
pub extern fn os_start() {
    use core::mem;
    use vga_buffer::WRITER;
    use vga_buffer::Color;
    
    vga_buffer::clear_screen();
    println!("RustyOS{}", "!");
    for i in 0..25 {
        unsafe {
            WRITER.lock()
                .change_color(mem::transmute_copy(&(i % 15)), Color::Black);
        }
        for _ in 0..80{
            print!("\x08");
        }
        println!("");
    }
}
#[no_mangle]
pub extern fn rust_main(multiboot_info_address: usize) {
    // ATTENTION: we have a very small stack and no guard page
    os_start();
    print_memory_areas(multiboot_info_address);
    let boot_info = unsafe { multiboot2::load(multiboot_info_address) };
    enable_nxe_bit();
    enable_write_protect_bit();

    // set up guard page and map the heap pages
    memory::init(boot_info);

    // initialize our IDT
    interrupts::init();

    // provoke a page fault inside println
    println!("{:?}", unsafe{ *(0xdeadbeaf as *mut u64) = 42 });

    println!("It did not crash!");
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
#[lang = "eh_personality"] extern "C" fn eh_personality() {}

#[cfg(not(test))]
#[lang = "panic_fmt"]
extern "C" fn panic_fmt(fmt: ::core::fmt::Arguments,
                    filen: &str, line_no: u32) -> ! {
    println!("\n\nPanicked in {} at line number {}: ", filen, line_no);
    println!("    {}", fmt);
    loop{}
}
