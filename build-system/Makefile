arch := x86_64
project_root := ..
qemu_system_binary := qemu-system-$(arch)
code_model := kernel
assembly_dir := $(project_root)/packages/boot/$(arch)
kernel_dir := $(project_root)/packages/kernel
kernel_src := $(kernel_dir)/src
kernel_lib_name := kernel
kernel_target := $(arch)-$(kernel_lib_name)
kernel_lib := $(kernel_dir)/target/$(kernel_target)/debug/lib$(kernel_lib_name).a
linker_script := $(assembly_dir)/linker.ld
iso := experiment-$(arch).iso
grub_cfg := grub.cfg
assembly_sources := $(wildcard $(assembly_dir)/*.asm)
assembly_objects := $(patsubst $(assembly_dir)/%.asm, $(assembly_dir)/%.o, $(assembly_sources))
kernel_sources := $(wildcard $(kernel_src)/*.rs)
output_binary := experiment.kernel

.PHONY: all link iso kernel run debug clean

all: iso

link: $(linker_script) $(output_binary)

$(output_binary): $(assembly_objects) $(kernel_lib)
	ld -n --gc-sections -T $(linker_script) -o $(output_binary) $(assembly_objects) $(kernel_lib)
	@# ld.lld --script $(linker_script) -o $(output_binary) $(assembly_objects)

$(assembly_dir)/%.o: $(assembly_dir)/%.asm
	nasm -Wall -f elf64 -o $@ $<
	@# llvm-mc --triple=$(target_triple) --code-model=$(code_model) --filetype=obj -o=$@ $<

kernel: $(kernel_lib)

$(kernel_lib): $(kernel_sources)
	cd $(kernel_dir); export RUST_TARGET_PATH="`pwd`"; xargo build --target=$(kernel_target)

iso:	link
	@mkdir -p iso/boot/grub
	@cp $(output_binary) iso/boot/experiment.kernel
	@cp $(grub_cfg) iso/boot/grub
	@grub-mkrescue /usr/lib/grub/i386-pc -o $(iso) iso 2> /dev/null
	@# @rm -r iso

run: all
	$(qemu_system_binary) -cdrom $(iso)

debug: all
	$(qemu_system_binary) -cdrom $(iso) -s -S

clean:
	rm -f $(assembly_objects)
	rm -f $(output_binary)
	rm -rf $(kernel_dir)/target
