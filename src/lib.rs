#![feature(lang_items)]
#![feature(const_fn, unique)]
#![no_std]

extern crate rlibc;
extern crate spin;
use core::fmt::Write;


#[macro_use]
mod vga_buffer;

#[no_mangle]
pub extern fn rust_main() {
    // ATTENTION: we have a very small stack and no guard page
    vga_buffer::clear_screen();
    println!("RustyOS{}", "!");
}

#[lang = "eh_personality"] extern fn eh_personality() {}

#[lang = "panic_fmt"]
extern fn panic_fmt() -> ! {
    let buffer_ptr = (0xb8000) as *mut _;
    let red = 0x4f;
    unsafe {
        *buffer_ptr = [b'P', red, b'a', red, b'n', red, b'i', red, b'c', red, b'!', red];
    };
    loop{}
}
