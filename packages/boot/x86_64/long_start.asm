global long_start

section .text
bits 64
long_start:
	; load 0 into all data segment registers
	mov ax, 0
	mov ss, ax
	mov ds, ax
	mov es, ax
	mov fs, ax
	mov gs, ax

	extern rust_main
	call rust_main

	; print `OKAY` to screen
	mov rax, 0x02590241024b024f
	mov qword [0xb8000], rax
	hlt