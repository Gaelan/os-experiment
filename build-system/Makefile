arch := x86_64
project_root := ..
build_root := $(project_root)/build
qemu_system_binary := qemu-system-$(arch)
code_model := kernel
assembly_dir := $(project_root)/packages/boot/$(arch)
kernel_dir := $(project_root)/packages/kernel
kernel_src := $(kernel_dir)/src
kernel_lib_name := kernel
kernel_target := $(arch)-$(kernel_lib_name)
kernel_lib := $(build_root)/kernel/target/$(kernel_target)/debug/lib$(kernel_lib_name).a
linker_script := $(assembly_dir)/linker.ld
iso := $(build_root)/experiment-$(arch).iso
grub_cfg := grub.cfg
assembly_sources := $(wildcard $(assembly_dir)/*.asm)
assembly_objects := $(patsubst $(assembly_dir)/%.asm, $(build_root)/boot/x86_64/%.o, $(assembly_sources))
kernel_sources := $(wildcard $(kernel_src)/*.rs)
output_binary := $(build_root)/experiment.kernel

.PHONY: all link iso kernel run debug clean

all: iso

$(output_binary): $(linker_script) $(assembly_objects) $(kernel_lib)
	ld -n --gc-sections -T $(linker_script) -o $(output_binary) $(assembly_objects) $(kernel_lib)
	@# ld.lld --script $(linker_script) -o $(output_binary) $(assembly_objects)

$(build_root)/boot/x86_64/%.o: $(assembly_dir)/%.asm
	mkdir -p $(build_root)/boot/x86_64
	nasm -Wall -f elf64 -o $@ $<
	@# llvm-mc --triple=$(target_triple) --code-model=$(code_model) --filetype=obj -o=$@ $<

kernel: $(kernel_lib)

$(kernel_lib): export CARGO_HOME=/project/build/cache/cargo
$(kernel_lib): export CARGO_TARGET_DIR=/project/build/kernel/target
$(kernel_lib): rustfmt clippy $(kernel_sources)
	mkdir -p $(build_root)/kernel/target
	cd $(kernel_dir); export RUST_TARGET_PATH="`pwd`"; xargo build --target=$(kernel_target)

rustfmt: export CARGO_HOME=/project/build/cache/cargo
rustfmt: $(kernel_sources)
	cd $(kernel_dir); cargo fmt

clippy: export CARGO_HOME=/project/build/cache/cargo
clippy: $(kernel_sources)
	cd $(kernel_dir); cargo clippy

iso: $(output_binary)
	@echo 'Building iso.'
	@mkdir -p $(build_root)/iso/boot/grub
	@cp $(output_binary) $(build_root)/iso/boot/experiment.kernel
	@cp $(grub_cfg) $(build_root)/iso/boot/grub
	@grub-mkrescue /usr/lib/grub/i386-pc -o $(iso) $(build_root)/iso 2> /dev/null

run: all
	$(qemu_system_binary) -cdrom $(iso)

debug: all
	$(qemu_system_binary) -cdrom $(iso) -s -S

clean:
	rm -f $(assembly_objects)
	rm -f $(output_binary)
	rm -rf $(kernel_dir)/target
