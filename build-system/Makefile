arch := x86_64
qemu_system_binary := qemu-system-$(arch)
code_model := kernel
assembly_dir := ../packages/boot/$(arch)
linker_script := $(assembly_dir)/linker.ld
iso := experiment-$(arch).iso
grub_cfg := grub.cfg
assembly_sources := $(wildcard $(assembly_dir)/*.asm)
assembly_objects := $(patsubst $(assembly_dir)/%.asm, $(assembly_dir)/%.o, $(assembly_sources))
output_binary := experiment.kernel

.PHONY: all link iso run debug clean

all: iso

link: $(output_binary)
$(output_binary): $(assembly_objects)
	ld -n -T $(linker_script) -o $(output_binary) $(assembly_objects)
	@# ld.lld --script $(linker_script) -o $(output_binary) $(assembly_objects)

$(assembly_dir)/%.o: $(assembly_dir)/%.asm
	nasm -f elf64 -o $@ $<
	@# llvm-mc --triple=$(target_triple) --code-model=$(code_model) --filetype=obj -o=$@ $<

iso: $(iso)

$(iso):	link
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
