arch ?= x86_64
target ?= $(arch)-unknown-linux-gnu
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso

rust_os := target/$(target)/debug/librustyos.a
linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/*.asm)
assembly_object_files := $(patsubst src/arch/$(arch)/%.asm, \
	build/arch/$(arch)/%.o, $(assembly_source_files))

.PHONY: all clean run debug iso cargo gdb

all: $(kernel)

clean:
	@cargo clean
	@rm -rf build

logint:
	@qemu-system-x86_64 -d int -no-reboot -cdrom $(iso)

disasm:
	@objdump -D build/kernel-x86_64.bin

section:
	@objdump -h build/kernel-x86_64.bin

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso) -s

gdb:
	@rust-gdb "build/kernel-x86_64.bin" -ex "target remote :1234"

debug: $(iso)
	@qemu-system-x86_64 -cdrom $(iso) -s -S

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(kernel): cargo $(rust_os) $(assembly_object_files) $(linker_script)
	@ld -n --gc-sections -T $(linker_script) -o $(kernel) \
		$(assembly_object_files) $(rust_os)

cargo:
	@cargo build --target $(target)

# compile assembly files
build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@
