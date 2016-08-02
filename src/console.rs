use vga::{self, Color, ColorCode};
use spin::Mutex;
use core::{self, result};
use pci;
use multiboot2;
use memory;
use keyboard::Keyboard;
// TODO:
//    - get lock to cons buffer to write
//    - add write_to_cons
//    - consoleintr
//    - remove ringbuffer

struct ConsoleBuffer {
    buf: [u8; 256],
    ridx: usize,
    widx: usize,
    eidx: usize,
}

static mut CONS_BUF: Mutex<ConsoleBuffer> = Mutex::new(
    ConsoleBuffer {
        buf: [0; 256],
        ridx: 0,
        widx: 0,
        eidx: 0,
    }
);

pub static WELCOME: &'static str = "  Welcome to RstyOS                                                             \
                                    ";
pub fn shell() -> ! {
    clear();
    loop {
        kprint!("> ");
        let mut buffer = [0; 128];
        let input = core::str::from_utf8(input(&mut buffer[..])).unwrap();
        match input {
            "lspci" => lspci(),
            "clear" => clear(),
            "yes" => yes(),
            e @ _ => kprintln!("Got: |{}|", e),
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
