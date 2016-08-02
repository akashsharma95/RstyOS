use spin::Mutex;
use console;
use vga;
use cpuio::Port;

static KBDUS: [u8; 59] = *b"??1234567890-=??qwertyuiop[]\n?asdfghjkl;'`?\\zxcvbnm,./?*? ?";
static KBDUS_SHIFT: [u8; 59] = *b"??!@#$%^&*()_+??QWERTYUIOP{}\n?ASDFGHJKL:\"~?|ZXCVBNM<>??*? ?";

// State of Modifier keys
pub static STATE: Mutex<Modifiers> = Mutex::new(Modifiers {
    shift: false,
    ctrl: false,
    alt: false,
    caps: false,
});

pub struct Modifiers {
    shift: bool,
    ctrl: bool,
    alt: bool,
    caps: bool,
}

impl Modifiers {
    pub fn update_state(&mut self, scancode: u8) {
        // Check if keydown
        match scancode {
            0x2A | 0x36 => self.shift = true,
            0x1D => self.ctrl = true,
            0x38 => self.alt = true,
            0x3A => self.caps = !self.caps,
            0xAA | 0xB6 => self.shift = false,
            0x9D => self.ctrl = false,
            0xB8 => self.alt = false,
            _ => {}
        }
    }
}

pub struct Keyboard;

pub enum KeyChar {
    Some(char),
    Backsp,
    None
}

impl Keyboard {
    pub fn kbdintr() {
        console::consoleintr();
    }
    pub fn kbdgetchar() -> KeyChar {
        let kbd = unsafe { Port::new(0x60) };
        let scancode = kbd.read();
        STATE.lock().update_state(scancode);
        if scancode <= 59 {
            let state = STATE.lock();
            if scancode == 14 {
                return KeyChar::Backsp;
            } else if state.shift ^ state.caps {
                return KeyChar::Some(KBDUS_SHIFT[scancode as usize] as char);
            } else {
                return KeyChar::Some(KBDUS[scancode as usize] as char);
            }
        }
        KeyChar::None
    }

}
