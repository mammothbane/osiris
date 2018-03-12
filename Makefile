arch ?= x86_64
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
asm_src := $(wildcard src/arch/$(arch)/*.asm)
asm_obj := $(patsubst src/arch/$(arch)/%.asm, \
	build/arch/$(arch)/%.o, $(asm_src))

rust_src := $(wildcard src/**/*.rs)

target ?= $(arch)-osiris
rust_os := target/$(target)/debug/libosiris.a

.PHONY: all clean run iso kernel gdb debug int

all: $(kernel)

clean:
	@rm -rf build
	@xargo clean

debug: $(iso)
	@qemu-system-x86_64 -cdrom $(iso) -s -S

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso) -s

int: $(iso)
	@qemu-system-x86_64 -cdrom $(iso) -d int -no-reboot

gdb: $(kernel)
	@rust-os-gdb/bin/rust-gdb "$(kernel)" -ex "target remote :1234"

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(kernel): kernel $(rust_os) $(asm_obj) $(linker_script)
	@ld -n --gc-sections -T $(linker_script) -o $(kernel) $(asm_obj) $(rust_os)

# Note: this produces $(rust_os) but we're keeping it separate to force Xargo to rebuild every time
kernel:
	@xargo build --target $(target)

build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 -Fdwarf $< -o $@
