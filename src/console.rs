use vga::{self, Color, ColorCode};

use core::{self, result};
use pci;
use multiboot2;
use memory;

pub static WELCOME: &'static str = "  Welcome to RstyOS                                                             \
                                    ";
static mut CON_BUFFER: [u8; 256] = [0; 256];
static mut BUFFER_END_IDX: usize = 0;

pub fn write_to_buffer(c: u8) {
    unsafe {
        CON_BUFFER[BUFFER_END_IDX] = c;
        BUFFER_END_IDX += 1;
    }
}

pub fn pop_from_buffer() -> result::Result<(), ()> {
    unsafe {
        if BUFFER_END_IDX != 0 {
            BUFFER_END_IDX -= 1;
            Ok(())
        } else {
            Err(())
        }
    }
}

pub fn input() -> &'static [u8] {
    unsafe {
        loop {
            if BUFFER_END_IDX == 0 {
                continue;
            }
            if CON_BUFFER[BUFFER_END_IDX - 1] == b'\n' {
                let end = BUFFER_END_IDX - 1;
                BUFFER_END_IDX = 0;
                return &CON_BUFFER[0..end];
            }
        }
    }
}

pub fn shell(mb_info_addr: usize) -> ! {
    clear();
    loop {
        kprint!("> ");
        let input = core::str::from_utf8(input()).unwrap();
        match input {
            "lspci" => lspci(),
            "memarea" => memarea(mb_info_addr),
            "clear" => clear(),
            "yes" => yes(),
            e @ _ => kprintln!("Got: {}", e),
        }
    }
}

fn lspci() {
    for function in pci::functions() {
        kprintln!("{}", function);
    }
}

fn clear() {
    vga::clear_console();
    vga::BUFFER.lock().change_color(ColorCode::new(Color::Black, Color::White));
    kprint!("{}", WELCOME);
    vga::BUFFER.lock().change_color(ColorCode::new(Color::LightGreen, Color::Black));
}

fn yes() {
    loop {
        kprintln!("y");
    }
}

fn memarea(multiboot_info_addr: usize) {
    let boot_info = unsafe { multiboot2::load(multiboot_info_addr) };
    let memory_map_tag = boot_info.memory_map_tag()
        .expect("Memory map tag is required!");
    kprintln!("Memory areas : ");
    for area in memory_map_tag.memory_areas() {
        kprintln!("    start: 0x{:x}, length 0x{:x}",
                  area.base_addr,
                  area.length);
    }
    let elf_sec_tags = boot_info.elf_sections_tag()
        .expect("ELF-sections tag required");
    kprintln!("Kernel Sections: ");
    for sections in elf_sec_tags.sections() {
        kprintln!("    addr: 0x{:x}, size 0x{:x}, flags: 0x{:x}",
                  sections.addr,
                  sections.size,
                  sections.flags);
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
    kprintln!("kernel start: 0x{:x}, kernel end: 0x{:x}",
              kernel_start,
              kernel_end);
    kprintln!("multiboot_start: 0x{:x}, multiboot_end: 0x{:x}",
              multiboot_start,
              multiboot_end);
    let mut frame_allocator = memory::AreaFrameAllocator::new(kernel_start as usize,
                                                              kernel_end as usize,
                                                              multiboot_start,
                                                              multiboot_end,
                                                              memory_map_tag.memory_areas());
    for i in 0.. {
        use memory::FrameAllocator;
        if let None = frame_allocator.allocate_frame() {
            kprintln!("allocated {} frames", i);
            break;
        }
    }
}
