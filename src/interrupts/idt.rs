use x86::segmentation::{SegmentSelector};
use bit_field::BitField;

macro_rules! make_idt_entry {
    ($name:ident, $body:expr) => {{
        fn body() {
            $body
        }
        use self::idt::Entry;
        #[naked]
        unsafe extern fn $name() {
            asm!("push rbp
                  push r15
                  push r14
                  push r13
                  push r12
                  push r11
                  push r10
                  push r9
                  push r8
                  push rsi
                  push rdi
                  push rdx
                  push rcx
                  push rbx
                  push rax

                  mov rsi, rsp
                  push rsi
                  
                  call $0

                  add rsp, 8

                  pop rax
                  pop rbx
                  pop rcx
                  pop rdx
                  pop rdi
                  pop rsi
                  pop r8
                  pop r9
                  pop r10
                  pop r11
                  pop r12
                  pop r13
                  pop r14
                  pop r15
                  pop rbp

                  iretq" :: "s"(body as fn()) :: "volatile", "intel");
            intrinsics::unreachable();
        }

        Entry::new(segmentation::cs(), $name)
    }}
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Entry {
    pointer_low: u16,
    gdt_selector: SegmentSelector, // SegmentSelector before!
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

pub type HandlerFunc = unsafe extern fn();

impl Entry {
    pub fn new(gdt_selector: SegmentSelector, handler: HandlerFunc) -> Self {
        let pointer = handler as u64;
        Entry {
            gdt_selector: gdt_selector,
            pointer_low: pointer as u16,
            pointer_middle: (pointer >> 16) as u16,
            pointer_high: (pointer >> 32) as u32,
            options: EntryOptions::new(),
            reserved: 0,
        }
    }

    fn missing() -> Self {
        Entry {
            gdt_selector: SegmentSelector::new(0),
            pointer_low: 0,
            pointer_middle: 0,
            pointer_high: 0,
            options: EntryOptions::minimal(),
            reserved: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EntryOptions(BitField<u16>);
#[allow(dead_code)]
impl EntryOptions {
    fn minimal() -> Self {
        let mut options = BitField::new(0);
        options.set_range(9..12, 0b111); // 'must-be-one' bits
        EntryOptions(options)
    }

    fn new() -> Self {
        let mut options = Self::minimal();
        options.set_present(true).disable_interrupts(true);
        options
    }

    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.0.set_bit(15, present);
        self
    }

    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
        self.0.set_bit(8, !disable);
        self
    }

    pub fn set_privilege_level(&mut self, dpl: u16) -> &mut Self {
        self.0.set_range(13..15, dpl);
        self
    }

    pub fn set_stack_index(&mut self, index: u16) -> &mut Self {
        self.0.set_range(0..3, index);
        self
    }
}


pub struct Idt([Entry; 256]);

impl Idt {
    pub fn new() -> Idt {
        Idt([Entry::missing(); 256])
    }
    //  fn set_isr(&mut self, num: u8, entry: IdtEntry) {
    pub fn set_handler(&mut self, index: u8, entry: Entry) -> &mut EntryOptions {
        self.0[index as usize] = entry;
        &mut self.0[index  as usize].options
    }

    pub fn load(&'static self) {
        use x86::dtables::{DescriptorTablePointer, lidt};
        use core::mem::size_of;

        let ptr = DescriptorTablePointer {
            base: self as *const _ as u64,
            limit: (size_of::<Self>() - 1) as u16,
        };

        unsafe { lidt(&ptr) };
    }
}