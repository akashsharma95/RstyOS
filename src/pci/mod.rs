use core::fmt;
use core::intrinsics::transmute;
use core::iter::Iterator;
use spin::Mutex;
use cpuio;

struct Pci {
    address: cpuio::Port<u32>,
    data: cpuio::Port<u32>,
}

impl Pci {
    unsafe fn read_config(&mut self, bus: u8, slot: u8, function: u8, offset: u8) -> u32 {
        let address: u32 = 0x80000000 | (bus as u32) << 16 | (slot as u32) << 11 |
                           (function as u32) << 8 |
                           (offset & 0b1111_1100) as u32;
        self.address.write(address);
        self.data.read()
    }

    unsafe fn probe(&mut self, bus: u8, slot: u8, function: u8) -> Option<FunctionInfo> {
        let config_0 = self.read_config(bus, slot, function, 0);
        if config_0 == 0xFFFFFFFF {
            return None;
        }

        let config_4 = self.read_config(bus, slot, function, 0x8);
        let config_c = self.read_config(bus, slot, function, 0xC);

        Some(FunctionInfo {
            bus: bus,
            device: slot,
            function: function,
            vendor_id: config_0 as u16,
            device_id: (config_0 >> 16) as u16,
            revision_id: config_4 as u8,
            subclass: (config_4 >> 16) as u8,
            class_code: DeviceClass::from_u8((config_4 >> 24) as u8),
            multifunction: config_c & 0x800000 != 0,
        })
    }
}

#[derive(Debug)]
#[repr(u8)]
#[allow(dead_code)]
pub enum DeviceClass {
    Legacy = 0x00,
    MassStorage = 0x01,
    Network = 0x02,
    Display = 0x03,
    Multimedia = 0x04,
    Memory = 0x05,
    BridgeDevice = 0x06,
    SimpleCommunication = 0x07,
    BaseSystemPeripheral = 0x08,
    InputDevice = 0x09,
    DockingStation = 0x0A,
    Processor = 0x0B,
    SerialBus = 0x0C,
    Wireless = 0x0D,
    IntelligentIO = 0x0E,
    SatelliteCommunication = 0x0F,
    EncryptionDecryption = 0x10,
    DataAndSignalProcessing = 0x11,
    Unknown,
}

impl DeviceClass {
    fn from_u8(c: u8) -> DeviceClass {
        if c <= DeviceClass::DataAndSignalProcessing as u8 {
            unsafe { transmute(c) }
        } else {
            DeviceClass::Unknown
        }
    }
}

#[derive(Debug)]
pub struct FunctionInfo {
    bus: u8,
    device: u8,
    function: u8,
    vendor_id: u16,
    device_id: u16,
    revision_id: u8,
    subclass: u8,
    class_code: DeviceClass,
    multifunction: bool,
}

impl fmt::Display for FunctionInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}.{}.{}: {:04x} {:04x} {:?} {:02x}",
               self.bus,
               self.device,
               self.function,
               self.vendor_id,
               self.device_id,
               self.class_code,
               self.subclass)
    }
}

static PCI: Mutex<Pci> = Mutex::new(Pci {
    address: unsafe { cpuio::Port::new(0xCF8) },
    data: unsafe { cpuio::Port::new(0xCFC) },
});

#[allow(dead_code)]
pub struct FunctionIterator {
    done: bool,
    bus: u8,
    device: u8,
    multifunction: bool,
    function: u8,
}

const MAX_BUS: u8 = 255;
const MAX_DEVICE: u8 = 31;
const MAX_FUNCTION: u8 = 7;

impl Iterator for FunctionIterator {
    type Item = FunctionInfo;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let mut pci = PCI.lock();
        loop {
            let result = unsafe { pci.probe(self.bus, self.device, self.function) };
            match result {
                Some(FunctionInfo { function: 0, multifunction: true, .. }) => {
                    self.multifunction = true
                }
                _ => {}
            }
            if self.multifunction && self.function < MAX_FUNCTION {
                self.function += 1;
            } else if self.device < MAX_DEVICE {
                self.function = 0;
                self.multifunction = false;
                self.device += 1;
            } else if self.bus < MAX_BUS {
                self.function = 0;
                self.multifunction = false;
                self.device = 0;
                self.bus += 1;
            } else {
                self.done = true;
                return None;
            }
            if let Some(_) = result {
                return result;
            }
        }
    }
}

#[allow(dead_code)]
pub fn functions() -> FunctionIterator {
    FunctionIterator {
        done: false,
        bus: 0,
        device: 0,
        multifunction: false,
        function: 0,
    }
}
