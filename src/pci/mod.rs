use cpuio::Port;


pub struct PCI {
	address: Port<u32>,
	data:Port<u32>,
}

struct PCIHeader {
	common: CommonHeader,
	rest: HeaderType,
}

#[allow(dead_code)]
#[repr(C,packed)]
struct CommonHeader {
	device: u16,
	vendor: u16,
	status: u16,
	command: u16,
	class: u8,
	subclass: u8,
	prog_if: u8,
	rev_id: u8,
	bist: u8,
	header_type: u8,
	latency_timer: u8,
	cache_line_size: u8,
}

#[allow(dead_code)]
#[repr(C,packed)]
struct HeaderTy0 {
	base_address: [u32; 6],
	cardbus_cis_ptr: u32,
	subsystem: u16,
	subsystem_vendor: u16,
	expansion_rom_baddr: u32,
	capabilities_ptr: u8,
	reserverd: [u8; 7],
	max_latency: u8,
	min_grant: u8,
	interrupt_pin: u8,
	interrupt_line: u8,
}

enum HeaderType {
	Basic(HeaderTy0)
}

impl PCI {
	//TODO
}