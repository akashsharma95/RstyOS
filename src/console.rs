use vga::{self, Color, ColorCode};
use spin::Mutex;
use core::{self, result};
use pci;
use multiboot2;
use memory;
use keyboard::{Keyboard, KeyChar};
// TODO:
//    - get lock to cons buffer to write
//    - add write_to_cons
//    - consoleintr
//    - remove ringbuffer

const BUF_SIZE: usize = 256;
pub struct ConsoleBuffer {
    buf: [u8; BUF_SIZE],
    ridx: usize,
    widx: usize,
    eidx: usize,
}

pub static CONS_BUF: Mutex<ConsoleBuffer> = Mutex::new(
    ConsoleBuffer {
        buf: [0; BUF_SIZE],
        ridx: 0,
        widx: 0,
        eidx: 0,
    }
);

pub static WELCOME: &'static str = "  Welcome to RstyOS                                                             \
                                    ";

pub fn consoleintr() {
    let mut cons = CONS_BUF.lock();
    match Keyboard::kbdgetchar() {
        KeyChar::Some(ch) => {
            if cons.widx > cons.ridx || cons.widx == 0 && cons.ridx == 0 {
                let idx = cons.widx;
                cons.buf[idx % BUF_SIZE] = ch;
                cons.widx = cons.widx + 1;
                kprint!("{}", ch as char);
            }
        },
        KeyChar::Backsp => {
            let idx = cons.widx;
            if idx != 0 {
                cons.buf[idx % BUF_SIZE] = 0;
                cons.widx = cons.widx - 1;
                vga::BUFFER.lock().backsp();
            }
        },
        KeyChar::None => {},
    }
}

pub fn consoleread(buf: &mut [u8]) {
    loop {

        let mut cons = CONS_BUF.lock();
        if cons.ridx < cons.widx {
            if cons.buf[cons.ridx % BUF_SIZE] == '\n' as u8 {
                return;
            }
            buf[cons.ridx % BUF_SIZE] = cons.buf[cons.ridx % BUF_SIZE];
            cons.ridx = cons.ridx + 1;
        }
    }
}

pub fn shell() -> ! {
    clear();
    loop {
         kprint!("> ");
         let mut buf = [0; BUF_SIZE];
         consoleread(&mut buf[..]);
         let input = core::str::from_utf8(&buf).unwrap();
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
