//! The THISOSSTILLDOESNTHAVEANAMEWHATAREWEDOING kernel. As small as possible, hopefully.

#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![cfg_attr(feature = "cargo-clippy", deny(clippy))]
#![cfg_attr(feature = "cargo-clippy", deny(clippy_pedantic))]
#![cfg_attr(feature = "cargo-clippy", allow(shadow_same))]
#![cfg_attr(feature = "cargo-clippy", allow(doc_markdown))]
#![no_std]

extern crate multiboot2;
extern crate rlibc;
extern crate spin;
extern crate volatile;
#[macro_use]
extern crate bitflags;
extern crate x86_64;

#[macro_use]
mod vga_buffer;
mod memory;

use memory::FrameAllocator;

#[no_mangle]
/// The first Rust code that runs when we boot. On x86_64, it is called from long_start.asm.
pub extern "C" fn rust_main(multiboot_information_address: usize) {
    // ATTENTION: we have no guard page
    vga_buffer::clear_screen();

    let boot_info = unsafe { multiboot2::load(multiboot_information_address) };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    println!("Memory Areas:");
    for area in memory_map_tag.memory_areas() {
        println!(
            "\tstart: 0x{:x},\tlength: 0x{:x}",
            area.base_addr, area.length
        );
    }

    let elf_sections_tag = boot_info
        .elf_sections_tag()
        .expect("Elf-sections tag required");

    println!("Kernel Sections:");
    for section in elf_sections_tag.sections() {
        println!(
            "\taddr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
            section.addr, section.size, section.flags
        );
    }

    let kernel_start = elf_sections_tag
        .sections()
        .map(|s| s.addr)
        .min()
        .expect("elf sections tag required");
    let kernel_end = elf_sections_tag
        .sections()
        .map(|s| s.addr + s.size)
        .max()
        .expect("elf sections tag required");

    let multiboot_start = multiboot_information_address;
    let multiboot_end = multiboot_start + (boot_info.total_size as usize);

    println!(
        "kernel_start: 0x{:x} kernel_end: 0x{:x}",
        kernel_start, kernel_end
    );

    println!(
        "multiboot_start: 0x{:x} multiboot_end: 0x{:x}",
        multiboot_start, multiboot_end
    );

    #[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
    let mut frame_allocator = memory::AreaFrameAllocator::new(
        kernel_start as usize,
        kernel_end as usize,
        multiboot_start,
        multiboot_end,
        memory_map_tag.memory_areas(),
    );

    enable_nxe_bit();
    enable_write_protect_bit();
    memory::remap_kernel(&mut frame_allocator, boot_info);

    frame_allocator.allocate_frame();
    println!("Kernel Successfully Remapped");

    /*
    for i in 0.. {
        if frame_allocator.allocate_frame().is_none() {
            println!("Allocated {} frames", i);
            break;
        }
    }
    */

    #[cfg_attr(feature = "cargo-clippy", allow(empty_loop))]
    loop {}
}

/// Enable the NXE bit in the extended feature register (EFER) allowing the NO_EXECUTE bit to be set on pages
fn enable_nxe_bit() {
    use x86_64::registers::msr::{rdmsr, wrmsr, IA32_EFER};
    let nxe_bit = 1 << 11;
    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | nxe_bit);
    }
}

/// Enable the write protect bit, so that the kernel can not write to pages not flagged as WRITABLE
fn enable_write_protect_bit() {
    use x86_64::registers::control_regs::{Cr0, cr0, cr0_write};
    unsafe { cr0_write(cr0() | Cr0::WRITE_PROTECT) };
}

#[lang = "eh_personality"]
/// The Rust compiler requires this for exception handling. Currently a no-op.
extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
/// The Rust compiler requires this for panic handling. Currently just loops forever.
pub extern "C" fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("PANIC in {} at line {}:", file, line);
    println!("\t{}", fmt);
    #[cfg_attr(feature = "cargo-clippy", allow(empty_loop))]
    loop {}
}
