run_arch := x86_64
qemu_system_binary := qemu-system-$(run_arch)
code_model := kernel
linker_script := linker.ld
iso := experiment-$(run_arch).iso
grub_cfg := grub.cfg
assembly_sources := $(wildcard *.S)
assembly_objects := $(patsubst %.S, %.o, $(assembly_sources))
output_binary := test.kernel

.PHONY: all link iso run debug clean

all: iso

link: $(output_binary)
$(output_binary): $(assembly_objects)
	ld.lld --script $(linker_script) -o $(output_binary) $(assembly_objects)

%.o: %.S
	nasm -f elf64 -o $@ $<
	@# llvm-mc --triple=$(target_triple) --code-model=$(code_model) --filetype=obj -o=$@ $<

iso: $(iso)

$(iso):	link
	@mkdir -p iso/boot/grub
	@cp $(output_binary) iso/boot/grub/experiment.kernel
	@cp $(grub_cfg) iso/boot/grub
	@grub-mkrescue -o $(iso) iso 2> /dev/null
	@rm -r iso

run: all
	$(qemu_system_binary) -cdrom $(iso)

debug: all
	$(qemu_system_binary) -cdrom $(iso) -s -S

clean:
	rm -f *.o
	rm -f $(output_binary)
