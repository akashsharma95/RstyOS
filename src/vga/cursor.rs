use cpuio::Port;

const CRT_PORT: u16 = 0x3D4;

pub fn initialize() {
    let mut crt_index: Port<u8> = unsafe { Port::new(CRT_PORT) };
    let mut crt_io: Port<u8> = unsafe { Port::new(CRT_PORT + 1) };

    crt_index.write(0b1010);
    crt_io.write(0b0000);

    crt_index.write(0b1011);
    crt_io.write(0b1111);
}

pub fn set(position: u16) {
    let mut crt_index: Port<u8> = unsafe { Port::new(CRT_PORT) };
    let mut crt_io: Port<u8> = unsafe { Port::new(CRT_PORT + 1) };

    crt_index.write(0b1111);
    crt_io.write(position as u8);

    crt_index.write(0b1110);
    crt_io.write((position >> 8) as u8);
}
